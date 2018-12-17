#![feature(duration_float)]
#![feature(euclidean_division)]
#![feature(self_struct_ctor)]
#![feature(try_trait)]

use std::path::PathBuf;

use chrono::Local;

use glium::glutin::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use glium::Surface;

mod aabb;
mod bsdf;
mod bvh;
mod camera;
mod color;
mod config;
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

use self::config::RenderConfig;
use self::gl_renderer::GLRenderer;
use self::input::InputState;
use self::pt_renderer::PTRenderer;

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

    // Initialize an OpenGL context that is needed for post-processing
    let events_loop = glium::glutin::EventsLoop::new();
    // Preferably this wouldn't need use a window at all but alas this is the closest I have gotten.
    // There exists HeadlessContext but that still pops up a window (atleast on Windows).
    // TODO: Maybe change this such that the window displays the current render?
    let window = glium::glutin::WindowBuilder::new()
        .with_dimensions(glium::glutin::dpi::LogicalSize::new(0.0, 0.0))
        .with_visibility(false)
        .with_decorations(false)
        .with_title("Benchmark");
    let context = glium::glutin::ContextBuilder::new();
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    for scene_name in &scenes {
        stats::new_scene(scene_name);
        let _t = stats::time("Total");
        println!("{}...", scene_name);
        let (scene, camera) = load::load_cpu_scene(scene_name, &config);
        let pt_renderer = PTRenderer::offline_render(&display, &scene, &camera, &config);

        stats::time("Post-process");
        let scene_dir = output_dir.join(scene_name);
        std::fs::create_dir_all(scene_dir.clone()).unwrap();
        let timestamped_image = scene_dir.join(format!("{}_{}.png", scene_name, time_stamp));
        pt_renderer.save_image(&display, &timestamped_image);
        // Make a copy to the main output directory
        let default_image = output_dir.join(scene_name).with_extension("png");
        std::fs::copy(timestamped_image, default_image).unwrap();
    }
    let stats_file = output_dir.join(format!("benchmark_{}.txt", time_stamp));
    stats::print_and_save(&stats_file);
}

fn online_render() {
    let mut config = RenderConfig::default();
    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new()
        .with_dimensions(config.dimensions())
        .with_resizable(false); // TODO: enable resizing
    let context = glium::glutin::ContextBuilder::new().with_depth_buffer(24);
    let display =
        glium::Display::new(window, context, &events_loop).expect("Failed to create display");

    let (mut scene, mut gpu_scene, mut camera) =
        load::load_gpu_scene(VirtualKeyCode::Key1, &display, &config).unwrap();
    let gl_renderer = GLRenderer::new(&display);
    let mut pt_renderer: Option<PTRenderer> = None;

    let mut input = InputState::new();
    let mut quit = false;

    loop {
        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
        if let Some(renderer) = &mut pt_renderer {
            renderer.update_and_render(&mut target);
        } else {
            gl_renderer.render(&mut target, &gpu_scene, &camera);
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
                        if pt_renderer.is_some() {
                            pt_renderer = None;
                        } else {
                            pt_renderer =
                                Some(PTRenderer::start_render(&display, &scene, &camera, &config));
                        }
                    }
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::C),
                        ..
                    } => println!("camera: {:?}", camera.pos),
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(keycode),
                        ..
                    } => {
                        if pt_renderer.is_none() {
                            if let Some(res) = load::load_gpu_scene(keycode, &display, &config) {
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
            return;
        }
    }
}
