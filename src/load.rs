use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use cgmath::prelude::*;
use cgmath::{Point3, Quaternion, Vector3};

use glium::backend::Facade;
use glium::glutin::VirtualKeyCode;

use crate::camera::Camera;
use crate::config::RenderConfig;
use crate::float::*;
use crate::scene::{GPUScene, Scene, SceneBuilder};
use crate::stats;
use crate::util;

lazy_static::lazy_static! {
    static ref SCENE_LIBRARY: SceneLibrary = {
        let root_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let scene_dir = root_path.join("scenes");
        let mut lib = SceneLibrary::new();
        lib.add_scene("plane".to_string(), scene_dir.join("plane.obj"),
                      CameraPos::Offset, Some(VirtualKeyCode::Key1));
        lib.add_scene("chesterfield".to_string(),
                      scene_dir.join("cornell").join("cornell_chesterfield.obj"),
                      CameraPos::Defined(Point3::new(-0.74, 0.4, 0.97),
                                         Quaternion::new(0.95, -0.15, -0.28, -0.04)),
                      Some(VirtualKeyCode::Key2));
        lib.add_scene("cornell-sphere".to_string(),
                      scene_dir.join("cornell-box").join("CornellBox-Sphere.obj"),
                      CameraPos::Offset, Some(VirtualKeyCode::Key3));
        lib.add_scene("cornell-glossy".to_string(),
                      scene_dir.join("cornell-box").join("CornellBox-Glossy.obj"),
                      CameraPos::Offset, Some(VirtualKeyCode::Key4));
        lib.add_scene("cornell-water".to_string(),
                      scene_dir.join("cornell-box").join("CornellBox-Water.obj"),
                      CameraPos::Offset, Some(VirtualKeyCode::Key5));
        lib.add_scene("indirect".to_string(),
                      scene_dir.join("indirect-test").join("indirect-test_tex.obj"),
                      CameraPos::Defined(Point3::new(0.43, 0.45, 0.8),
                                         Quaternion::new(0.98, -0.01, 0.18, 0.0)),
                      Some(VirtualKeyCode::Key6));
        lib.add_scene("conference".to_string(),
                      scene_dir.join("conference-new").join("conference.obj"),
                      CameraPos::Defined(Point3::new(-0.84, 0.06, 0.4),
                                         Quaternion::new(0.84, -0.06, -0.54, -0.04)),
                      Some(VirtualKeyCode::Key7));
        lib.add_scene("nanosuit".to_string(),
                      scene_dir.join("nanosuit").join("nanosuit.obj"),
                      CameraPos::Offset, Some(VirtualKeyCode::Key8));
        lib.add_scene("sibenik".to_string(),
                      scene_dir.join("sibenik").join("sibenik.obj"),
                      CameraPos::Defined(Point3::new(-10.7, -7.85, 0.11),
                                         Quaternion::new(0.73, -0.06, -0.68, -0.06)),
                      Some(VirtualKeyCode::Key9));
        lib.add_scene("sponza".to_string(),
                      scene_dir.join("crytek-sponza").join("sponza.obj"),
                      CameraPos::Defined(Point3::new(-783.01, 184.23, 173.92),
                                         Quaternion::new(0.89, -0.06, 0.44, 0.03)),
                      Some(VirtualKeyCode::Key0));
        lib.add_scene("sponza-bump".to_string(),
                      scene_dir.join("sponza_bump").join("sponza.obj"),
                      CameraPos::Defined(Point3::new(-783.01, 184.23, 173.92),
                                         Quaternion::new(0.89, -0.06, 0.44, 0.03)),
                      Some(VirtualKeyCode::Minus));
        lib
    };
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
enum CameraPos {
    Center,
    Offset,
    Defined(Point3<Float>, Quaternion<Float>),
}

struct SceneInfo {
    path: PathBuf,
    camera_pos: CameraPos,
}

struct SceneLibrary {
    scene_map: HashMap<String, SceneInfo>,
    key_map: HashMap<VirtualKeyCode, String>,
}

impl SceneLibrary {
    fn new() -> SceneLibrary {
        SceneLibrary {
            scene_map: HashMap::new(),
            key_map: HashMap::new(),
        }
    }

    fn add_scene(
        &mut self,
        name: String,
        path: PathBuf,
        camera_pos: CameraPos,
        key: Option<VirtualKeyCode>,
    ) {
        if let Some(code) = key {
            self.key_map.insert(code, name.clone());
        }
        let info = SceneInfo { path, camera_pos };
        self.scene_map.insert(name, info);
    }

    pub fn get(&self, name: &str) -> Option<&SceneInfo> {
        self.scene_map.get(name)
    }

    pub fn key_to_name(&self, key: VirtualKeyCode) -> Option<&String> {
        self.key_map.get(&key)
    }
}

fn initialize_camera(scene: &Scene, pos: CameraPos, config: &RenderConfig) -> Camera {
    let mut camera = match pos {
        CameraPos::Center => Camera::new(scene.center(), Quaternion::one()),
        CameraPos::Offset => Camera::new(
            scene.center() + scene.size() * Vector3::new(0.0, 0.0, 1.0),
            Quaternion::one(),
        ),
        // Normalize the rotation because its magnitude is probably slightly off
        CameraPos::Defined(pos, rot) => Camera::new(pos, rot.normalize()),
    };
    camera.set_scale(scene.size());
    camera.update_viewport(config.dimensions());
    camera
}

fn cpu_scene(path: &Path, camera_pos: CameraPos, config: &RenderConfig) -> (Arc<Scene>, Camera) {
    let scene = SceneBuilder::new(config).build(path);
    let camera = initialize_camera(&scene, camera_pos, config);
    (scene, camera)
}

fn gpu_scene<F: Facade>(
    facade: &F,
    path: &Path,
    camera_pos: CameraPos,
    config: &RenderConfig,
) -> (Arc<Scene>, GPUScene, Camera) {
    let (scene, camera) = cpu_scene(path, camera_pos, config);
    let gpu_scene = scene.upload_data(facade);
    (scene, gpu_scene, camera)
}

pub fn cpu_scene_from_name(name: &str, config: &RenderConfig) -> (Arc<Scene>, Camera) {
    let _t = stats::time("Load");
    let info = SCENE_LIBRARY.get(name).unwrap();
    cpu_scene(&info.path, info.camera_pos, config)
}

pub fn gpu_scene_from_path<F: Facade>(
    facade: &F,
    path: &Path,
    config: &RenderConfig,
) -> Option<(Arc<Scene>, GPUScene, Camera)> {
    if let Some("obj") = util::lowercase_extension(path).as_ref().map(|s| s.as_str()) {
        stats::new_scene(path.to_str().unwrap());
        let res = gpu_scene(facade, path, CameraPos::Offset, config);
        println!("Loaded scene from {:?}", path);
        Some(res)
    } else {
        println!("{:?} is not object file (.obj)", path);
        None
    }
}

pub fn gpu_scene_from_key<F: Facade>(
    facade: &F,
    key: VirtualKeyCode,
    config: &RenderConfig,
) -> Option<(Arc<Scene>, GPUScene, Camera)> {
    let name = SCENE_LIBRARY.key_to_name(key)?;
    stats::new_scene(name);
    let info = SCENE_LIBRARY.get(name).unwrap();
    let res = gpu_scene(facade, &info.path, info.camera_pos, config);
    println!("Loaded scene {}", name);
    Some(res)
}
