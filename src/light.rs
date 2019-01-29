use std::fmt::Debug;

use cgmath::prelude::*;
use cgmath::{Point3, Vector3};

use crate::color::Color;
use crate::consts;
use crate::float::*;
use crate::intersect::{Interaction, Ray};
use crate::sample;
use crate::triangle::Triangle;

pub trait Light: Debug {
    /// Total emissive power of the light
    fn power(&self) -> Color;

    /// Emitted radiance to dir
    fn le(&self, dir: Vector3<Float>) -> Color;

    /// Evaluate the cosine with dir
    fn cos_t(&self, dir: Vector3<Float>) -> Float;

    /// Check if light contains a delta distribution
    fn is_delta(&self) -> bool;

    /// Sample a position on the lights surface
    /// Return point and area pdf
    fn sample_pos(&self) -> (Point3<Float>, Float);

    /// Pdf of position sampling in area measure
    fn pdf_pos(&self) -> Float;

    /// Sample a direction for emitted radiance
    /// Return radiance, direction and solid angle pdf
    fn sample_dir(&self) -> (Color, Vector3<Float>, Float);

    /// Pdf of direction sampling in solid angle measure
    fn pdf_dir(&self, dir: Vector3<Float>) -> Float;

    /// Sample radiance toward receiving interaction.
    /// Return radiance, shadow ray and the pdf
    fn sample_towards(&self, recv: &Interaction) -> (Color, Ray, Float) {
        let (p, pdf_a) = self.sample_pos();
        let ray = recv.shadow_ray(p);
        let pdf = sample::to_dir_pdf(pdf_a, ray.length.powi(2), self.cos_t(ray.dir).abs());
        let le = self.le(-ray.dir);
        (le, ray, pdf)
    }
}

impl Light for Triangle {
    fn power(&self) -> Color {
        consts::PI * self.material.emissive.unwrap() * self.area()
    }

    fn le(&self, dir: Vector3<Float>) -> Color {
        if let Some(le) = self.material.emissive {
            if self.ng.dot(dir) > 0.0 {
                return le;
            }
        }
        Color::black()
    }

    fn cos_t(&self, dir: Vector3<Float>) -> Float {
        self.ng.dot(dir)
    }

    fn is_delta(&self) -> bool {
        false
    }

    fn sample_pos(&self) -> (Point3<Float>, Float) {
        let (u, v) = Triangle::sample();
        let (p, _, _) = self.bary_pnt(u, v);
        (p, self.pdf_pos())
    }

    fn pdf_pos(&self) -> Float {
        1.0 / self.area()
    }

    fn sample_dir(&self) -> (Color, Vector3<Float>, Float) {
        let local_dir = sample::cosine_sample_hemisphere(1.0);
        let dir_pdf = sample::cosine_hemisphere_pdf(local_dir.z.abs());
        let dir = sample::local_to_world(self.ng) * local_dir;
        (self.le(dir), dir, dir_pdf)
    }

    fn pdf_dir(&self, dir: Vector3<Float>) -> Float {
        let cos_t = self.cos_t(dir);
        if cos_t < 0.0 {
            0.0
        } else {
            sample::cosine_hemisphere_pdf(cos_t)
        }
    }
}

#[derive(Debug)]
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

    fn le(&self, _dir: Vector3<Float>) -> Color {
        self.intensity
    }

    fn cos_t(&self, _dir: Vector3<Float>) -> Float {
        1.0
    }

    fn is_delta(&self) -> bool {
        true
    }

    fn sample_pos(&self) -> (Point3<Float>, Float) {
        (self.pos, 1.0)
    }

    fn pdf_pos(&self) -> Float {
        0.0
    }

    fn sample_dir(&self) -> (Color, Vector3<Float>, Float) {
        let dir = sample::uniform_sample_sphere();
        let pdf = sample::uniform_sphere_pdf();
        (self.intensity, dir, pdf)
    }

    fn pdf_dir(&self, _dir: Vector3<Float>) -> Float {
        sample::uniform_sphere_pdf()
    }
}
