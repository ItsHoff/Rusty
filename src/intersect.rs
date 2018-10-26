use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

use cgmath::prelude::*;
use cgmath::{Point2, Point3, Vector3};

use crate::color::Color;
use crate::consts;
use crate::material::Material;
use crate::triangle::Triangle;
use crate::Float;

static RAY_COUNT: AtomicUsize = ATOMIC_USIZE_INIT;

pub trait Intersect<'a, H> {
    fn intersect(&'a self, ray: &Ray) -> Option<H>;
}

#[derive(Clone, Debug)]
pub struct Ray {
    pub orig: Point3<Float>,
    pub dir: Vector3<Float>,
    pub length: Float,
    // For more efficient ray box intersections
    pub reciprocal_dir: Vector3<Float>,
    pub neg_dir: [bool; 3],
}

impl Ray {
    fn new(orig: Point3<Float>, dir: Vector3<Float>, length: Float) -> Ray {
        RAY_COUNT.fetch_add(1, Ordering::Relaxed);
        let reciprocal_dir = 1.0 / dir;
        let neg_dir = [dir.x < 0.0, dir.y < 0.0, dir.z < 0.0];
        Ray {
            orig,
            dir,
            length,
            reciprocal_dir,
            neg_dir,
        }
    }

    /// Infinite ray with a given direction and origin
    fn from_dir(orig: Point3<Float>, dir: Vector3<Float>) -> Ray {
        Ray::new(orig, dir, consts::INFINITY)
    }

    /// Infinite ray from origin towards another point
    pub fn from_point(orig: Point3<Float>, to: Point3<Float>) -> Ray {
        let dir = (to - orig).normalize();
        Ray::new(orig, dir, consts::INFINITY)
    }

    /// Shadow ray between two points
    fn shadow(orig: Point3<Float>, to: Point3<Float>) -> Ray {
        let dp = to - orig;
        let length = dp.magnitude() - consts::EPSILON;
        let dir = dp.normalize();
        Ray::new(orig, dir, length)
    }

    pub fn count() -> usize {
        RAY_COUNT.load(Ordering::Relaxed)
    }

    pub fn reset_count() {
        RAY_COUNT.store(0, Ordering::SeqCst);
    }
}

pub struct Interaction<'a> {
    pub tri: &'a Triangle,
    pub p: Point3<Float>,
    pub n: Vector3<Float>,
    pub t: Point2<Float>,
    pub mat: &'a Material,
}

impl Interaction<'_> {
    pub fn le(&self, dir: Vector3<Float>) -> Color {
        self.tri.le(dir)
    }

    pub fn ray(&self, dir: Vector3<Float>) -> Ray {
        Ray::from_dir(self.ray_origin(), dir)
    }

    pub fn shadow_ray(&self, to: Point3<Float>) -> Ray {
        Ray::shadow(self.ray_origin(), to)
    }

    fn ray_origin(&self) -> Point3<Float> {
        self.p + consts::EPSILON * self.n
    }

    pub fn brdf(&self) -> Color {
        self.mat.diffuse(self.t) / consts::PI
    }

    pub fn sample_brdf(&self) -> (Color, Vector3<Float>, Float) {
        let dir = 2.0 * consts::PI * rand::random::<Float>();
        let length = rand::random::<Float>().sqrt();
        let x = length * dir.cos();
        let y = length * dir.sin();
        let z = (1.0 - length.powi(2)).sqrt();
        let nx = if self.n.x.abs() > self.n.y.abs() {
            Vector3::new(self.n.z, 0.0, -self.n.x).normalize()
        } else {
            Vector3::new(0.0, -self.n.z, self.n.y).normalize()
        };
        let ny = self.n.cross(nx);
        let dir = x * nx + y * ny + z * self.n;
        let pdf = z / consts::PI;
        (self.brdf(), dir, pdf)
    }
}
