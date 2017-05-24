#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#[macro_use]
extern crate glium;
extern crate cgmath;

use std::path::PathBuf;

use glium::{DisplayBuild, Surface};
use glium::glutin::{Event, ElementState, VirtualKeyCode};

use cgmath::{Vector3, Point3};

mod camera;
mod input;
mod scene;

use camera::Camera;
use input::InputState;

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
    let mut scene = scene::Scene::init(&root_path.join(scenes[0]), &display);

    let mut input = InputState::new();
    let mut camera = Camera::new(Point3::from(scene.get_center())
                                 + scene.get_size() * Vector3::new(0.0, 0.0, 1.0f32),
                                 Vector3::new(0.0, 0.0, -1.0f32));
    camera.set_scale(scene.get_size());
    let mut trace = false;

    loop {
        let mut target = display.draw();

        let (width, height) = target.get_dimensions();
        // Don't draw if the window is minimized
        if width != 0 && height != 0 {
            let camera_to_clip = camera.get_camera_to_clip(width, height);
            let world_to_camera = camera.get_world_to_camera();

            target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
            if trace {
            } else {
                scene.draw(&mut target, camera_to_clip * world_to_camera);
            }
        }
        target.finish().unwrap();

        for event in display.poll_events() {
            input.update(&event);
            match event {
                // Get number pressed based on the keycode
                Event::KeyboardInput(ElementState::Pressed, code @ 2...11, _) => {
                    let i = code as usize - 2;
                    if i < scenes.len() {
                        scene = scene::Scene::init(&root_path.join(scenes[i]), &display);
                        camera.set_position(Point3::from(scene.get_center())
                                 + scene.get_size() * Vector3::new(0.0, 0.0, 1.0f32),
                                 Vector3::new(0.0, 0.0, -1.0f32));
                        camera.set_scale(scene.get_size());
                    }
                },
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Space)) => trace = !trace,
                Event::Closed => return,
                _ => ()
            }
        }
        camera.process_input(&input);
        input.reset_deltas();
    }
}
