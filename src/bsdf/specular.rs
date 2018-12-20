use cgmath::prelude::*;
use cgmath::Vector3;

use crate::color::Color;
use crate::Float;

use super::BSDFT;

#[derive(Debug)]
pub struct SpecularBRDF {
    color: Color,
}

impl SpecularBRDF {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

impl BSDFT for SpecularBRDF {
    fn is_specular(&self) -> bool {
        true
    }

    fn eval(&self, _in_dir: Vector3<Float>, _out_dir: Vector3<Float>) -> Color {
        Color::black()
    }

    fn sample(&self, out_dir: Vector3<Float>) -> (Color, Vector3<Float>, Float) {
        let in_dir = Vector3::new(-out_dir.x, -out_dir.y, out_dir.z);
        (self.color, in_dir, 1.0)
    }
}

#[derive(Debug)]
pub struct SpecularBTDF {
    color: Color,
    eta: Float,
}

impl SpecularBTDF {
    pub fn new(color: Color, eta: Float) -> Self {
        Self { color, eta }
    }
}

impl BSDFT for SpecularBTDF {
    fn is_specular(&self) -> bool {
        true
    }

    fn eval(&self, _in_dir: Vector3<Float>, _out_dir: Vector3<Float>) -> Color {
        Color::black()
    }

    fn sample(&self, out_dir: Vector3<Float>) -> (Color, Vector3<Float>, Float) {
        let (n, eta) = if out_dir.z > 0.0 {
            (Vector3::unit_z(), 1.0 / self.eta)
        } else {
            (-Vector3::unit_z(), self.eta)
        };
        let cos_i = out_dir.dot(n);
        let sin2_i = (1.0 - cos_i.powi(2)).max(0.0);
        let sin2_t = eta.powi(2) * sin2_i;
        // Total internal reflection
        if sin2_t >= 1.0 {
            return (Color::black(), Vector3::unit_z(), 1.0);
        }
        let cos_t = (1.0 - sin2_t).sqrt();
        let in_dir = -out_dir * eta + (eta * cos_i - cos_t) * n;
        (self.color / in_dir.z.abs(), in_dir, 1.0)
    }
}

#[derive(Debug)]
pub struct FresnelBSDF {
    brdf: SpecularBRDF,
    btdf: SpecularBTDF,
}

impl FresnelBSDF {
    pub fn new(reflect: Color, transmit: Color, eta: Float) -> Self {
        let brdf = SpecularBRDF::new(reflect);
        let btdf = SpecularBTDF::new(transmit, eta);
        Self { brdf, btdf }
    }
}

fn fresnel_dielectric(mut cos_i: Float, eta_mat: Float) -> Float {
    let (eta_i, eta_t) = if cos_i > 0.0 {
        (1.0, eta_mat)
    } else {
        cos_i = -cos_i;
        (eta_mat, 1.0)
    };
    let sin2_i = (1.0 - cos_i.powi(2)).max(0.0);
    let sin2_t = (eta_i / eta_t).powi(2) * sin2_i;
    // Total internal reflection
    if sin2_t >= 1.0 {
        return 1.0;
    }
    let cos_t = (1.0 - sin2_t).sqrt();
    let paral = (eta_t * cos_i - eta_i * cos_t) / (eta_t * cos_i + eta_i * cos_t);
    let perp = (eta_i * cos_i - eta_t * cos_t) / (eta_i * cos_i + eta_t * cos_t);
    (paral.powi(2) + perp.powi(2)) / 2.0
}

impl BSDFT for FresnelBSDF {
    fn is_specular(&self) -> bool {
        true
    }

    fn eval(&self, _in_dir: Vector3<Float>, _out_dir: Vector3<Float>) -> Color {
        Color::black()
    }

    fn sample(&self, out_dir: Vector3<Float>) -> (Color, Vector3<Float>, Float) {
        let f = fresnel_dielectric(out_dir.z, self.btdf.eta);
        if rand::random::<Float>() < f {
            let (color, in_dir, pdf) = self.brdf.sample(out_dir);
            (f * color, in_dir, f * pdf)
        } else {
            let (color, in_dir, pdf) = self.btdf.sample(out_dir);
            let ft = 1.0 - f;
            (ft * color, in_dir, ft * pdf)
        }
    }
}
