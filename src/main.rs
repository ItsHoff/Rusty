#[macro_use]
extern crate glium;
extern crate cgmath;

use std::io::Read;
use std::fs::File;
use std::path::{Path, PathBuf};

use glium::{DisplayBuild, Surface};
use glium::glutin::{Event, ElementState};

use cgmath::{Vector3, Point3};

mod scene;
mod camera;
use camera::Camera;

/// Get the root directory of the project
fn get_project_root() -> PathBuf {
    let exe_dir = std::env::current_exe().unwrap();
    let mut parent_dir = exe_dir.parent().unwrap();
    // This fails if the executable is not in the project tree
    while !parent_dir.ends_with("rusty") {
        parent_dir = parent_dir.parent().unwrap();
    }
    parent_dir.to_path_buf()
}

/// Read a shader found at path
fn read_shader_from_file(shader_path: &Path) -> String {
    let mut file = File::open(shader_path).unwrap();
    let mut shader_src = String::new();
    file.read_to_string(&mut shader_src).unwrap();
    shader_src
}

fn main() {
    let display = glium::glutin::WindowBuilder::new().with_depth_buffer(24).build_glium().unwrap();

    let root_path = get_project_root();
    // TODO: Enable use of arbitrary scene
    let scenes = vec!("scenes/cornell/cornell.obj",
                      "scenes/cornell/cornell_chesterfield.obj",
                      "scenes/cornell-box/CornellBox-Original.obj",
                      "scenes/cornell-box/CornellBox-Glossy.obj",
                      "scenes/cornell-box/CornellBox-Water.obj",
                      "scenes/nanosuit/nanosuit.obj",
                      "scenes/sibenik/sibenik.obj");
    let mut scene = scene::load_scene(&root_path.join(scenes[0]), &display);

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

    let mut camera = Camera::new(Point3::from(scene.get_center())
                                 + scene.get_size() * Vector3::new(0.0, 0.0, 1.0f32),
                                 Vector3::new(0.0, 0.0, -1.0f32));

    loop {
        let mut target = display.draw();

        let (width, height) = target.get_dimensions();
        let camera_to_clip = camera.get_camera_to_clip(width, height);
        let world_to_camera = camera.get_world_to_camera();

        target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
        // Draw meshes one at a time
        for mesh in &scene.meshes {
            mesh.draw(&mut target, &program, &params, camera_to_clip * world_to_camera);
        }
        target.finish().unwrap();

        for event in display.poll_events() {
            camera.handle_event(&event);
            match event {
                // Not sure how portable this is
                Event::KeyboardInput(ElementState::Pressed, code @ 2...11, _) => {
                    let i = code as usize - 2;
                    if i < scenes.len() {
                        scene = scene::load_scene(&root_path.join(scenes[i]), &display);
                        camera.set_position(Point3::from(scene.get_center())
                                 + scene.get_size() * Vector3::new(0.0, 0.0, 1.0f32),
                                 Vector3::new(0.0, 0.0, -1.0f32));
                    }
                }
                Event::Closed => return,
                _ => ()
            }
        }
    }
}
