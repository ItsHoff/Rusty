use glium::{DrawParameters, IndexBuffer, VertexBuffer, Program, Surface};
use glium::backend::Facade;
use glium::index::PrimitiveType;

use cgmath::prelude::*;
use cgmath::Matrix4;
use cgmath::conv::*;

use scene::obj_load;
use scene::Vertex;
use scene::material::Material;

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
    pub fn new(obj_mat: &obj_load::Material) -> Mesh {
        Mesh { vertices: Vec::new(),
               indices: Vec::new(),
               material: Material::new(obj_mat),
               local_to_world: Matrix4::identity(),
               vertex_buffer: None,
               index_buffer: None
        }
    }

    /// Load the textures + vertex and index buffers to the GPU
    pub fn upload_data<F: Facade>(&mut self, facade: &F) {
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
