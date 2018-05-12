#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#[macro_use]
extern crate glium;
extern crate cgmath;

use std::path::{Path, PathBuf};

use glium::Surface;
use glium::backend::Facade;
use glium::glutin::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode};

use cgmath::Vector3;

mod aabb;
mod camera;
mod input;
mod renderer;
mod scene;

use camera::Camera;
use input::InputState;
use scene::{Scene, GPUScene};

/// Get the root directory of the project
fn get_project_root() -> PathBuf {
    let exe_dir = std::env::current_exe().unwrap();
    let mut parent_dir = exe_dir.parent().unwrap();
    // This fails if the executable is not in the project tree
    // or the directory is renamed
    while !(parent_dir.ends_with("rusty") || parent_dir.ends_with("Rusty")) {
        parent_dir = parent_dir.parent()
            .expect(&format!("Failed to find project root from {:?}!", exe_dir));
    }
    parent_dir.to_path_buf()
}

fn new_scene<F: Facade>(path: &Path, facade: &F) -> (Scene, GPUScene, Camera) {
    let scene = Scene::new(path);
    let gpu_scene = scene.upload_data(facade);
    let mut camera = Camera::new(scene.center() + scene.size() * Vector3::new(0.0, 0.0, 1.0f32),
                                 Vector3::new(0.0, 0.0, -1.0f32));
    camera.set_scale(scene.size());
    (scene, gpu_scene, camera)
}

fn main() {
    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new();
    let context = glium::glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(window, context, &events_loop).expect("Failed to create display");

    let root_path = get_project_root();
    // TODO: Enable use of arbitrary scene
    let scenes = vec!(root_path.join("scenes/plane.obj"),
                      root_path.join("scenes/cornell/cornell.obj"),
                      root_path.join("scenes/cornell/cornell_chesterfield.obj"),
                      root_path.join("scenes/cornell-box/CornellBox-Original.obj"),
                      root_path.join("scenes/cornell-box/CornellBox-Glossy.obj"),
                      root_path.join("scenes/cornell-box/CornellBox-Water.obj"),
                      root_path.join("scenes/nanosuit/nanosuit.obj"),
                      root_path.join("scenes/sibenik/sibenik.obj"));

    let (mut scene, mut gpu_scene, mut camera) = new_scene(&scenes[0], &display);
    let gl_renderer = renderer::GLRenderer::new(&display);
    let mut pt_renderer = renderer::PTRenderer::new(&display);

    let mut input = InputState::new();
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
                pt_renderer.render(&scene, &mut target, &camera);
            } else {
                gl_renderer.render(&gpu_scene, &mut target, camera_to_clip * world_to_camera);
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
                                      virtual_keycode: Some(VirtualKeyCode::Space), ..} => {
                            trace = !trace;
                            if trace {
                                pt_renderer.start_render(&display, width, height);
                            }
                        },
                        KeyboardInput{state: ElementState::Pressed,
                                      virtual_keycode: Some(VirtualKeyCode::Key1), ..} => {
                            let res = new_scene(&scenes[0], &display);
                            scene = res.0;
                            gpu_scene = res.1;
                            camera = res.2;
                        },
                        KeyboardInput{state: ElementState::Pressed,
                                      virtual_keycode: Some(VirtualKeyCode::Key2), ..} => {
                            let res = new_scene(&scenes[1], &display);
                            scene = res.0;
                            gpu_scene = res.1;
                            camera = res.2;
                        },
                        KeyboardInput{state: ElementState::Pressed,
                                      virtual_keycode: Some(VirtualKeyCode::Key3), ..} => {
                            let res = new_scene(&scenes[2], &display);
                            scene = res.0;
                            gpu_scene = res.1;
                            camera = res.2;
                        },
                        KeyboardInput{state: ElementState::Pressed,
                                      virtual_keycode: Some(VirtualKeyCode::Key4), ..} => {
                            let res = new_scene(&scenes[3], &display);
                            scene = res.0;
                            gpu_scene = res.1;
                            camera = res.2;
                        },
                        KeyboardInput{state: ElementState::Pressed,
                                      virtual_keycode: Some(VirtualKeyCode::Key5), ..} => {
                            let res = new_scene(&scenes[4], &display);
                            scene = res.0;
                            gpu_scene = res.1;
                            camera = res.2;
                        },
                        KeyboardInput{state: ElementState::Pressed,
                                      virtual_keycode: Some(VirtualKeyCode::Key6), ..} => {
                            let res = new_scene(&scenes[5], &display);
                            scene = res.0;
                            gpu_scene = res.1;
                            camera = res.2;
                        },
                        KeyboardInput{state: ElementState::Pressed,
                                      virtual_keycode: Some(VirtualKeyCode::Key7), ..} => {
                            let res = new_scene(&scenes[6], &display);
                            scene = res.0;
                            gpu_scene = res.1;
                            camera = res.2;
                        },
                        KeyboardInput{state: ElementState::Pressed,
                                      virtual_keycode: Some(VirtualKeyCode::Key8), ..} => {
                            let res = new_scene(&scenes[7], &display);
                            scene = res.0;
                            gpu_scene = res.1;
                            camera = res.2;
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
