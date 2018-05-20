#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#[macro_use]
extern crate glium;
extern crate cgmath;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

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

fn load_scene<F: Facade>(path: &Path, facade: &F) -> (Arc<Scene>, GPUScene, Camera) {
    let scene = Scene::new(path);
    let gpu_scene = scene.upload_data(facade);
    let mut camera = Camera::new(scene.center() + scene.size() * Vector3::new(0.0, 0.0, 1.0f32),
                                 Vector3::new(0.0, 0.0, -1.0f32));
    camera.set_scale(scene.size());
    (Arc::new(scene), gpu_scene, camera)
}

fn main() {
    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new();
    let context = glium::glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(window, context, &events_loop).expect("Failed to create display");

    let root_path = get_project_root();
    // TODO: Enable use of arbitrary scene
    let scenes: HashMap<VirtualKeyCode, PathBuf> =
        [(VirtualKeyCode::Key1, root_path.join("scenes/plane.obj")),
         (VirtualKeyCode::Key2, root_path.join("scenes/cornell/cornell.obj")),
         (VirtualKeyCode::Key3, root_path.join("scenes/cornell/cornell_chesterfield.obj")),
         (VirtualKeyCode::Key4, root_path.join("scenes/cornell-box/CornellBox-Original.obj")),
         (VirtualKeyCode::Key5, root_path.join("scenes/cornell-box/CornellBox-Glossy.obj")),
         (VirtualKeyCode::Key6, root_path.join("scenes/cornell-box/CornellBox-Water.obj")),
         (VirtualKeyCode::Key7, root_path.join("scenes/nanosuit/nanosuit.obj")),
         (VirtualKeyCode::Key8, root_path.join("scenes/sibenik/sibenik.obj"))]
        .iter().cloned().collect();

    let (mut scene, mut gpu_scene, mut camera) =
        load_scene(&scenes[&VirtualKeyCode::Key1], &display);
    let gl_renderer = renderer::GLRenderer::new(&display);
    let mut pt_renderer = renderer::PTRenderer::new(&display);

    let mut input = InputState::new();
    let mut trace = false;
    let mut quit = false;

    loop {
        let mut target = display.draw();

        let (width, height) = target.get_dimensions();
        // Don't draw if the window is minimized
        if width != 0 && height != 0 {
            let camera_to_clip = camera.get_camera_to_clip(width, height);
            let world_to_camera = camera.get_world_to_camera();

            target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
            if trace {
                pt_renderer.render(&mut target);
            } else {
                gl_renderer.render(&gpu_scene, &mut target, camera_to_clip * world_to_camera);
            }
        }
        target.finish().unwrap();

        events_loop.poll_events(|event| {
            input.update(&event);
            match event {
                Event::WindowEvent{event: WindowEvent::KeyboardInput{input, ..}, ..} => {
                    match input {
                        KeyboardInput{state: ElementState::Pressed,
                                      virtual_keycode: Some(VirtualKeyCode::Space), ..} => {
                            trace = !trace;
                            if trace {
                                pt_renderer.start_render(&display, &scene, &camera, width, height);
                            } else {
                                pt_renderer.stop_threads();
                            }
                        },
                        KeyboardInput{state: ElementState::Pressed,
                                      virtual_keycode: Some(ref keycode), ..} => {
                            if let Some(scene_to_load) = scenes.get(keycode) {
                                let res = load_scene(scene_to_load, &display);
                                scene = res.0;
                                gpu_scene = res.1;
                                camera = res.2;
                            }
                        },
                        _ => ()
                    }
                }
                Event::WindowEvent{event: WindowEvent::Closed, ..} => quit = true,
                _ => ()
            }
        });
        camera.process_input(&input);
        input.reset_deltas();
        if quit {
            pt_renderer.stop_threads();
            return;
        }
    }
}
