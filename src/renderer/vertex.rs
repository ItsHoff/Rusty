use cgmath::{Vector3, Point2, Point3};

/// Vertex using raw arrays that can be inserted in vertex buffers
#[derive(Copy, Clone, Debug, Default)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
}

implement_vertex!(Vertex, pos, normal, tex_coords);

/// Vertex utilising cgmath types
#[derive(Clone, Debug)]
pub struct CGVertex {
    pub pos: Point3<f32>,
    pub normal: Vector3<f32>,
    pub tex_coords: Point2<f32>,
}

impl From<Vertex> for CGVertex {
    fn from(v: Vertex) -> CGVertex {
        CGVertex {
            pos: Point3::from(v.pos),
            normal: Vector3::from(v.normal),
            tex_coords: Point2::from(v.tex_coords),
        }
    }
}
