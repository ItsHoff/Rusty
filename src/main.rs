#![feature(duration_float)]
#![feature(euclidean_division)]
#![feature(self_struct_ctor)]
#![feature(try_trait)]

use std::path::PathBuf;

use chrono::Local;

use glium::glutin::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use glium::Surface;

mod aabb;
mod bvh;
mod camera;
mod color;
mod consts;
mod gl_renderer;
mod index_ptr;
mod input;
mod intersect;
mod light;
mod load;
mod material;
mod mesh;
mod obj_load;
mod pt_renderer;
mod scene;
mod stats;
mod texture;
mod triangle;
mod util;
mod vertex;

use self::gl_renderer::GLRenderer;
use self::input::InputState;
use self::pt_renderer::{PTRenderer, RenderConfig};

type Float = f64;

fn main() {
    match std::env::args().nth(1).as_ref().map(|s| s.as_str()) {
        Some(_) => benchmark(),
        None => online_render(),
    }
}

fn benchmark() {
    let scenes = [
        "plane",
        "cornell-glossy",
        "cornell-water",
        "indirect",
        "conference",
        "sponza",
    ];
    let config = RenderConfig::benchmark();
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output_dir = root_dir.join("results");
    std::fs::create_dir_all(output_dir.clone()).unwrap();
    let time_stamp = Local::now().format("%F_%H%M%S").to_string();
    for scene_name in &scenes {
        stats::new_scene(scene_name);
        let _t = stats::time("Total");
        let mut pt_renderer = PTRenderer::new();
        println!("{}...", scene_name);
        let (scene, camera) = load::load_cpu_scene(scene_name);
        pt_renderer.offline_render(&scene, &camera, &config);

        // Save timestamped version in a addition to the default image
        let scene_dir = output_dir.join(scene_name);
        std::fs::create_dir_all(scene_dir.clone()).unwrap();
        let timestamped_image = scene_dir.join(format!("{}_{}.png", scene_name, time_stamp));
        let default_image = output_dir.join(scene_name).with_extension("png");
        pt_renderer.save_image(&timestamped_image);
        pt_renderer.save_image(&default_image);
    }
    let stats_file = output_dir.join(format!("benchmark_{}.txt", time_stamp));
    stats::print_and_save(&stats_file);
}

fn online_render() {
    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new();
    let context = glium::glutin::ContextBuilder::new().with_depth_buffer(24);
    let display =
        glium::Display::new(window, context, &events_loop).expect("Failed to create display");

    let (mut scene, mut gpu_scene, mut camera) =
        load::load_gpu_scene(VirtualKeyCode::Key1, &display).unwrap();
    let gl_renderer = GLRenderer::new(&display);
    let mut pt_renderer = PTRenderer::new();

    let mut config = RenderConfig::default();
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
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput { input, .. },
                    ..
                } => match input {
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Space),
                        ..
                    } => {
                        trace = !trace;
                        if trace {
                            pt_renderer.online_render(&display, &scene, &camera, &config);
                        } else {
                            pt_renderer.stop_threads();
                        }
                    }
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(keycode),
                        ..
                    } => {
                        if !trace {
                            if let Some(res) = load::load_gpu_scene(keycode, &display) {
                                scene = res.0;
                                gpu_scene = res.1;
                                camera = res.2;
                            }
                            config.handle_key(keycode);
                        }
                    }
                    _ => (),
                },
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => quit = true,
                _ => (),
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
