mod obj_load;

use std::collections::HashMap;
use std::path::Path;
use std::vec::Vec;

use self::obj_load::Material;

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3]
}

implement_vertex!(Vertex, position, normal);

#[derive(Default, Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material: Material
}

impl Mesh {
    fn new(material: Material) -> Mesh {
        Mesh { material: material, ..Default::default() }
    }
}

pub struct Scene {
    pub meshes: Vec<Mesh>
}

pub fn load_scene(scene_path: &Path) -> Scene {
    let mut scene = Scene { meshes: vec!() };
    let obj = obj_load::load_obj(scene_path).expect("Failed to load.");
    for range in obj.material_ranges {
        let material = obj.materials.get(&range.name)
            .expect(&::std::fmt::format(format_args!("Couldn't find material {}!", range.name)));
        let mut mesh = Mesh::new(material.clone());
        let mut vertex_map = HashMap::new();
        for polygon in obj.polygons[range.start_i..range.end_i].iter() {
            let planar_normal = [0.0; 3];
            for index_vertex in &polygon.index_vertices {
                match vertex_map.get(index_vertex) {
                    Some(&i) => mesh.indices.push(i),
                    None => {
                        vertex_map.insert(index_vertex, mesh.vertices.len() as u32);
                        let pos = obj.positions[index_vertex[0] - 1];
                        let normal;
                        let normal_i = index_vertex[1];
                        if normal_i > 0 {
                            normal = obj.normals[normal_i - 1];
                        } else {
                            normal = planar_normal;
                        }
                        mesh.indices.push(mesh.vertices.len() as u32);
                        mesh.vertices.push(Vertex { position: pos, normal: normal });
                    }
                }
            }
        }
        scene.meshes.push(mesh);
    }
    scene
}
