#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#[macro_use]
extern crate glium;
extern crate cgmath;

use std::path::PathBuf;

use glium::Surface;
use glium::glutin::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode};

use cgmath::{Vector3, Point3};

mod camera;
mod input;
mod renderer;
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
    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new();
    let context = glium::glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(window, context, &events_loop).expect("Failed to create display");

    let root_path = get_project_root();
    // TODO: Enable use of arbitrary scene
    let scenes = vec!("scenes/plane.obj",
                      "scenes/cornell/cornell.obj",
                      "scenes/cornell/cornell_chesterfield.obj",
                      "scenes/cornell-box/CornellBox-Original.obj",
                      "scenes/cornell-box/CornellBox-Glossy.obj",
                      "scenes/cornell-box/CornellBox-Water.obj",
                      "scenes/nanosuit/nanosuit.obj",
                      "scenes/sibenik/sibenik.obj");
    let mut scene = scene::Scene::new(&root_path.join(scenes[0]), &display);
    let gl_renderer = renderer::GLRenderer::new(&display);
    let pt_renderer = renderer::PTRenderer::new(&display);

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
                pt_renderer.render(&scene, &mut target, &display, width as usize, height as usize, &camera);
            } else {
                gl_renderer.render(&scene, &mut target, camera_to_clip * world_to_camera);
            }
        }
        target.finish().unwrap();

        { // TODO clean this up
        let event_handler = |event| {
            input.update(&event);
            match event {
                Event::WindowEvent{event: WindowEvent::KeyboardInput{input, ..}, ..} => {
                    match input {
                        KeyboardInput{state: ElementState::Pressed,
                                      virtual_keycode: Some(VirtualKeyCode::Space), ..}
                        => trace = !trace,
                        // Get number pressed based on the keycode
                        KeyboardInput{state: ElementState::Pressed, scancode, ..} => {
                            let i = scancode as usize - 2;
                            if 0 < i && i < scenes.len() {
                                scene = scene::Scene::new(&root_path.join(scenes[i]), &display);
                                camera.set_position(Point3::from(scene.get_center())
                                                    + scene.get_size() * Vector3::new(0.0, 0.0, 1.0f32),
                                                    Vector3::new(0.0, 0.0, -1.0f32));
                                camera.set_scale(scene.get_size());
                            }
                        },
                        _ => ()
                    }
                }
                Event::WindowEvent{event: WindowEvent::Closed, ..} => std::process::exit(0),
                _ => ()
            }
        };
        events_loop.poll_events(event_handler);
        }
        camera.process_input(&input);
        input.reset_deltas();
    }
}
