use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use cgmath::Vector3;

use chrono::Local;

use crate::bvh::{BVH, SplitMode};
use crate::camera::Camera;
use crate::pt_renderer::PTRenderer;
use crate::scene::Scene;

// TODO: prettify output https://crates.io/crates/prettytable-rs

fn load_scene(path: &Path) -> (Arc<Scene>, Camera) {
    let scene = Scene::new(path);
    let mut camera = Camera::new(scene.center() + scene.size() * Vector3::new(0.0, 0.0, 1.0f32),
                                 Vector3::new(0.0, 0.0, -1.0f32));
    camera.set_scale(scene.size());
    (Arc::new(scene), camera)
}

fn extract_scene_name(path: &Path) -> &str {
    let file_name = path.file_name().unwrap().to_str().unwrap();
    file_name.split('.').next().unwrap()
}

pub fn benchmark_bvh_build() {
    let root_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let scenes: Vec<PathBuf> =
        [root_path.join("scenes/cornell-box/CornellBox-Water.obj"),
         root_path.join("scenes/nanosuit/nanosuit.obj"),
         root_path.join("scenes/sibenik/sibenik.obj"),
         root_path.join("scenes/conference-new/conference.obj"),
         root_path.join("scenes/crytek-sponza/sponza.obj"),
        ].to_vec();

    let mut combined_info = String::new();
    let save_path = root_path.join("results");
    if !save_path.exists() {
        std::fs::create_dir_all(save_path.clone()).unwrap();
    }
    for scene_path in scenes {
        let scene_name = extract_scene_name(&scene_path);
        let scene = Scene::without_bvh(&scene_path);

        let mut triangles = scene.triangles.clone();
        let build_start = Instant::now();
        // TODO: benchmark other builders?
        let bvh = BVH::build(&mut triangles, SplitMode::SAH);
        let build_duration = build_start.elapsed();

        let mut info_string = String::new();
        writeln!(&mut info_string, "{} built in {:#?} ({} nodes)",
                 scene_name, build_duration, bvh.size()).unwrap();
        println!("{}", info_string);
        combined_info.push_str(&info_string);
    }
    let timing_path = save_path.join(Local::now().format("bvh_%F_%H%M%S.txt").to_string());
    let mut timing_file = File::create(timing_path).unwrap();
    timing_file.write_all(&combined_info.into_bytes()).unwrap();
}

pub fn benchmark_render() {
    let root_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // TODO: set better camera pos for sibenik
    let scenes: Vec<PathBuf> =
        [root_path.join("scenes/plane.obj"),
         root_path.join("scenes/cornell-box/CornellBox-Glossy.obj"),
         root_path.join("scenes/cornell-box/CornellBox-Water.obj"),
         root_path.join("scenes/nanosuit/nanosuit.obj"),
         root_path.join("scenes/sibenik/sibenik.obj"),
        ].to_vec();

    let mut pt_renderer = PTRenderer::new();
    let mut combined_info = String::new();
    let save_path = root_path.join("results");
    if !save_path.exists() {
        std::fs::create_dir_all(save_path.clone()).unwrap();
    }
    for scene_path in scenes {
        let scene_name = extract_scene_name(&scene_path);
        let (scene, mut camera) = load_scene(&scene_path);
        camera.update_viewport((600, 400));

        let render_start = Instant::now();
        pt_renderer.offline_render(&scene, &camera, 2);
        let render_duration = render_start.elapsed();
        let ray_count = pt_renderer.get_ray_count();

        let mut info_string = String::new();
        let float_time = render_duration.as_secs() as f64
            + f64::from(render_duration.subsec_nanos()) / 1_000_000_000.0;
        let ray_speed = ray_count as f64 / float_time;
        writeln!(&mut info_string, "{}:", scene_name).unwrap();
        writeln!(&mut info_string, "    rendered in {:#?}", render_duration).unwrap();
        writeln!(&mut info_string, "    {} total rays", ray_count).unwrap();
        writeln!(&mut info_string, "    {:.0} rays / sec", ray_speed).unwrap();
        println!("{}", info_string);
        combined_info.push_str(&info_string);

        let mut save_file = String::from(scene_name);
        save_file.push_str(".png");
        pt_renderer.save_image(&save_path.join(save_file));
    }
    let timing_path = save_path.join(Local::now().format("render_%F_%H%M%S.txt").to_string());
    let mut timing_file = File::create(timing_path).unwrap();
    timing_file.write_all(&combined_info.into_bytes()).unwrap();
}
