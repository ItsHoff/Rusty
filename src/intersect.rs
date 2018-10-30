use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

use cgmath::prelude::*;
use cgmath::{Matrix3, Point2, Point3, Vector3};

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

#[derive(Debug)]
pub struct Hit<'a> {
    pub tri: &'a Triangle,
    pub t: Float,
    pub u: Float,
    pub v: Float,
}

impl<'a> Hit<'a> {
    pub fn interaction(self) -> Interaction<'a> {
        let (p, n, t) = self.tri.bary_pnt(self.u, self.v);
        Interaction {
            tri: self.tri,
            to_local: world_to_normal(n),
            p,
            n,
            t,
            mat: &*self.tri.material,
        }
    }
}

/// Compute the orthonormal transformation to an arbitrary
/// coordinate frame where n defines is the z-axis
fn world_to_normal(n: Vector3<Float>) -> Matrix3<Float> {
    let nx = if n.x.abs() > n.y.abs() {
        Vector3::new(n.z, 0.0, -n.x).normalize()
    } else {
        Vector3::new(0.0, -n.z, n.y).normalize()
    };
    let ny = n.cross(nx);
    Matrix3::from_cols(nx, ny, n).transpose()
}

pub struct Interaction<'a> {
    pub tri: &'a Triangle,
    to_local: Matrix3<Float>,
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
        let angle = 2.0 * consts::PI * rand::random::<Float>();
        let length = rand::random::<Float>().sqrt();
        let x = length * angle.cos();
        let y = length * angle.sin();
        let z = (1.0 - length.powi(2)).sqrt();
        let dir = self.to_local.transpose() * Vector3::new(x, y, z);
        let pdf = z / consts::PI;
        (self.brdf(), dir, pdf)
    }
}
