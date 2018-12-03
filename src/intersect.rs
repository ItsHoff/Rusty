use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

use cgmath::prelude::*;
use cgmath::{Matrix3, Point3, Vector3};

use crate::bsdf::BSDF;
use crate::color::Color;
use crate::config::RenderConfig;
use crate::consts;
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
    pub fn interaction(self, config: &RenderConfig) -> Interaction<'a> {
        let (p, mut ns, t) = self.tri.bary_pnt(self.u, self.v);
        if config.normal_mapping {
            if let Some(ts_normal) = self.tri.material.normal(t) {
                if let Some(to_world) = self.tri.tangent_to_world(ns) {
                    ns = to_world * ts_normal;
                }
            }
        }
        Interaction {
            tri: self.tri,
            to_local: world_to_normal(ns),
            p,
            ns,
            ng: self.tri.ng,
            bsdf: self.tri.material.bsdf(t),
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
    tri: &'a Triangle,
    to_local: Matrix3<Float>,
    p: Point3<Float>,
    pub ns: Vector3<Float>,
    ng: Vector3<Float>,
    bsdf: Box<dyn BSDF>,
}

impl Interaction<'_> {
    pub fn le(&self, dir: Vector3<Float>) -> Color {
        self.tri.le(dir)
    }

    pub fn ray(&self, dir: Vector3<Float>) -> Ray {
        Ray::from_dir(self.ray_origin(dir), dir)
    }

    pub fn shadow_ray(&self, to: Point3<Float>) -> Ray {
        Ray::shadow(self.ray_origin(to - self.p), to)
    }

    fn ray_origin(&self, dir: Vector3<Float>) -> Point3<Float> {
        if dir.dot(self.ng) > 0.0 {
            self.p + consts::EPSILON * self.ng
        } else {
            self.p - consts::EPSILON * self.ng
        }
    }

    pub fn bsdf(&self, in_dir: Vector3<Float>, out_dir: Vector3<Float>) -> Color {
        if self.ng.dot(in_dir) * self.ng.dot(out_dir) < 0.0 {
            // TODO: evaluate transmission
            return Color::black();
        }
        let local_in = self.to_local * in_dir;
        let local_out = self.to_local * out_dir;
        self.bsdf.eval(local_in, local_out)
    }

    pub fn sample_bsdf(&self, out_dir: Vector3<Float>) -> (Color, Ray, Float) {
        let local_out = self.to_local * out_dir;
        let (bsdf, local_in, pdf) = self.bsdf.sample(local_out);
        let in_dir = self.to_local.transpose() * local_in;
        // TODO: avoid light leaks caused by shading normals
        (bsdf, self.ray(in_dir), pdf)
    }
}
