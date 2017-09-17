#![cfg_attr(feature="clippy", allow(forget_copy))]

mod gl_renderer;
mod pt_renderer;

use cgmath::{Vector3, Point2, Point3, Matrix4};
use cgmath::prelude::*;

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
            Ok(RTTriangle::new(
                CGVertex::from(self.vertices[0]),
                CGVertex::from(self.vertices[1]),
                CGVertex::from(self.vertices[2]))
               )
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
}

impl RTTriangle {
    fn new(v1: CGVertex, v2: CGVertex, v3: CGVertex) -> RTTriangle {
        let p1 = v1.position;
        let p2 = v2.position;
        let p3 = v3.position;
        let z = (p2 - p1).cross(p3 - p1).normalize();
        let from_barycentric = Matrix4::from_cols((p2-p1).extend(0.0),
                                                  (p3-p1).extend(0.0),
                                                  z.extend(0.0),
                                                  p1.to_homogeneous());
        let to_barycentric = from_barycentric.invert()
            .expect("Non invertible barycentric tranform");
        RTTriangle { v1: v1, v2: v2, v3: v3, to_barycentric: to_barycentric }
    }

    fn intersect(&self, ray: &Ray) -> Option<Hit> {
        let bary_o = self.to_barycentric * ray.orig.to_homogeneous();
        let bary_d = self.to_barycentric * ray.dir.extend(0.0);
        let t = -bary_o.z / bary_d.z;
        let u = bary_o.x + t * bary_d.x;
        let v = bary_o.y + t * bary_d.y;
        if u > 0.0 && v > 0.0 && u + v < 1.0 && t > 0.0 && t < ray.length {
            Some ( Hit { tri: self, t: t, u: u, v: v } )
        } else {
            None
        }
    }

    fn get_diffuse(&self, _u: f32, _v: f32) -> Vector3<f32> {
        let p = self.v1.position.to_vec();
        Vector3::new(p.x.abs(), p.y.abs(), p.z.abs()).normalize()
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
        Ray { orig: orig, dir: dir, length: length }
    }
}
