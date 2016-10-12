extern crate image;

mod obj_load;
pub mod camera;
pub use self::camera::Camera;

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::vec::Vec;

use glium::{IndexBuffer, VertexBuffer, Program, Surface, DrawParameters};
use glium::backend::Facade;
use glium::index::PrimitiveType;
use glium::texture::{RawImage2d, Texture2d};

use cgmath::prelude::*;
use cgmath::Matrix4;
use cgmath::conv::*;

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2]
}

implement_vertex!(Vertex, position, normal, tex_coords);

#[derive(Debug)]
pub struct Material {
    pub diffuse: Option<[f32; 3]>,
    pub has_diffuse: bool,
    pub diffuse_texture: Texture2d
}

impl Material {
    fn new<F: Facade>(facade: &F, obj_mat: obj_load::Material) -> Material {
        let (diffuse_texture, has_diffuse)  = match obj_mat.map_Kd {
            Some(tex_path) => {
                let tex_image = Material::load_texture(&tex_path);
                (Texture2d::new(facade, tex_image).expect("Failed to create texture!"), true)
            }
            None => (Texture2d::empty(facade, 0, 0).expect("Failed to create empty texture!"), false)
        };
        Material {
            diffuse: obj_mat.Kd,
            has_diffuse: has_diffuse,
            diffuse_texture: diffuse_texture
        }
    }

    fn load_texture(tex_path: &Path) -> RawImage2d<u8> {
        let tex_reader = BufReader::new(File::open(tex_path).expect("Failed to open texture!"));
        let image = image::load(tex_reader, image::PNG).expect("Failed to load image!").to_rgba();
        let image_dim = image.dimensions();
        RawImage2d::from_raw_rgba_reversed(image.into_raw(), image_dim)
    }
}

#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material: Material,
    pub local_to_world: Matrix4<f32>,
    pub vertex_buffer: Option<VertexBuffer<Vertex>>,
    pub index_buffer: Option<IndexBuffer<u32>>
}

impl Mesh {
    fn new<F: Facade>(facade: &F, obj_mat: obj_load::Material) -> Mesh {
        Mesh { vertices: Vec::new(),
               indices: Vec::new(),
               material: Material::new(facade, obj_mat),
               local_to_world: Matrix4::identity(),
               vertex_buffer: None,
               index_buffer: None
        }
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
            local_to_world: array4x4(self.local_to_world),
            world_to_clip: array4x4(world_to_clip),
            u_light: [-1.0, 0.4, 0.9f32],
            u_color: self.material.diffuse.expect("No diffuse color!"),
            u_has_diffuse: self.material.has_diffuse,
            diffuse_texture: &self.material.diffuse_texture
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
        let obj_mat = obj.materials.get(&range.name)
            .expect(&::std::fmt::format(format_args!("Couldn't find material {}!", range.name)));
        let mut mesh = Mesh::new(facade, obj_mat.clone());
        let mut vertex_map = HashMap::new();
        for polygon in obj.polygons[range.start_i..range.end_i].iter() {
            let default_normal = [0.0; 3];
            let default_tex_coords= [0.0; 2];
            for index_vertex in &polygon.index_vertices {
                match vertex_map.get(index_vertex) {
                    Some(&i) => mesh.indices.push(i),
                    None => {
                        vertex_map.insert(index_vertex, mesh.vertices.len() as u32);
                        let pos = obj.positions[index_vertex[0] - 1];

                        let tex_coords;
                        let tex_coords_i = index_vertex[1];
                        if tex_coords_i > 0 {
                            tex_coords = obj.tex_coords[tex_coords_i - 1];
                        } else {
                            tex_coords = default_tex_coords;
                        }

                        let normal;
                        let normal_i = index_vertex[2];
                        if normal_i > 0 {
                            normal = obj.normals[normal_i - 1];
                        } else {
                            normal = default_normal;
                        }

                        mesh.indices.push(mesh.vertices.len() as u32);
                        mesh.vertices.push(Vertex { position: pos, normal: normal, tex_coords: tex_coords });
                    }
                }
            }
        }
        mesh.create_buffers(facade);
        scene.meshes.push(mesh);
    }
    scene
}
