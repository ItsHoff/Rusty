use glium::IndexBuffer;
use glium::backend::Facade;
use glium::index::PrimitiveType;

use cgmath::prelude::*;
use cgmath::Matrix4;

/// Renderer representation of mesh with a common material
pub struct Mesh {
    pub indices: Vec<u32>,
    pub material_i: usize,
    pub local_to_world: Matrix4<f32>,
    pub index_buffer: Option<IndexBuffer<u32>>
}

impl Mesh {
    pub fn new(material_i: usize) -> Mesh {
        Mesh { indices: Vec::new(),
               material_i: material_i,
               local_to_world: Matrix4::identity(),
               index_buffer: None
        }
    }

    /// Load the textures + vertex and index buffers to the GPU
    pub fn upload_data<F: Facade>(&mut self, facade: &F) {
        self.index_buffer = Some(IndexBuffer::new(facade, PrimitiveType::TrianglesList, &self.indices)
                                 .expect("Failed to create index buffer!"));
    }
}
