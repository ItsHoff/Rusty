use cgmath::{Point2, Point3, Vector3};
use glium::implement_vertex;

use crate::float::*;

/// Vertex using raw arrays that can be inserted in vertex buffers
#[derive(Copy, Clone, Debug, Default)]
pub struct RawVertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
}

implement_vertex!(RawVertex, pos, normal, tex_coords);

/// Vertex utilising cgmath types
#[derive(Clone, Debug)]
pub struct Vertex {
    pub p: Point3<Float>,
    pub n: Vector3<Float>,
    pub t: Point2<Float>,
}

impl Vertex {
    pub fn new(pos: [f32; 3], normal: [f32; 3], tex_coords: [f32; 2]) -> Self {
        Self {
            p: Point3::from_array(pos),
            n: Vector3::from_array(normal),
            t: Point2::from_array(tex_coords),
        }
    }
}

impl From<&Vertex> for RawVertex {
    fn from(v: &Vertex) -> Self {
        Self {
            pos: v.p.into_array(),
            normal: v.n.into_array(),
            tex_coords: v.t.into_array(),
        }
    }
}
