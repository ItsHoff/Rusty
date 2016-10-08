#[macro_use]
extern crate glium;
extern crate cgmath;

use std::io::Read;
use std::fs::File;
use std::path::{Path, PathBuf};

use glium::{DisplayBuild, Surface};

mod common;

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
    //let scene = common::load_scene(&root_path.join("scenes/cornell/cornell_chesterfield.obj"), &display);
    //let scene = common::load_scene(&root_path.join("scenes/cornell-box/CornellBox-Original.obj"), &display);
    let scene = common::load_scene(&root_path.join("scenes/nanosuit/nanosuit.obj"), &display);

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

    let mut camera_pos = cgmath::Point3::new(0.0, 0.0, 80.0);

    loop {
        let mut target = display.draw();

        let (width, height) = target.get_dimensions();
        let perspective = cgmath::perspective(cgmath::Rad(std::f32::consts::PI / 3.0),
                                              width as f32 / height as f32, 0.01, 1000.0f32);
        let camera = cgmath::Matrix4::look_at(camera_pos,
                                              cgmath::Point3::new(1.0, 0.0, 0.0),
                                              cgmath::vec3(0.0, 1.0, 0.0f32));

        target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
        for mesh in &scene.meshes {
            mesh.draw(&mut target, &program, &params, perspective*camera);
        }
        target.finish().unwrap();

        for event in display.poll_events() {
            match event {
                glium::glutin::Event::Closed => return,
                glium::glutin::Event::KeyboardInput(_, _, _) => {
                    camera_pos.z -= 0.5;
                }
                _ => ()
            }
        }
    }
}
