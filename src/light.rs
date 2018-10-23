use cgmath::prelude::*;
use cgmath::Point3;

use crate::color::Color;
use crate::consts::PI;
use crate::index_ptr::IndexPtr;
use crate::intersect::Interaction;
use crate::triangle::Triangle;
use crate::Float;

pub struct Light {
    // Emitted radiance
    le: Color,
    tri: IndexPtr<Triangle>,
}

impl Light {
    pub fn new(tri: IndexPtr<Triangle>) -> Self {
        Self {
            le: tri.material.emissive.unwrap(),
            tri,
        }
    }

    // fn power(&self) -> Color {
    //     self.le * self.tri.area() * PI
    // }

    /// Sample radiance toward receiving interaction.
    /// Return radiance, shadow ray and the pdf
    pub fn sample_li(&self, recv: &Interaction) -> (Color, Point3<Float>, Float) {
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
