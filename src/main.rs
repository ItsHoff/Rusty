#![feature(try_trait)]

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

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
mod float;
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
mod sample;
mod scattering;
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

// TODO: add comparison mode
fn main() {
    match std::env::args().nth(1).as_ref().map(std::string::String::as_str) {
        Some("hq") => high_quality(),
        Some("pt") => high_quality_pt(),
        Some("comp") => compare(),
        Some("b") => benchmark("bdpt", RenderConfig::bdpt_benchmark()),
        Some(_) => benchmark("", RenderConfig::benchmark()),
        None => online_render(),
    }
}

fn compare() {
    let scenes = [
        "cornell-sphere",
        "cornell-glossy",
        "cornell-water",
        "indirect",
        "conference",
        "sponza",
    ];
    let mut config = RenderConfig::benchmark();
    config.samples_per_dir *= 4;
    config.width /= 2;
    config.height /= 2;
    let output_dir = PathBuf::from("results").join("compare");
    offline_render(&scenes, "pt", &output_dir, config);
    config = RenderConfig::bdpt_benchmark();
    config.samples_per_dir *= 4;
    config.width /= 2;
    config.height /= 2;
    offline_render(&scenes, "bdpt", &output_dir, config.clone());
    config.mis = false;
    offline_render(&scenes, "no_mis", &output_dir, config);
}

fn high_quality_pt() {
    // TODO: Add command line switches to select scenes and config settings
    let scenes = [
        // "cornell-sphere",
        // "cornell-glossy",
        // "cornell-water",
        // "indirect",
        "conference",
        // "sponza",
    ];
    let tag = "pt_hq";
    let config = RenderConfig::high_quality_pt();
    let output_dir = PathBuf::from("results").join("hq");
    offline_render(&scenes, tag, &output_dir, config);
}

fn high_quality() {
    // TODO: Add command line switches to select scenes and config settings
    let scenes = [
        // "cornell-sphere",
        // "cornell-glossy",
        // "cornell-water",
        // "indirect",
        "conference",
        // "sponza",
    ];
    let tag = "hq";
    let config = RenderConfig::high_quality();
    let output_dir = PathBuf::from("results").join("hq");
    offline_render(&scenes, tag, &output_dir, config);
}

fn benchmark(tag: &str, config: RenderConfig) {
    let scenes = [
        "cornell-sphere",
        "cornell-glossy",
        "cornell-water",
        "indirect",
        "conference",
        "sponza",
    ];
    let output_dir = PathBuf::from("results");
    offline_render(&scenes, tag, &output_dir, config);
}

fn offline_render(scenes: &[&str], tag: &str, output_dir: &Path, config: RenderConfig) {
    let tag = if tag.is_empty() {
        tag.to_string()
    } else {
        format!("_{}", tag)
    };
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output_dir = root_dir.join(output_dir);
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
        .with_title("Rusty");
    let context = glium::glutin::ContextBuilder::new();
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    for scene_name in scenes {
        stats::new_scene(scene_name);
        let _t = stats::time("Total");
        println!("{}...", scene_name);
        let (scene, camera) = load::cpu_scene_from_name(scene_name, &config);
        let pt_renderer = PTRenderer::offline_render(&display, &scene, &camera, &config);

        stats::time("Post-process");
        let scene_prefix = format!("{}{}", scene_name, tag);
        let scene_dir = output_dir.join(&scene_prefix);
        std::fs::create_dir_all(scene_dir.clone()).unwrap();
        let timestamped_image = scene_dir.join(format!("{}_{}.png", scene_prefix, time_stamp));
        pt_renderer.save_image(&display, &timestamped_image);
        // Make a copy to the main output directory
        let default_image = output_dir.join(scene_prefix).with_extension("png");
        std::fs::copy(timestamped_image, default_image).unwrap();
    }
    let stats_dir = output_dir.join(format!("stats{}", tag));
    std::fs::create_dir_all(stats_dir.clone()).unwrap();
    let stats_file = stats_dir.join(format!("stats{}_{}.txt", tag, time_stamp));
    // TODO: add config to stats
    stats::print_and_save(&stats_file);
}

fn online_render() {
    let mut config = RenderConfig::bdpt();
    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new()
        .with_dimensions(config.dimensions())
        .with_resizable(false); // TODO: enable resizing
    let context = glium::glutin::ContextBuilder::new().with_depth_buffer(24);
    let display =
        glium::Display::new(window, context, &events_loop).expect("Failed to create display");

    let (mut scene, mut gpu_scene, mut camera) =
        load::gpu_scene_from_key(&display, VirtualKeyCode::Key1, &config).unwrap();
    let gl_renderer = GLRenderer::new(&display);
    let mut pt_renderer: Option<PTRenderer> = None;

    let mut input = InputState::new();
    let mut quit = false;
    let mut last_frame = Instant::now();

    loop {
        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
        if let Some(renderer) = &mut pt_renderer {
            renderer.update_image();
            renderer.render_image(&display, &mut target);
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
                            if let Some(res) = load::gpu_scene_from_key(&display, keycode, &config)
                            {
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
                Event::WindowEvent {
                    event: WindowEvent::DroppedFile(path),
                    ..
                } => {
                    if pt_renderer.is_none() {
                        // TODO: don't crash on bad scenes
                        if let Some(res) = load::gpu_scene_from_path(&display, &path, &config) {
                            scene = res.0;
                            gpu_scene = res.1;
                            camera = res.2;
                            // TODO: would be nice if this grabbed the focus
                        }
                    }
                }
                _ => (),
            }
        });
        if pt_renderer.is_none() {
            camera.process_input(&input);
        }
        input.reset_deltas();
        if quit {
            return;
        }
        // Limit frame rate
        let frame_time = Duration::from_millis(5);
        let elapsed = last_frame.elapsed();
        if elapsed < frame_time {
            std::thread::park_timeout(frame_time - elapsed);
        }
        last_frame = Instant::now();
    }
}
