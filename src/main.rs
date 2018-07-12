#![feature(rust_2018_preview)]
#![feature(rust_2018_idioms)]
#![feature(nll)]
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
mod triangle;
mod vertex;

use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use cgmath::Vector3;

use chrono::Local;

use glium::Surface;
use glium::backend::Facade;
use glium::glutin::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode};

use crate::camera::Camera;
use crate::gl_renderer::GLRenderer;
use crate::input::InputState;
use crate::pt_renderer::PTRenderer;
use crate::scene::{Scene, GPUScene};

fn load_offline_scene(path: &Path) -> (Arc<Scene>, Camera) {
    let scene = Scene::new(path);
    let mut camera = Camera::new(scene.center() + scene.size() * Vector3::new(0.0, 0.0, 1.0f32),
                                 Vector3::new(0.0, 0.0, -1.0f32));
    camera.set_scale(scene.size());
    (Arc::new(scene), camera)
}

fn load_scene<F: Facade>(path: &Path, facade: &F) -> (Arc<Scene>, GPUScene, Camera) {
    let (scene, camera) = load_offline_scene(path);
    let gpu_scene = scene.upload_data(facade);
    (scene, gpu_scene, camera)
}

fn main() {
    if std::env::args().count() > 1 {
        offline_render();
    } else {
        online_render();
    }
}

fn offline_render() {
    let root_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // TODO: set better camera pos for sibenik
    let scenes: Vec<PathBuf> =
        [ root_path.join("scenes/plane.obj"),
          root_path.join("scenes/cornell-box/CornellBox-Glossy.obj"),
          root_path.join("scenes/cornell-box/CornellBox-Water.obj"),
          root_path.join("scenes/nanosuit/nanosuit.obj"),
          root_path.join("scenes/sibenik/sibenik.obj"),
        ].to_vec();

    let mut pt_renderer = PTRenderer::new();
    let save_path = root_path.join("results");
    if !save_path.exists() {
        std::fs::create_dir_all(save_path.clone()).unwrap();
    }
    let mut combined_info = String::new();
    for scene_path in scenes {
        let file_name = scene_path.file_name().unwrap().to_str().unwrap();
        let scene_name = file_name.split('.').next().unwrap();

        let load_start = Instant::now();
        let (scene, mut camera) = load_offline_scene(&scene_path);
        let load_duration = load_start.elapsed();

        camera.update_viewport((600, 400));
        let render_start = Instant::now();
        pt_renderer.offline_render(&scene, &camera, 2);
        let render_duration = render_start.elapsed();
        let ray_count = pt_renderer.get_ray_count();

        let mut info_string = String::new();
        let float_time = render_duration.as_secs() as f64
            + f64::from(render_duration.subsec_nanos()) / 1_000_000_000.0;
        let ray_speed = ray_count as f64 / float_time;
        write!(&mut info_string, "\n{}:\n", scene_name).unwrap();
        write!(&mut info_string, "    loaded in {:#?}\n", load_duration).unwrap();
        write!(&mut info_string, "    rendered in {:#?}\n", render_duration).unwrap();
        write!(&mut info_string, "    {} total rays\n", ray_count).unwrap();
        write!(&mut info_string, "    {:.0} rays / sec\n", ray_speed).unwrap();
        println!("{}", info_string);
        combined_info.push_str(&info_string);

        let mut save_file = String::from(scene_name);
        save_file.push_str(".png");
        pt_renderer.save_image(&save_path.join(save_file));
    }
    let timing_path = save_path.join(Local::now().format("timing_%F_%H%M%S.txt").to_string());
    let mut timing_file = File::create(timing_path).unwrap();
    timing_file.write_all(&combined_info.into_bytes()).unwrap();
}

fn online_render() {
    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new();
    let context = glium::glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(window, context, &events_loop).expect("Failed to create display");

    let root_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // TODO: Enable use of arbitrary scene
    let scenes: HashMap<VirtualKeyCode, PathBuf> =
        [(VirtualKeyCode::Key1, root_path.join("scenes/plane.obj")),
         (VirtualKeyCode::Key2, root_path.join("scenes/cornell/cornell_chesterfield.obj")),
         (VirtualKeyCode::Key3, root_path.join("scenes/cornell-box/CornellBox-Original.obj")),
         (VirtualKeyCode::Key4, root_path.join("scenes/cornell-box/CornellBox-Glossy.obj")),
         (VirtualKeyCode::Key5, root_path.join("scenes/cornell-box/CornellBox-Water.obj")),
         (VirtualKeyCode::Key6, root_path.join("scenes/indirect-test/indirect-test_tex.obj")),
         (VirtualKeyCode::Key7, root_path.join("scenes/conference-new/conference.obj")),
         (VirtualKeyCode::Key8, root_path.join("scenes/nanosuit/nanosuit.obj")),
         (VirtualKeyCode::Key9, root_path.join("scenes/sibenik/sibenik.obj")),
         (VirtualKeyCode::Key0, root_path.join("scenes/crytek-sponza/sponza.obj")),
        ]
        .iter().cloned().collect();

    let (mut scene, mut gpu_scene, mut camera) =
        load_scene(&scenes[&VirtualKeyCode::Key1], &display);
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
