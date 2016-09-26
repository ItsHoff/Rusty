mod obj_load;

use std::path::Path;
use std::vec::Vec;


#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3]
}

implement_vertex!(Vertex, position, normal);

pub struct Scene {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>
}

pub fn load_scene(scene_path: &Path) -> Scene {
    let mut scene = Scene { vertices: vec!(), indices: vec!() };
    let obj = obj_load::load_obj(scene_path).expect("Failed to load.");
    for (i, pos) in obj.positions.iter().enumerate() {
        let mut normal = [1.0, 0.0, 0.0];
        if obj.normals.len() != 0 {
            normal = obj.normals[i];
        }
        let vertex = Vertex { position: *pos, normal: normal };
        scene.vertices.push(vertex);
    }
    for mut p in obj.polygons {
        scene.indices.append(&mut p.indices);
    }
    scene
}
