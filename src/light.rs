use cgmath::prelude::*;

use crate::color::Color;
use crate::consts;
use crate::float::*;
use crate::index_ptr::IndexPtr;
use crate::intersect::{Interaction, Ray};
use crate::triangle::Triangle;

pub trait Light {
    fn power(&self) -> Color;

    /// Sample radiance toward receiving interaction.
    /// Return radiance, shadow ray and the pdf
    fn sample_li(&self, recv: &Interaction) -> (Color, Ray, Float);

    // fn pdf_li(&self) {}

    // fn sample_le(&self) {}

    // fn pdf_le(&self) {}
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

    /// Sample radiance toward receiving interaction.
    /// Return radiance, shadow ray and the pdf
    fn sample_li(&self, recv: &Interaction) -> (Color, Ray, Float) {
        let (u, v) = Triangle::sample();
        let (p, n, _) = self.tri.bary_pnt(u, v);
        let mut pdf = self.tri.pdf_a();
        let ray = recv.shadow_ray(p);
        // Convert pdf to solid angle
        pdf *= ray.length.powi(2) / n.dot(ray.dir).abs();
        let li = self.tri.le(-ray.dir);
        (li, ray, pdf)
    }

    // fn pdf_li(&self) {}

    // fn sample_le(&self) {}

    // fn pdf_le(&self) {}
}
