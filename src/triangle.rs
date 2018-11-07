use cgmath::prelude::*;
use cgmath::{Matrix3, Matrix4, Point2, Point3, Vector3};

use rand;

use crate::aabb::{self, AABB};
use crate::color::Color;
use crate::index_ptr::IndexPtr;
use crate::intersect::{Hit, Intersect, Ray};
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
        let to_barycentric = Self::world_to_barycentric(v1.p, v2.p, v3.p);
        Self {
            v1,
            v2,
            v3,
            ng,
            to_barycentric,
            material,
        }
    }

    /// Compute the conversion from world space to barycentric space
    fn world_to_barycentric(
        p1: Point3<Float>,
        p2: Point3<Float>,
        p3: Point3<Float>,
    ) -> Matrix4<Float> {
        // TODO: there should be a way to do this without matrix inversion
        let z = (p2 - p1).cross(p3 - p1).normalize();
        let from_barycentric = Matrix4::from_cols(
            (p2 - p1).extend(0.0),
            (p3 - p1).extend(0.0),
            z.extend(0.0),
            p1.to_homogeneous(),
        );
        from_barycentric
            .invert()
            .expect("Non invertible barycentric tranform")
    }

    /// Compute the conversion from tangent space to world space given a normal
    pub fn tangent_to_world(&self, n: Vector3<Float>) -> Option<Matrix3<Float>> {
        let v1 = &*self.v1;
        let v2 = &*self.v2;
        let v3 = &*self.v3;

        let dp1 = v2.p - v1.p;
        let dt1 = v2.t - v1.t;
        let dp2 = v3.p - v1.p;
        let dt2 = v3.t - v1.t;

        let det = dt1.x * dt2.y - dt1.y * dt2.x;
        // Triangle has zero area in texture space
        if det == 0.0 {
            return None;
        }
        let g_tangent = dt2.y * dp1 - dt1.y * dp2;
        // Input normal may not match geometric normal so we need make sure the tangent
        // is orthogonal with respect to the given normal
        let bitangent = n.cross(g_tangent).normalize();
        let tangent = bitangent.cross(n);
        // TODO: why the bitangent need to be flipped?
        // TODO: handle mirrored texture coordinates
        Some(Matrix3::from_cols(tangent, -bitangent, n))
    }

    /// Get the barycentric position, normal and texture coordinates
    #[allow(clippy::many_single_char_names)]
    pub fn bary_pnt(&self, u: Float, v: Float) -> (Point3<Float>, Vector3<Float>, Point2<Float>) {
        let v1 = &*self.v1;
        let p1 = v1.p;
        let n1 = v1.n;
        let t1 = v1.t;

        let v2 = &*self.v2;
        let p2 = v2.p;
        let n2 = v2.n;
        let t2 = v2.t;

        let v3 = &*self.v3;
        let p3 = v3.p;
        let n3 = v3.n;
        let t3 = v3.t;

        let b1 = 1.0 - u - v;
        let p = b1 * p1 + u * p2.to_vec() + v * p3.to_vec();
        let n = b1 * n1 + u * n2 + v * n3;
        let t = b1 * t1 + u * t2.to_vec() + v * t3.to_vec();
        (p, n, t)
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
