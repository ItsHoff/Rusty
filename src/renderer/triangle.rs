use cgmath::prelude::*;
use cgmath::{Vector3, Matrix4, Point3};

use scene::Material;

use aabb;
use aabb::AABB;
use super::{Vertex, CGVertex, Ray, Hit, Intersect};

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

    /// Get the diffuse color of the triangle at (u, v)
    pub fn diffuse(&self, materials: &[Material], _u: f32, _v: f32) -> Vector3<f32> {
        let material = &materials[self.material_i];
        Vector3::from(material.diffuse)
    }

    pub fn normal(&self, u: f32, v: f32) -> Vector3<f32> {
        let n1 = self.v1.normal;
        let n2 = self.v2.normal;
        let n3 = self.v3.normal;
        (1.0 - u - v) * n1 + u * n2 + v * n3
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
}

impl<'a> Intersect<'a, Hit<'a>> for RTTriangle {
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
