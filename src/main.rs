#![feature(rust_2018_preview)]
#![feature(euclidean_division)]
#![feature(try_trait)]

mod aabb;
mod bvh;
mod camera;
mod color;
mod gl_renderer;
mod input;
mod material;
mod mesh;
mod obj_load;
mod pt_renderer;
mod scene;
mod stats;
mod triangle;
mod util;
mod vertex;

use std::path::{Path, PathBuf};

use chrono::Local;

use glium::Surface;
use glium::glutin::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode};

use crate::gl_renderer::GLRenderer;
use crate::input::InputState;
use crate::pt_renderer::PTRenderer;

fn main() {
    match std::env::args().nth(1).as_ref().map(|s| s.as_str()) {
        Some(_) => benchmark(),
        None => online_render(),
    }
}

fn benchmark() {
    let scenes = ["plane", "cornell-glossy", "cornell-water", "nanosuit", "sibenik",];
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let save_dir = root_dir.join("results");
    std::fs::create_dir_all(save_dir.clone()).unwrap();
    for scene in &scenes {
        let mut output_file = save_dir.join(scene);
        output_file.set_extension("png");
        offline_render(scene, &output_file, 2);
    }
    let stats_file = save_dir.join(Local::now().format("benchmark_%F_%H%M%S.txt").to_string());
    stats::print_and_save(&stats_file);
}

fn offline_render(scene_name: &str, output_file: &Path, iterations: u32) {
    stats::new_scene(scene_name);
    let _t = stats::time("Total");
    let mut pt_renderer = PTRenderer::new();
    println!("{}...", scene_name);
    let (scene, camera) = util::load_cpu_scene(scene_name);
    pt_renderer.offline_render(&scene, &camera, iterations);
    pt_renderer.save_image(output_file);
}

fn online_render() {
    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new();
    let context = glium::glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(window, context, &events_loop).expect("Failed to create display");

    let (mut scene, mut gpu_scene, mut camera) =
        util::load_gpu_scene(VirtualKeyCode::Key1, &display).unwrap();
    let gl_renderer = GLRenderer::new(&display);
    let mut pt_renderer = PTRenderer::new();

    let mut input = InputState::new();
    let mut trace = false;
    let mut quit = false;

    loop {
        let mut target = display.draw();

        camera.update_viewport(target.get_dimensions());
        // Don't draw if the window is minimized
        if camera.width != 0 && camera.height != 0 {
            target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
            if trace {
                pt_renderer.render(&mut target);
            } else {
                gl_renderer.render(&mut target, &gpu_scene, &camera);
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
                                pt_renderer.online_render(&display, &scene, &camera);
                            } else {
                                pt_renderer.stop_threads();
                            }
                        },
                        KeyboardInput{state: ElementState::Pressed,
                                      virtual_keycode: Some(keycode), ..} => {
                            if let Some(res) = util::load_gpu_scene(keycode, &display) {
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
            if trace {
                pt_renderer.stop_threads();
            }
            return;
        }
    }
}
