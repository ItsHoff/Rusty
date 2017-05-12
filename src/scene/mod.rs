extern crate image;

mod material;
mod mesh;
mod obj_load;

use std::collections::HashMap;
use std::path::Path;
use std::vec::Vec;

use glium::{DrawParameters, VertexBuffer, Program, Surface};
use glium::backend::Facade;

use cgmath::Matrix4;
use cgmath::conv::*;

use self::mesh::Mesh;
use self::material::Material;

/// Renderer representation of a vertex
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2]
}

implement_vertex!(Vertex, position, normal, tex_coords);

/// Renderer representation of a scene
#[derive(Default)]
pub struct Scene {
    pub vertices: Vec<Vertex>,
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub vertex_buffer: Option<VertexBuffer<Vertex>>,
    /// Bounding box of the scene
    pub min: [f32; 3],
    pub max: [f32; 3],
}

#[cfg_attr(feature="clippy", allow(needless_range_loop))]
impl Scene {
    /// Load the textures + vertex and index buffers to the GPU
    pub fn upload_data<F: Facade>(&mut self, facade: &F) {
        self.vertex_buffer = Some(VertexBuffer::new(facade, &self.vertices)
                                  .expect("Failed to create vertex buffer!"));
        for mesh in &mut self.meshes {
            mesh.upload_data(facade);
        }
        for material in &mut self.materials {
            material.upload_textures(facade);
        }
    }

    pub fn draw<S: Surface>(&self, target: &mut S, program: &Program, draw_parameters: &DrawParameters,
                            world_to_clip: Matrix4<f32>) {
        for mesh in &self.meshes {
            let material = &self.materials[mesh.material_i];
            let uniforms = uniform! {
                local_to_world: array4x4(mesh.local_to_world),
                world_to_clip: array4x4(world_to_clip),
                u_light: [-1.0, 0.4, 0.9f32],
                u_color: material.diffuse,
                u_has_diffuse: material.diffuse_image.is_some(),
                tex_diffuse: material.diffuse_texture.as_ref().expect("Use of unloaded texture!")
            };
            target.draw(self.vertex_buffer.as_ref().expect("No vertex buffer"),
                        mesh.index_buffer.as_ref().expect("No index buffer!"),
                        program, &uniforms, draw_parameters).unwrap();
        }
    }

    /// Get the center of the scene as defined by the bounding box
    pub fn get_center(&self) -> [f32; 3] {
        let mut res = [0.0f32; 3];
        for i in 0..2 {
            res[i] = (self.min[i] + self.max[i]) / 2.0;
        }
        res
    }

    /// Get the longest edge of the bounding box
    pub fn get_size(&self) -> f32 {
        let mut max = 0.0f32;
        for i in 0..2 {
            max = max.max(self.max[i] - self.min[i]);
        }
        max
    }

    /// Update the bounding box with new position
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
pub fn load_scene(scene_path: &Path) -> Scene {
    let mut scene = Scene { .. Default::default() };
    let obj = obj_load::load_obj(scene_path).expect("Failed to load.");

    // Closure to calculate planar normal for a triangle
    let calculate_normal = |triangle: &obj_load::Triangle| -> [f32; 3] {
        let pos_i1 = triangle.index_vertices[0].pos_i;
        let pos_i2 = triangle.index_vertices[1].pos_i;
        let pos_i3 = triangle.index_vertices[2].pos_i;
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
    let mut vertex_map = HashMap::new();
    for range in &obj.material_ranges {
        let obj_mat = obj.materials.get(&range.name)
            .expect(&::std::fmt::format(format_args!("Couldn't find material {}!", range.name)));
        let mut mesh = Mesh::new(scene.materials.len());
        scene.materials.push(Material::new(obj_mat));
        for tri in &obj.triangles[range.start_i..range.end_i] {
            let default_tex_coords= [0.0; 2];
            for index_vertex in &tri.index_vertices {
                match vertex_map.get(index_vertex) {
                    // Vertex has already been added
                    Some(&i) => mesh.indices.push(i),
                    None => {
                        // Add vertex to map
                        vertex_map.insert(index_vertex, scene.vertices.len() as u32);
                        let pos = obj.positions[index_vertex.pos_i];
                        scene.update_ranges(pos);

                        let tex_coords = match index_vertex.tex_i {
                            Some(tex_i) => obj.tex_coords[tex_i],
                            None => default_tex_coords
                        };
                        let normal = match index_vertex.normal_i {
                            Some(normal_i) => obj.normals[normal_i],
                            None => calculate_normal(tri)
                        };

                        mesh.indices.push(scene.vertices.len() as u32);
                        scene.vertices.push(Vertex { position: pos, normal: normal, tex_coords: tex_coords });
                    }
                }
            }
        }
        if !mesh.indices.is_empty() {
            scene.meshes.push(mesh);
        }
    }
    scene
}
