#![cfg_attr(feature="clippy", allow(forget_copy))]

mod gl_renderer;
mod pt_renderer;

use cgmath::{Vector3, Point3, Point2};

pub use self::gl_renderer::GLRenderer;
pub use self::pt_renderer::PTRenderer;

/// Vertex using raw arrays that can be inserted in vertex buffers
#[derive(Copy, Clone, Debug, Default)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, normal, tex_coords);

/// Vertex utilising cgmath types
#[derive(Debug)]
struct CGVertex {
    position: Point3<f32>,
    normal: Vector3<f32>,
    tex_coords: Point2<f32>,
}

impl From<Vertex> for CGVertex {
    fn from(v: Vertex) -> CGVertex {
        CGVertex {
            position: Point3::from(v.position),
            normal: Vector3::from(v.normal),
            tex_coords: Point2::from(v.tex_coords),
        }
    }
}

#[derive(Default)]
pub struct RTTriangleBuilder {
    vertices: Vec<Vertex>,
}

impl RTTriangleBuilder {
    pub fn new() -> RTTriangleBuilder {
        RTTriangleBuilder { ..Default::default() }
    }

    pub fn add_vertex(&mut self, vertex: Vertex) {
        self.vertices.push(vertex);
    }

    pub fn build(self) -> Result<RTTriangle, String> {
        if self.vertices.len() != 3 {
            Err("Triangle doesn't have 3 vertices!".to_owned())
        } else {
            let vertices = [CGVertex::from(self.vertices[0]),
                            CGVertex::from(self.vertices[1]),
                            CGVertex::from(self.vertices[2])];
            Ok(RTTriangle { vertices: vertices })
        }
    }
}

/// Tracable triangle
#[derive(Debug)]
pub struct RTTriangle {
    vertices: [CGVertex; 3],
}
