#[macro_use]
extern crate glium;
extern crate cgmath;

use std::io::Read;
use std::fs::File;
use std::path::{Path, PathBuf};

use glium::{DisplayBuild, Surface};
use glium::glutin::Event;

use cgmath::{Vector3, Point3};

mod common;
use common::Camera;

fn get_project_root() -> PathBuf {
    let exe_dir = std::env::current_exe().unwrap();
    let mut parent_dir = exe_dir.parent().unwrap();
    while !parent_dir.ends_with("rusty") {
        parent_dir = parent_dir.parent().unwrap();
    }
    parent_dir.to_path_buf()
}

fn read_shader_from_file(shader_path: &Path) -> String {
    let mut file = File::open(shader_path).unwrap();
    let mut shader_src = String::new();
    file.read_to_string(&mut shader_src).unwrap();
    shader_src
}

fn main() {
    let display = glium::glutin::WindowBuilder::new().with_depth_buffer(24).build_glium().unwrap();

    let root_path = get_project_root();
    let scene = common::load_scene(&root_path.join("scenes/cornell/cornell_chesterfield.obj"), &display);
    //let scene = common::load_scene(&root_path.join("scenes/cornell-box/CornellBox-Original.obj"), &display);
    //let scene = common::load_scene(&root_path.join("scenes/nanosuit/nanosuit.obj"), &display);

    let src_path = root_path.join("src");
    let vertex_shader_src = read_shader_from_file(&src_path.join("vertex.glsl"));
    let fragment_shader_src = read_shader_from_file(&src_path.join("fragment.glsl"));
    let program = glium::Program::from_source(&display, &vertex_shader_src, &fragment_shader_src, None)
        .expect("Failed to create program!");

    let params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            .. Default::default()
        },
        .. Default::default()
    };

    let mut camera = Camera::new(Point3::new(0.0, 0.0, -2.0), Vector3::new(0.0, 0.0, 1.0));

    loop {
        let mut target = display.draw();

        let (width, height) = target.get_dimensions();
        let camera_to_clip = cgmath::perspective(cgmath::Rad(std::f32::consts::PI / 3.0),
                                              width as f32 / height as f32, 0.01, 1000.0f32);
        let world_to_camera = camera.get_world_to_camera();

        target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
        for mesh in &scene.meshes {
            mesh.draw(&mut target, &program, &params, camera_to_clip * world_to_camera);
        }
        target.finish().unwrap();

        for event in display.poll_events() {
            camera.handle_event(&event);
            match event {
                Event::Closed => return,
                _ => ()
            }
        }
    }
}
