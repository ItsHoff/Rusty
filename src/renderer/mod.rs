#![cfg_attr(feature="clippy", allow(forget_copy))]

mod gl_renderer;
mod pt_renderer;

use cgmath::{Vector3, Point2, Point3, Matrix4};
use cgmath::prelude::*;

use scene::Material;

pub use self::gl_renderer::GLRenderer;
pub use self::pt_renderer::PTRenderer;

/// Vertex using raw arrays that can be inserted in vertex buffers
#[derive(Copy, Clone, Debug, Default)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
}

implement_vertex!(Vertex, pos, normal, tex_coords);

/// Vertex utilising cgmath types
#[derive(Debug)]
struct CGVertex {
    pos: Point3<f32>,
    normal: Vector3<f32>,
    tex_coords: Point2<f32>,
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

    pub fn build(self, material_i: usize) -> Result<RTTriangle, String> {
        if self.vertices.len() != 3 {
            Err("Triangle doesn't have 3 vertices!".to_owned())
        } else {
            Ok(RTTriangle::new(
                CGVertex::from(self.vertices[0]),
                CGVertex::from(self.vertices[1]),
                CGVertex::from(self.vertices[2]),
                material_i
            ))
        }
    }
}

/// Tracable triangle
#[derive(Debug)]
pub struct RTTriangle {
    v1: CGVertex,
    v2: CGVertex,
    v3: CGVertex,
    to_barycentric: Matrix4<f32>,
    material_i: usize,
}

impl RTTriangle {
    fn new(v1: CGVertex, v2: CGVertex, v3: CGVertex, material_i: usize) -> RTTriangle {
        let p1 = v1.pos;
        let p2 = v2.pos;
        let p3 = v3.pos;
        let z = (p2 - p1).cross(p3 - p1).normalize();
        let from_barycentric = Matrix4::from_cols((p2-p1).extend(0.0),
                                                  (p3-p1).extend(0.0),
                                                  z.extend(0.0),
                                                  p1.to_homogeneous());
        let to_barycentric = from_barycentric.invert()
            .expect("Non invertible barycentric tranform");
        RTTriangle {
            v1, v2, v3,
            to_barycentric,
            material_i
        }
    }

    fn intersect(&self, ray: &Ray) -> Option<Hit> {
        let bary_o = self.to_barycentric * ray.orig.to_homogeneous();
        let bary_d = self.to_barycentric * ray.dir.extend(0.0);
        let t = -bary_o.z / bary_d.z;
        let u = bary_o.x + t * bary_d.x;
        let v = bary_o.y + t * bary_d.y;
        if u >= 0.0 && v >= 0.0 && u + v <= 1.0 && t > 0.0 && t < ray.length {
            Some ( Hit { tri: self, t, u, v } )
        } else {
            None
        }
    }

    /// Get the diffuse color of the triangle at (u, v)
    fn get_diffuse(&self, materials: &[Material], _u: f32, _v: f32) -> Vector3<f32> {
        let material = &materials[self.material_i];
        Vector3::from(material.diffuse)
    }

    fn get_normal(&self, u: f32, v: f32) -> Vector3<f32> {
        let n1 = self.v1.normal;
        let n2 = self.v2.normal;
        let n3 = self.v3.normal;
        (1.0 - u - v) * n1 + u * n2 + v * n3
    }
}

#[derive(Debug)]
pub struct Hit<'a> {
    tri: &'a RTTriangle,
    t: f32,
    u: f32,
    v: f32,
}

pub struct Ray {
    orig: Point3<f32>,
    dir: Vector3<f32>,
    length: f32,
}

impl Ray {
    fn new(orig: Point3<f32>, dir: Vector3<f32>, length: f32) -> Ray {
        Ray { orig, dir, length }
    }
}
