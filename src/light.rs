use cgmath::prelude::*;
use cgmath::{Point3, Vector3};

use crate::color::Color;
use crate::consts;
use crate::float::*;
use crate::index_ptr::IndexPtr;
use crate::intersect::{Interaction, Intersect, Ray};
use crate::sample;
use crate::triangle::Triangle;

pub trait Light {
    fn power(&self) -> Color;

    /// Sample radiance toward receiving interaction.
    /// Return radiance, shadow ray and the pdf
    fn sample_li(&self, recv: &Interaction) -> (Color, Ray, Float);

    /// Evaluate pdf of sampling radiance towards receiving interaction.
    fn pdf_li(&self, recv: &Interaction, w: Vector3<Float>) -> Float;

    /// Sample emitted radiance of the light.
    /// Return radiance, shadow ray, normal, area pdf and directional pdf
    fn sample_le(&self) -> (Color, Ray, Vector3<Float>, Float, Float);

    /// Evaluate pdf of sampling emitted radiance.
    /// Return area pdf and directional pdf separately.
    fn pdf_le(&self, wi: Vector3<Float>) -> (Float, Float);
}

pub struct AreaLight {
    tri: IndexPtr<Triangle>,
}

impl AreaLight {
    pub fn new(tri: IndexPtr<Triangle>) -> Self {
        Self { tri }
    }
}

impl Light for AreaLight {
    fn power(&self) -> Color {
        consts::PI * self.tri.material.emissive.unwrap() * self.tri.area()
    }

    fn sample_li(&self, recv: &Interaction) -> (Color, Ray, Float) {
        let (u, v) = Triangle::sample();
        let (p, _, _) = self.tri.bary_pnt(u, v);
        let ray = recv.shadow_ray(p);
        let pdf = sample::to_dir_pdf(self.tri.pdf_a(), &ray, self.tri.ng);
        let li = self.tri.le(-ray.dir);
        (li, ray, pdf)
    }

    fn pdf_li(&self, recv: &Interaction, w: Vector3<Float>) -> Float {
        let ray = recv.ray(w);
        if let Some(hit) = self.tri.intersect(&ray) {
            sample::to_dir_pdf(self.tri.pdf_a(), &ray, self.tri.ng)
        } else {
            0.0
        }
    }

    fn sample_le(&self) -> (Color, Ray, Vector3<Float>, Float, Float) {
        let (u, v) = Triangle::sample();
        let (p, n, _) = self.tri.bary_pnt(u, v);
        let area_pdf = self.tri.pdf_a();
        let local_dir = sample::cosine_sample_hemisphere(1.0);
        let dir_pdf = sample::cosine_hemisphere_pdf(local_dir);
        let dir = sample::local_to_world(n) * local_dir;
        (self.tri.le(dir), Ray::from_dir(p, dir), n, area_pdf, dir_pdf)
    }

    fn pdf_le(&self, w: Vector3<Float>) -> (Float, Float) {
        let cos_t = self.tri.ng.dot(w);
        if cos_t < 0.0 {
            (0.0, 0.0)
        } else {
            (self.tri.pdf_a(), cos_t.abs() / consts::PI)
        }
    }
}

pub struct PointLight {
    pos: Point3<Float>,
    intensity: Color,
}

impl PointLight {
    pub fn new(pos: Point3<Float>, intensity: Color) -> Self {
        Self { pos, intensity }
    }
}

// Enable the use of camera as a backup light
impl Light for PointLight {
    fn power(&self) -> Color {
        4.0 * consts::PI * self.intensity
    }

    fn sample_li(&self, recv: &Interaction) -> (Color, Ray, Float) {
        let ray = recv.shadow_ray(self.pos);
        let li = self.intensity / ray.length.powi(2);
        (li, ray, 1.0)
    }

    fn pdf_li(&self, _recv: &Interaction, _w: Vector3<Float>) -> Float {
        0.0
    }

    fn sample_le(&self) -> (Color, Ray, Vector3<Float>, Float, Float) {
        let dir = sample::uniform_sample_sphere();
        let dir_pdf = sample::uniform_sphere_pdf();
        let ray = Ray::from_dir(self.pos, dir);
        let n = ray.dir;
        (self.intensity, ray, n, 1.0, dir_pdf)
    }

    fn pdf_le(&self, wi: Vector3<Float>) -> (Float, Float) {
        (0.0, sample::uniform_sphere_pdf())
    }
}
