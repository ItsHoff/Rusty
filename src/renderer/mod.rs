#![cfg_attr(feature="clippy", allow(forget_copy))]

mod gl_renderer;
mod pt_renderer;

pub use self::gl_renderer::GLRenderer;
pub use self::pt_renderer::PTRenderer;

/// Renderer representation of a vertex
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2]
}

implement_vertex!(Vertex, position, normal, tex_coords);

#[derive(Default)]
pub struct RTTriangleBuilder {
    vertex_indices: Vec<usize>,
}

impl RTTriangleBuilder {
    pub fn new() -> RTTriangleBuilder {
        RTTriangleBuilder { ..Default::default() }
    }

    pub fn add_vertex(&mut self, index: usize) {
        self.vertex_indices.push(index);
    }

    pub fn build(self) -> RTTriangle {
        RTTriangle { ..Default::default() }
    }
}

/// Tracable triangle
#[derive(Default, Debug)]
pub struct RTTriangle {
    vert_is: [usize; 3],
}
