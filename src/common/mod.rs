mod obj_load;

use std::collections::HashMap;
use std::path::Path;
use std::vec::Vec;

use glium::{IndexBuffer, VertexBuffer, Program, Surface, DrawParameters};
use glium::backend::Facade;
use glium::index::PrimitiveType;

use cgmath::Matrix4;
use cgmath::conv::*;

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
    pub material: Material,
    pub vertex_buffer: Option<VertexBuffer<Vertex>>,
    pub index_buffer: Option<IndexBuffer<u32>>
}

impl Mesh {
    fn new(material: Material) -> Mesh {
        Mesh { material: material, ..Default::default() }
    }

    fn create_buffers<F: Facade>(&mut self, facade: &F) {
        self.vertex_buffer = Some(VertexBuffer::new(facade, &self.vertices)
                                  .expect("Failed to create vertex buffer!"));
        self.index_buffer = Some(IndexBuffer::new(facade, PrimitiveType::TrianglesList, &self.indices)
                                 .expect("Failed to create index buffer!"));
    }

    pub fn draw<S: Surface>(&self, target: &mut S, program: &Program, draw_parameters: &DrawParameters,
                            world_to_clip: Matrix4<f32>) {
        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ],
            world_to_clip: array4x4(world_to_clip),
            u_light: [-1.0, 0.4, 0.9f32],
            u_color: self.material.Kd.expect("No diffuse color!")
        };
        target.draw(self.vertex_buffer.as_ref().expect("No vertex buffer!"),
                    self.index_buffer.as_ref().expect("No index buffer!"),
                    &program, &uniforms, &draw_parameters).unwrap();
    }
}

pub struct Scene {
    pub meshes: Vec<Mesh>
}

pub fn load_scene<F: Facade>(scene_path: &Path, facade: &F) -> Scene {
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
        mesh.create_buffers(facade);
        scene.meshes.push(mesh);
    }
    scene
}
