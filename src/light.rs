use cgmath::prelude::*;
use cgmath::Point3;

use crate::color::Color;
use crate::consts;
use crate::index_ptr::IndexPtr;
use crate::intersect::Interaction;
use crate::triangle::Triangle;
use crate::Float;

pub trait Light {
    fn power(&self) -> Color;

    /// Sample radiance toward receiving interaction.
    /// Return radiance, shadow ray and the pdf
    fn sample_li(&self, recv: &Interaction) -> (Color, Point3<Float>, Float);

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
    fn sample_li(&self, recv: &Interaction) -> (Color, Point3<Float>, Float) {
        let (u, v) = Triangle::sample();
        let p = self.tri.pos(u, v);
        let n = self.tri.normal(u, v);
        let mut pdf = self.tri.pdf_a();
        let dp = p - recv.p;
        let dir = dp.normalize();
        // Convert pdf to solid angle
        pdf *= dp.magnitude2() / n.dot(-dir).abs();
        let li = self.tri.le(-dir);
        (li, p, pdf)
    }

    // fn pdf_li(&self) {}

    // fn sample_le(&self) {}

    // fn pdf_le(&self) {}
}
