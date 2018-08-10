use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use cgmath::Vector3;

use glium::backend::Facade;
use glium::glutin::VirtualKeyCode;

use crate::camera::Camera;
use crate::scene::{Scene, GPUScene};

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
    Center, Offset
}

struct SceneInfo {
    name: String,
    path: PathBuf,
    camera_pos: CameraPos,
}

struct SceneLibrary {
    scene_map: HashMap<String, SceneInfo>,
    key_map: HashMap<VirtualKeyCode, String>,
}

impl SceneLibrary {
    fn new() -> SceneLibrary {
        SceneLibrary { scene_map: HashMap::new(), key_map: HashMap::new() }
    }

    fn add_scene(&mut self, name: String, path: PathBuf, camera_pos: CameraPos,
                 key: Option<VirtualKeyCode>) {
        if let Some(code) = key {
            self.key_map.insert(code, name.clone());
        }
        let info = SceneInfo { name: name.clone(), path, camera_pos };
        self.scene_map.insert(name, info);
    }

    pub fn get_with_name(&self, name: &str) -> Option<&SceneInfo> {
        self.scene_map.get(name)
    }

    pub fn get_with_key(&self, key: VirtualKeyCode) -> Option<&SceneInfo> {
        let name = self.key_map.get(&key)?;
        self.get_with_name(name)
    }
}

pub fn load_scene<F: Facade>(key: VirtualKeyCode, facade: &F) -> Option<(Arc<Scene>, GPUScene, Camera)> {
    let info = SCENE_LIBRARY.get_with_key(key)?;
    let scene = Scene::new(&info.path);
    let mut camera = match info.camera_pos {
        CameraPos::Center => Camera::new(scene.center(), Vector3::new(0.0, 0.0, -1.0f32)),
        CameraPos::Offset => Camera::new(scene.center() + scene.size() * Vector3::new(0.0, 0.0, 1.0f32),
                                 Vector3::new(0.0, 0.0, -1.0f32)),
    };
    camera.set_scale(scene.size());
    let gpu_scene = scene.upload_data(facade);
    println!("Loaded scene {}", info.name);
    Some((Arc::new(scene), gpu_scene, camera))
}

pub fn load_benchmark_scene(name: &str) -> (Arc<Scene>, Camera) {
    let info = SCENE_LIBRARY.get_with_name(name).unwrap();
    let scene = Scene::new(&info.path);
    let mut camera = match info.camera_pos {
        CameraPos::Center => Camera::new(scene.center(), Vector3::new(0.0, 0.0, -1.0f32)),
        CameraPos::Offset => Camera::new(scene.center() + scene.size() * Vector3::new(0.0, 0.0, 1.0f32),
                                 Vector3::new(0.0, 0.0, -1.0f32)),
    };
    camera.set_scale(scene.size());
    (Arc::new(scene), camera)
}
