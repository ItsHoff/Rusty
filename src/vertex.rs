use cgmath::{Point2, Point3, Vector3};
use glium::implement_vertex;

use crate::Float;

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
    #[allow(clippy::identity_conversion)]
    pub fn new(pos: [f32; 3], normal: [f32; 3], tex_coords: [f32; 2]) -> Self {
        Self {
            p: Point3::new(pos[0].into(), pos[1].into(), pos[2].into()),
            n: Vector3::new(normal[0].into(), normal[1].into(), normal[2].into()),
            t: Point2::new(tex_coords[0].into(), tex_coords[1].into()),
        }
    }
}

impl From<&Vertex> for RawVertex {
    fn from(v: &Vertex) -> Self {
        Self {
            pos: [v.p.x as f32, v.p.y as f32, v.p.z as f32],
            normal: [v.n.x as f32, v.n.y as f32, v.n.z as f32],
            tex_coords: [v.t.x as f32, v.t.y as f32],
        }
    }
}
