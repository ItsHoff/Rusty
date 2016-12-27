extern crate image;

mod obj_load;

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::vec::Vec;

use glium::{IndexBuffer, VertexBuffer, Program, Surface, DrawParameters};
use glium::backend::Facade;
use glium::index::PrimitiveType;
use glium::texture::{RawImage2d, SrgbTexture2d};

use cgmath::prelude::*;
use cgmath::Matrix4;
use cgmath::conv::*;

/// Renderer representation of a vertex
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2]
}

implement_vertex!(Vertex, position, normal, tex_coords);

/// Renderer representation of a material
pub struct Material {
    pub diffuse: [f32; 3],
    pub diffuse_image: Option<image::RgbaImage>,
    pub diffuse_texture: Option<SrgbTexture2d>
}

impl Material {
    /// Create a new material based on a material loaded from the scene file
    fn new(obj_mat: obj_load::Material) -> Material {
        // Create diffuse texture and load it to the GPU
        let diffuse_image = match obj_mat.tex_diffuse {
            Some(tex_path) => Some(Material::load_image(&tex_path)),
            None => None
        };
        Material {
            diffuse: obj_mat.c_diffuse.expect("No diffuse color!"),
            diffuse_image: diffuse_image,
            diffuse_texture: None
        }
    }

    /// Load an image at
    fn load_image(path: &Path) -> image::RgbaImage {
        let tex_reader = BufReader::new(File::open(path).expect("Failed to open image!"));
        image::load(tex_reader, image::PNG).expect("Failed to load image!").to_rgba()
    }

    fn upload_textures<F: Facade>(&mut self, facade: &F) {
        self.diffuse_texture = match self.diffuse_image {
            Some(ref image) => {
                let image_dim = image.dimensions();
                let tex_image = RawImage2d::from_raw_rgba_reversed(image.clone().into_raw(), image_dim);
                Some(SrgbTexture2d::new(facade, tex_image).expect("Failed to upload texture!"))
            }
            None => Some(SrgbTexture2d::empty(facade, 0, 0).expect("Failed to upload empty texture!"))
        }
    }
}

/// Renderer representation of mesh with a common material
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material: Material,
    pub local_to_world: Matrix4<f32>,
    pub vertex_buffer: Option<VertexBuffer<Vertex>>,
    pub index_buffer: Option<IndexBuffer<u32>>
}

impl Mesh {
    fn new(obj_mat: obj_load::Material) -> Mesh {
        Mesh { vertices: Vec::new(),
               indices: Vec::new(),
               material: Material::new(obj_mat),
               local_to_world: Matrix4::identity(),
               vertex_buffer: None,
               index_buffer: None
        }
    }

    /// Load the textures + vertex and index buffers to the GPU
    fn upload_data<F: Facade>(&mut self, facade: &F) {
        self.material.upload_textures(facade);
        self.vertex_buffer = Some(VertexBuffer::new(facade, &self.vertices)
                                  .expect("Failed to create vertex buffer!"));
        self.index_buffer = Some(IndexBuffer::new(facade, PrimitiveType::TrianglesList, &self.indices)
                                 .expect("Failed to create index buffer!"));
    }

    /// Draw this mesh to the target
    pub fn draw<S: Surface>(&self, target: &mut S, program: &Program, draw_parameters: &DrawParameters,
                            world_to_clip: Matrix4<f32>) {
        let uniforms = uniform! {
            local_to_world: array4x4(self.local_to_world),
            world_to_clip: array4x4(world_to_clip),
            u_light: [-1.0, 0.4, 0.9f32],
            u_color: self.material.diffuse,
            u_has_diffuse: self.material.diffuse_image.is_some(),
            tex_diffuse: self.material.diffuse_texture.as_ref().expect("Use of unloaded texture!")
        };
        target.draw(self.vertex_buffer.as_ref().expect("No vertex buffer!"),
                    self.index_buffer.as_ref().expect("No index buffer!"),
                    &program, &uniforms, &draw_parameters).unwrap();
    }
}

