use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use cgmath::Vector3;

use glium::backend::Facade;
use glium::glutin::VirtualKeyCode;

use crate::camera::Camera;
use crate::scene::{GPUScene, Scene, SceneBuilder};
use crate::stats;

lazy_static::lazy_static! {
    static ref SCENE_LIBRARY: SceneLibrary = {
        let root_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let scene_dir = root_path.join("scenes");
        let mut lib = SceneLibrary::new();
        lib.add_scene("plane".to_string(), scene_dir.join("plane.obj"),
                      CameraPos::Offset, Some(VirtualKeyCode::Key1));
        lib.add_scene("chesterfield".to_string(),
                      scene_dir.join("cornell").join("cornell_chesterfield.obj"),
                      CameraPos::Center, Some(VirtualKeyCode::Key2));
        lib.add_scene("cornell".to_string(),
                      scene_dir.join("cornell-box").join("CornellBox-Original.obj"),
                      CameraPos::Offset, Some(VirtualKeyCode::Key3));
        lib.add_scene("cornell-glossy".to_string(),
                      scene_dir.join("cornell-box").join("CornellBox-Glossy.obj"),
                      CameraPos::Offset, Some(VirtualKeyCode::Key4));
        lib.add_scene("cornell-water".to_string(),
                      scene_dir.join("cornell-box").join("CornellBox-Water.obj"),
                      CameraPos::Offset, Some(VirtualKeyCode::Key5));
        lib.add_scene("indirect".to_string(),
                      scene_dir.join("indirect-test").join("indirect-test_tex.obj"),
                      CameraPos::Center, Some(VirtualKeyCode::Key6));
        lib.add_scene("conference".to_string(),
                      scene_dir.join("conference-new").join("conference.obj"),
                      CameraPos::Center, Some(VirtualKeyCode::Key7));
        lib.add_scene("nanosuit".to_string(),
                      scene_dir.join("nanosuit").join("nanosuit.obj"),
                      CameraPos::Offset, Some(VirtualKeyCode::Key8));
        lib.add_scene("sibenik".to_string(),
                      scene_dir.join("sibenik").join("sibenik.obj"),
                      CameraPos::Center, Some(VirtualKeyCode::Key9));
        lib.add_scene("sponza".to_string(),
                      scene_dir.join("crytek-sponza").join("sponza.obj"),
                      CameraPos::Center, Some(VirtualKeyCode::Key0));
        lib
    };
}

enum CameraPos {
    Center,
    Offset,
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

fn initialize_camera(scene: &Scene, info: &SceneInfo) -> Camera {
    let mut camera = match info.camera_pos {
        CameraPos::Center => Camera::new(scene.center(), Vector3::new(0.0, 0.0, -1.0)),
        CameraPos::Offset => Camera::new(
            scene.center() + scene.size() * Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(0.0, 0.0, -1.0),
        ),
    };
    camera.set_scale(scene.size());
    camera.update_viewport((600, 400));
    camera
}

pub fn load_cpu_scene(name: &str) -> (Arc<Scene>, Camera) {
    let _t = stats::time("Load");
    let info = SCENE_LIBRARY.get(name).unwrap();
    let scene = SceneBuilder::new().build(&info.path);
    let camera = initialize_camera(&scene, &info);
    (scene, camera)
}

pub fn load_gpu_scene<F: Facade>(
    key: VirtualKeyCode,
    facade: &F,
) -> Option<(Arc<Scene>, GPUScene, Camera)> {
    let name = SCENE_LIBRARY.key_to_name(key)?;
    stats::new_scene(name);
    let (scene, camera) = load_cpu_scene(name);
    let gpu_scene = scene.upload_data(facade);
    println!("Loaded scene {}", name);
    Some((scene, gpu_scene, camera))
}
