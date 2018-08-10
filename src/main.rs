#![feature(rust_2018_preview)]
#![feature(rust_2018_idioms)]
#![feature(nll)]
#![feature(euclidean_division)]
#![feature(try_trait)]

mod aabb;
mod benchmark;
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
mod triangle;
mod util;
mod vertex;

use glium::Surface;
use glium::glutin::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode};

use crate::gl_renderer::GLRenderer;
use crate::input::InputState;
use crate::pt_renderer::PTRenderer;

fn main() {
    match std::env::args().nth(1).as_ref().map(|s| s.as_str()) {
        Some("all") => benchmark::benchmark_all(),
        Some("bvh") => benchmark::benchmark_bvh_build(),
        Some("render") => benchmark::benchmark_render(),
        Some(unknown) => println!("Unknown benchmark {}", unknown),
        None => default_render(),
    }
}

fn default_render() {
    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new();
    let context = glium::glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(window, context, &events_loop).expect("Failed to create display");

    let (mut scene, mut gpu_scene, mut camera) = util::load_scene(VirtualKeyCode::Key1, &display).unwrap();
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

        // TODO: Fix crash when tracing is started during scene load
        //   * Camera is missing viewport info immediately after load when the tracing starts
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
                            if let Some(res) = util::load_scene(keycode, &display) {
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
