use glium::backend::Facade;
use glium::index::PrimitiveType;
use glium::IndexBuffer;

/// Mesh with a common material for CPU rendering
pub struct Mesh {
    pub indices: Vec<u32>,
    pub material_i: usize,
}

/// Mesh for GPU rendering
pub struct GPUMesh {
    pub material_i: usize,
    pub index_buffer: IndexBuffer<u32>,
}

impl Mesh {
    pub fn new(material_i: usize) -> Mesh {
        Mesh {
            indices: Vec::new(),
            material_i,
        }
    }

    /// Load the index buffer to the GPU
    pub fn upload_data<F: Facade>(&self, facade: &F) -> GPUMesh {
        let index_buffer = IndexBuffer::new(facade, PrimitiveType::TrianglesList, &self.indices)
            .expect("Failed to create index buffer!");
        GPUMesh {
            material_i: self.material_i,
            index_buffer,
        }
    }
}