/// Renderer representation of a scene
#[derive(Default)]
pub struct Scene {
    pub meshes: Vec<Mesh>,
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl Scene {
    pub fn get_center(&self) -> [f32; 3] {
        let mut res = [0.0f32; 3];
        for i in 0..2 {
            res[i] = (self.min[i] + self.max[i]) / 2.0;
        }
        res
    }

    pub fn get_size(&self) -> f32 {
        let mut max = 0.0f32;
        for i in 0..2 {
            max = max.max(self.max[i] - self.min[i]);
        }
        max
    }

    fn update_ranges(&mut self, new_pos: [f32; 3]) {
        for i in 0..2 {
            self.min[i] = self.min[i].min(new_pos[i]);
        }
        for i in 0..2 {
            self.max[i] = self.max[i].max(new_pos[i]);
        }
    }

}

/// Load a scene from the given path bind resources to given facade
pub fn load_scene<F: Facade>(scene_path: &Path, facade: &F) -> Scene {
    let mut scene = Scene { .. Default::default() };
    let obj = obj_load::load_obj(scene_path).expect("Failed to load.");

    // Closure to calculate planar normal for a polygon
    let calculate_normal = |polygon: &obj_load::Polygon| -> [f32; 3] {
        let pos_i1 = polygon.index_vertices[0][0].expect("No vertex positions!");
        let pos_i2 = polygon.index_vertices[1][0].expect("No vertex positions!");
        let pos_i3 = polygon.index_vertices[2][0].expect("No vertex positions!");
        let pos_1 = obj.positions[pos_i1];
        let pos_2 = obj.positions[pos_i2];
        let pos_3 = obj.positions[pos_i3];
        let u = [pos_2[0] - pos_1[0],
                 pos_2[1] - pos_1[1],
                 pos_2[2] - pos_1[2]];
        let v = [pos_3[0] - pos_1[0],
                 pos_3[1] - pos_1[1],
                 pos_3[2] - pos_1[2]];
        [u[1]*v[2] - u[2]*v[1],
         u[2]*v[0] - u[0]*v[2],
         u[0]*v[1] - u[1]*v[0]]
    };

    // Group the polygons by materials for easy rendering
    for range in &obj.material_ranges {
        let obj_mat = obj.materials.get(&range.name)
            .expect(&::std::fmt::format(format_args!("Couldn't find material {}!", range.name)));
        let mut mesh = Mesh::new(obj_mat.clone());
        let mut vertex_map = HashMap::new();
        for tri in &obj.polygons[range.start_i..range.end_i] {
            let default_tex_coords= [0.0; 2];
            for index_vertex in &tri.index_vertices {
                match vertex_map.get(index_vertex) {
                    // Vertex has already been added
                    Some(&i) => mesh.indices.push(i),
                    None => {
                        // Add vertex to map
                        vertex_map.insert(index_vertex, mesh.vertices.len() as u32);
                        // Panic if there is no positions
                        let pos_i = index_vertex[0].expect("No vertex positions!");
                        let pos = obj.positions[pos_i];
                        scene.update_ranges(pos);

                        let tex_coords = match index_vertex[1] {
                            Some(tex_coords_i) => obj.tex_coords[tex_coords_i],
                            None => default_tex_coords
                        };
                        let normal = match index_vertex[2] {
                            Some(normal_i) => obj.normals[normal_i],
                            None => calculate_normal(tri)
                        };

                        mesh.indices.push(mesh.vertices.len() as u32);
                        mesh.vertices.push(Vertex { position: pos, normal: normal, tex_coords: tex_coords });
                    }
                }
            }
        }
        if !mesh.vertices.is_empty() {
            mesh.upload_data(facade);
            scene.meshes.push(mesh);
        }
    }
    scene
}
