use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

use cgmath::prelude::*;
use cgmath::{Matrix3, Point3, Vector3};

use crate::bsdf::BSDF;
use crate::color::Color;
use crate::config::RenderConfig;
use crate::consts;
use crate::float::*;
use crate::light::Light;
use crate::pt_renderer::PathType;
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
    pub fn shadow(orig: Point3<Float>, to: Point3<Float>) -> Ray {
        let dp = to - orig;
        let length = dp.magnitude() - consts::EPSILON;
        let dir = dp.normalize();
        Ray::new(orig, dir, length)
    }

    pub fn count() -> usize {
        RAY_COUNT.load(Ordering::Relaxed)
    }

    pub fn increment_count() {
        RAY_COUNT.fetch_add(1, Ordering::Relaxed);
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

#[derive(Clone, Debug)]
pub struct Interaction<'a> {
    pub tri: &'a Triangle,
    to_local: Matrix3<Float>,
    pub p: Point3<Float>,
    pub ns: Vector3<Float>,
    ng: Vector3<Float>,
    bsdf: BSDF,
}

impl Interaction<'_> {
    pub fn le(&self, wo: Vector3<Float>) -> Color {
        self.tri.le(wo)
    }

    pub fn ray(&self, dir: Vector3<Float>) -> Ray {
        Ray::from_dir(self.ray_origin(dir), dir)
    }

    pub fn shadow_ray(&self, to: Point3<Float>) -> Ray {
        Ray::shadow(self.ray_origin(to - self.p), to)
    }

    pub fn ray_origin(&self, dir: Vector3<Float>) -> Point3<Float> {
        if dir.dot(self.ng) > 0.0 {
            self.p + consts::EPSILON * self.ng
        } else {
            self.p - consts::EPSILON * self.ng
        }
    }

    pub fn is_specular(&self) -> bool {
        self.bsdf.is_specular()
    }

    /// Evaluate geometric cosine of dir
    pub fn cos_g(&self, dir: Vector3<Float>) -> Float {
        self.ng.dot(dir)
    }

    /// Evaluate shading cosine of dir
    pub fn cos_s(&self, dir: Vector3<Float>) -> Float {
        self.ns.dot(dir)
    }

    pub fn pdf(&self, wo: Vector3<Float>, wi: Vector3<Float>) -> Float {
        let wo_local = self.to_local * wo;
        let wi_local = self.to_local * wi;
        self.bsdf.pdf(wo_local, wi_local)
    }

    /// Evaluate the bsdf for directions in world coordinates
    pub fn bsdf(&self, wo: Vector3<Float>, wi: Vector3<Float>, path_type: PathType) -> Color {
        let wo_local = self.to_local * wo;
        let wi_local = self.to_local * wi;
        self.normal_correction(wo, wi, path_type) * self.bsdf_local(wo_local, wi_local, path_type)
    }

    /// Evaluate the bsdf for directions in local coordinates without normal correction
    fn bsdf_local(
        &self,
        wo_local: Vector3<Float>,
        wi_local: Vector3<Float>,
        path_type: PathType,
    ) -> Color {
        let ng_local = self.to_local * self.ng;
        // Check if the interaction is geometrically transmitted or reflected
        if ng_local.dot(wo_local) * ng_local.dot(wi_local) < 0.0 {
            self.bsdf.btdf(wo_local, wi_local, path_type)
        } else {
            self.bsdf.brdf(wo_local, wi_local)
        }
    }

    /// Sample the bsdf for outgoing world dir wo.
    /// Return the value of the bsdf, continuation ray and sampling pdf.
    pub fn sample_bsdf(
        &self,
        wo: Vector3<Float>,
        path_type: PathType,
    ) -> Option<(Color, Ray, Float)> {
        let wo_local = self.to_local * wo;
        let (mut bsdf, wi_local, pdf) = self.bsdf.sample(wo_local, path_type)?;
        let wi = self.to_local.transpose() * wi_local;
        // Avoid light leaks caused by shading normals
        if !self.bsdf.is_specular() {
            bsdf = self.bsdf_local(wo_local, wi_local, path_type);
        }
        Some((
            self.normal_correction(wo, wi, path_type) * bsdf,
            self.ray(wi),
            pdf,
        ))
    }

    /// Compute the correction factor resulting from use of shading normals
    /// for paths starting from a light.
    fn normal_correction(
        &self,
        wo: Vector3<Float>,
        wi: Vector3<Float>,
        path_type: PathType,
    ) -> Float {
        if path_type.is_light() {
            (self.cos_s(wo) * self.cos_g(wi) / (self.cos_g(wo) * self.cos_s(wi))).abs()
        } else {
            1.0
        }
    }
}
