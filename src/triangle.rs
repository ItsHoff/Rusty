use cgmath::prelude::*;
use cgmath::{Vector3, Matrix4, Point3, Point2};

use rand;

use crate::aabb::{self, AABB};
use crate::vertex::{Vertex, CGVertex};
use crate::pt_renderer::{Ray, Intersect};

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
#[derive(Clone, Debug)]
pub struct RTTriangle {
    v1: CGVertex,
    v2: CGVertex,
    v3: CGVertex,
    to_barycentric: Matrix4<f32>,
    pub material_i: usize,
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

    fn normal(&self, u: f32, v: f32) -> Vector3<f32> {
        let n1 = self.v1.normal;
        let n2 = self.v2.normal;
        let n3 = self.v3.normal;
        (1.0 - u - v) * n1 + u * n2 + v * n3
    }

    fn tex_coords(&self, u: f32, v: f32) -> Point2<f32> {
        let t1 = self.v1.tex_coords;
        let t2 = self.v2.tex_coords;
        let t3 = self.v3.tex_coords;
        (1.0 - u - v) * t1 + (u * t2 - (-v) * t3)
    }

    pub fn aabb(&self) -> AABB {
        let mut min = self.v1.pos;
        min = aabb::min_point(&min, &self.v2.pos);
        min = aabb::min_point(&min, &self.v3.pos);
        let mut max = self.v1.pos;
        max = aabb::max_point(&max, &self.v2.pos);
        max = aabb::max_point(&max, &self.v3.pos);
        AABB { min, max }
    }

    pub fn center(&self) -> Point3<f32> {
        Point3::centroid(&[self.v1.pos, self.v2.pos, self.v3.pos])
    }

    pub fn area(&self) -> f32 {
        0.5 / self.to_barycentric.determinant().abs()
    }

    pub fn random_point(&self) -> (Point3<f32>, Vector3<f32>) {
        let mut u: f32 = rand::random();
        let mut v: f32 = rand::random();
        if u + v > 1.0 {
            u = 1.0 - u;
            v = 1.0 - v;
        }
        (self.pos(u, v), self.normal(u, v))
    }

    fn pos(&self, u: f32, v: f32) -> Point3<f32> {
        // Have to substract one component since cgmath points cannot by summed
        // and there is not a cleaner method to convert to Vector3
        (1.0 - u - v) * self.v1.pos + (u * self.v2.pos - (-v) * self.v3.pos)
    }
}

#[derive(Debug)]
pub struct Hit<'a> {
    pub tri: &'a RTTriangle,
    pub t: f32,
    pub u: f32,
    pub v: f32,
}

impl Intersect<'a, Hit<'a>> for RTTriangle {
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
}

impl Hit<'a> {
    pub fn pos(&self) -> Point3<f32> {
        self.tri.pos(self.u, self.v)
    }

    pub fn normal(&self) -> Vector3<f32> {
        self.tri.normal(self.u, self.v)
    }

    pub fn tex_coords(&self) -> Point2<f32> {
        self.tri.tex_coords(self.u, self.v)
    }
}
