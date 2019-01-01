use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

use cgmath::prelude::*;
use cgmath::{Matrix3, Point3, Vector3};

use crate::bsdf::BSDF;
use crate::color::Color;
use crate::config::RenderConfig;
use crate::consts;
use crate::float::*;
use crate::sample;
use crate::triangle::Triangle;

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
    pub fn from_dir(orig: Point3<Float>, dir: Vector3<Float>) -> Ray {
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
            to_local: sample::local_to_world(ns).transpose(),
            p,
            ns,
            ng: self.tri.ng,
            bsdf: self.tri.material.bsdf(t),
        }
    }
}

#[derive(Debug)]
pub struct Interaction<'a> {
    tri: &'a Triangle,
    to_local: Matrix3<Float>,
    p: Point3<Float>,
    pub ns: Vector3<Float>,
    ng: Vector3<Float>,
    bsdf: BSDF,
}

impl Interaction<'_> {
    pub fn le(&self, in_ray: &Ray) -> Color {
        self.tri.le(-in_ray.dir)
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

    pub fn is_specular(&self) -> bool {
        self.bsdf.is_specular()
    }

    pub fn bsdf(&self, in_ray: &Ray, out_ray: &Ray) -> Color {
        let in_dir = -in_ray.dir;
        let out_dir = out_ray.dir;
        self.eval_bsdf(in_dir, out_dir)
    }

    fn eval_bsdf(&self, wo: Vector3<Float>, wi: Vector3<Float>) -> Color {
        let wo_local = self.to_local * wo;
        let wi_local = self.to_local * wi;
        // Check if the interaction is geometrically transmitted or reflected
        if self.ng.dot(wo) * self.ng.dot(wi) < 0.0 {
            self.bsdf.btdf(wo_local, wi_local)
        } else {
            self.bsdf.brdf(wo_local, wi_local)
        }
    }

    pub fn sample_bsdf(&self, in_ray: &Ray) -> Option<(Color, Ray, Float)> {
        let wo = -in_ray.dir;
        let wo_local = self.to_local * wo;
        let (mut bsdf, wi_local, pdf) = self.bsdf.sample(wo_local)?;
        let wi = self.to_local.transpose() * wi_local;
        // Avoid light leaks caused by shading normals
        if !self.bsdf.is_specular() {
            bsdf = self.eval_bsdf(wo, wi);
        }
        Some((bsdf, self.ray(wi), pdf))
    }
}
