use cgmath::prelude::*;
use cgmath::{Matrix4, Point2, Point3, Vector3};

use rand;

use crate::aabb::{self, AABB};
use crate::index_ptr::IndexPtr;
use crate::material::Material;
use crate::pt_renderer::{Intersect, Ray};
use crate::vertex::Vertex;
use crate::Float;

#[derive(Default)]
pub struct RTTriangleBuilder {
    vertices: Vec<IndexPtr<Vertex>>,
}

impl RTTriangleBuilder {
    pub fn new() -> RTTriangleBuilder {
        RTTriangleBuilder {
            ..Default::default()
        }
    }

    pub fn add_vertex(&mut self, vertex: IndexPtr<Vertex>) {
        self.vertices.push(vertex);
    }

    pub fn build(self, material: IndexPtr<Material>) -> Result<RTTriangle, String> {
        if self.vertices.len() != 3 {
            Err("Triangle doesn't have 3 vertices!".to_string())
        } else {
            Ok(RTTriangle::new(
                self.vertices[0].clone(),
                self.vertices[1].clone(),
                self.vertices[2].clone(),
                material,
            ))
        }
    }
}

/// Tracable triangle
#[derive(Clone, Debug)]
pub struct RTTriangle {
    v1: IndexPtr<Vertex>,
    v2: IndexPtr<Vertex>,
    v3: IndexPtr<Vertex>,
    to_barycentric: Matrix4<Float>,
    pub material: IndexPtr<Material>,
}

impl RTTriangle {
    fn new(
        v1: IndexPtr<Vertex>,
        v2: IndexPtr<Vertex>,
        v3: IndexPtr<Vertex>,
        material: IndexPtr<Material>,
    ) -> RTTriangle {
        let p1 = v1.p;
        let p2 = v2.p;
        let p3 = v3.p;
        let z = (p2 - p1).cross(p3 - p1).normalize();
        let from_barycentric = Matrix4::from_cols(
            (p2 - p1).extend(0.0),
            (p3 - p1).extend(0.0),
            z.extend(0.0),
            p1.to_homogeneous(),
        );
        let to_barycentric = from_barycentric
            .invert()
            .expect("Non invertible barycentric tranform");
        RTTriangle {
            v1,
            v2,
            v3,
            to_barycentric,
            material,
        }
    }

    fn normal(&self, u: Float, v: Float) -> Vector3<Float> {
        let n1 = self.v1.n;
        let n2 = self.v2.n;
        let n3 = self.v3.n;
        (1.0 - u - v) * n1 + u * n2 + v * n3
    }

    fn tex_coords(&self, u: Float, v: Float) -> Point2<Float> {
        let t1 = self.v1.t;
        let t2 = self.v2.t;
        let t3 = self.v3.t;
        (1.0 - u - v) * t1 + (u * t2 - (-v) * t3)
    }

    pub fn aabb(&self) -> AABB {
        let mut min = self.v1.p;
        min = aabb::min_point(&min, &self.v2.p);
        min = aabb::min_point(&min, &self.v3.p);
        let mut max = self.v1.p;
        max = aabb::max_point(&max, &self.v2.p);
        max = aabb::max_point(&max, &self.v3.p);
        AABB { min, max }
    }

    pub fn center(&self) -> Point3<Float> {
        Point3::centroid(&[self.v1.p, self.v2.p, self.v3.p])
    }

    pub fn area(&self) -> Float {
        0.5 / self.to_barycentric.determinant().abs()
    }

    pub fn random_point(&self) -> (Point3<Float>, Vector3<Float>) {
        let mut u: Float = rand::random();
        let mut v: Float = rand::random();
        // TODO: use sampling that doesnt need this flip
        if u + v > 1.0 {
            u = 1.0 - u;
            v = 1.0 - v;
        }
        (self.pos(u, v), self.normal(u, v))
    }

    fn pos(&self, u: Float, v: Float) -> Point3<Float> {
        // Have to substract one component since cgmath points cannot by summed
        // and there is not a cleaner method to convert to Vector3
        (1.0 - u - v) * self.v1.p + (u * self.v2.p - (-v) * self.v3.p)
    }
}

#[derive(Debug)]
pub struct Hit<'a> {
    pub tri: &'a RTTriangle,
    pub t: Float,
    pub u: Float,
    pub v: Float,
}

impl<'a> Intersect<'a, Hit<'a>> for RTTriangle {
    fn intersect(&self, ray: &Ray) -> Option<Hit> {
        let bary_o = self.to_barycentric * ray.orig.to_homogeneous();
        let bary_d = self.to_barycentric * ray.dir.extend(0.0);
        let t = -bary_o.z / bary_d.z;
        let u = bary_o.x + t * bary_d.x;
        let v = bary_o.y + t * bary_d.y;
        if u >= 0.0 && v >= 0.0 && u + v <= 1.0 && t > 0.0 && t < ray.length {
            Some(Hit { tri: self, t, u, v })
        } else {
            None
        }
    }
}

impl Hit<'_> {
    pub fn pos(&self) -> Point3<Float> {
        self.tri.pos(self.u, self.v)
    }

    pub fn normal(&self) -> Vector3<Float> {
        self.tri.normal(self.u, self.v)
    }

    pub fn tex_coords(&self) -> Point2<Float> {
        self.tri.tex_coords(self.u, self.v)
    }
}
