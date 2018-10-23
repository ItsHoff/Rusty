use cgmath::prelude::*;
use cgmath::{Matrix4, Point2, Point3, Vector3};

use rand;

use crate::aabb::{self, AABB};
use crate::color::Color;
use crate::index_ptr::IndexPtr;
use crate::intersect::{Interaction, Intersect, Ray};
use crate::material::Material;
use crate::util::ConvArr;
use crate::vertex::Vertex;
use crate::Float;

#[derive(Default)]
pub struct TriangleBuilder {
    vertices: Vec<IndexPtr<Vertex>>,
}

impl TriangleBuilder {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
        }
    }

    pub fn add_vertex(&mut self, vertex: IndexPtr<Vertex>) {
        self.vertices.push(vertex);
    }

    pub fn build(self, ng: [f32; 3], material: IndexPtr<Material>) -> Result<Triangle, String> {
        if self.vertices.len() != 3 {
            Err("Triangle doesn't have 3 vertices!".to_string())
        } else {
            Ok(Triangle::new(
                self.vertices[0].clone(),
                self.vertices[1].clone(),
                self.vertices[2].clone(),
                Vector3::from_arr(ng),
                material,
            ))
        }
    }
}

/// Tracable triangle
#[derive(Clone, Debug)]
pub struct Triangle {
    v1: IndexPtr<Vertex>,
    v2: IndexPtr<Vertex>,
    v3: IndexPtr<Vertex>,
    /// Geometric normal
    ng: Vector3<Float>,
    to_barycentric: Matrix4<Float>,
    pub material: IndexPtr<Material>,
}

impl Triangle {
    fn new(
        v1: IndexPtr<Vertex>,
        v2: IndexPtr<Vertex>,
        v3: IndexPtr<Vertex>,
        ng: Vector3<Float>,
        material: IndexPtr<Material>,
    ) -> Self {
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
        Self {
            v1,
            v2,
            v3,
            ng,
            to_barycentric,
            material,
        }
    }

    pub fn pos(&self, u: Float, v: Float) -> Point3<Float> {
        let p1 = self.v1.p;
        let p2 = self.v2.p;
        let p3 = self.v3.p;
        (1.0 - u - v) * p1 + u * p2.to_vec() + v * p3.to_vec()
    }

    pub fn normal(&self, u: Float, v: Float) -> Vector3<Float> {
        let n1 = self.v1.n;
        let n2 = self.v2.n;
        let n3 = self.v3.n;
        (1.0 - u - v) * n1 + u * n2 + v * n3
    }

    fn tex_coords(&self, u: Float, v: Float) -> Point2<Float> {
        let t1 = self.v1.t;
        let t2 = self.v2.t;
        let t3 = self.v3.t;
        (1.0 - u - v) * t1 + u * t2.to_vec() + v * t3.to_vec()
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

    pub fn le(&self, dir: Vector3<Float>) -> Color {
        if let Some(le) = self.material.emissive {
            if self.ng.dot(dir) > 0.0 {
                return le;
            }
        }
        Color::black()
    }

    pub fn center(&self) -> Point3<Float> {
        Point3::centroid(&[self.v1.p, self.v2.p, self.v3.p])
    }

    pub fn area(&self) -> Float {
        0.5 / self.to_barycentric.determinant().abs()
    }

    pub fn pdf_a(&self) -> Float {
        1.0 / self.area()
    }

    pub fn sample() -> (Float, Float) {
        let r1: Float = rand::random();
        let r2: Float = rand::random();
        let sr1 = r1.sqrt();
        let u = 1.0 - sr1;
        let v = r2 * sr1;
        (u, v)
    }
}

#[derive(Debug)]
pub struct Hit<'a> {
    pub tri: &'a Triangle,
    pub t: Float,
    pub u: Float,
    pub v: Float,
}

impl<'a> Intersect<'a, Hit<'a>> for Triangle {
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

impl<'a> Hit<'a> {
    pub fn interaction(self) -> Interaction<'a> {
        Interaction {
            tri: self.tri,
            p: self.tri.pos(self.u, self.v),
            n: self.tri.normal(self.u, self.v),
            t: self.tri.tex_coords(self.u, self.v),
            mat: &*self.tri.material,
        }
    }
}
