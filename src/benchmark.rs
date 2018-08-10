use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Instant;

use chrono::Local;

use prettytable::{Table, cell, row};

use crate::bvh::{BVH, SplitMode};
use crate::pt_renderer::PTRenderer;
use crate::scene::Scene;
use crate::util;

fn extract_scene_name(path: &Path) -> &str {
    let file_name = path.file_name().unwrap().to_str().unwrap();
    file_name.split('.').next().unwrap()
}

pub fn benchmark_all() {
    println!("\nBVH benchmark:");
    benchmark_bvh_build();
    println!("\nRender benchmark:");
    benchmark_render();
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

    let mut info_table = Table::new();
    info_table.add_row(row!["scene", "triangles", "nodes", "time"]);
    for scene_path in scenes {
        let scene_name = extract_scene_name(&scene_path);
        println!("{}...", scene_name);
        let scene = Scene::without_bvh(&scene_path);

        let mut triangles = scene.triangles.clone();
        let build_start = Instant::now();
        // TODO: benchmark other builders?
        let bvh = BVH::build(&mut triangles, SplitMode::SAH);
        let build_duration = build_start.elapsed();

        info_table.add_row(row![scene_name, triangles.len(), bvh.size(),
                                format!("{:#.2?}", build_duration)]);
    }
    let save_path = root_path.join("results");
    if !save_path.exists() {
        std::fs::create_dir_all(save_path.clone()).unwrap();
    }
    let timing_path = save_path.join(Local::now().format("bvh_%F_%H%M%S.txt").to_string());
    let mut timing_file = File::create(timing_path).unwrap();
    info_table.printstd();
    info_table.print(&mut timing_file).unwrap();
}

pub fn benchmark_render() {
    let root_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let scenes = ["plane", "cornell-glossy", "cornell-water", "nanosuit", "sibenik",];

    let mut pt_renderer = PTRenderer::new();
    let mut info_table = Table::new();
    info_table.add_row(row!["scene", "rays", "time", "Mrays/s"]);
    let save_path = root_path.join("results");
    if !save_path.exists() {
        std::fs::create_dir_all(save_path.clone()).unwrap();
    }
    for scene_name in &scenes {
        println!("{}...", scene_name);
        let (scene, mut camera) = util::load_benchmark_scene(scene_name);
        camera.update_viewport((600, 400));

        let render_start = Instant::now();
        pt_renderer.offline_render(&scene, &camera, 2);
        let render_duration = render_start.elapsed();
        let ray_count = pt_renderer.get_ray_count();

        let float_time = render_duration.as_secs() as f64
            + f64::from(render_duration.subsec_nanos()) / 1_000_000_000.0;
        let mrps = ray_count as f64 / float_time / 1_000_000.0;
        info_table.add_row(row!(scene_name, ray_count,
                                format!("{:#.2?}", render_duration),
                                format!("{:.2}", mrps)));

        let mut save_file = scene_name.to_string();
        save_file.push_str(".png");
        pt_renderer.save_image(&save_path.join(save_file));
    }
    let timing_path = save_path.join(Local::now().format("render_%F_%H%M%S.txt").to_string());
    let mut timing_file = File::create(timing_path).unwrap();
    info_table.printstd();
    info_table.print(&mut timing_file).unwrap();
}
