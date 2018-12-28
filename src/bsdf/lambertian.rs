use cgmath::Vector3;

use crate::color::Color;
use crate::consts;
use crate::float::*;

use super::util;
use super::BSDFT;

#[derive(Debug)]
pub struct LambertianBRDF {
    color: Color,
}

impl LambertianBRDF {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

impl BSDFT for LambertianBRDF {
    fn is_specular(&self) -> bool {
        false
    }

    fn brdf(&self, _wo: Vector3<Float>, _wi: Vector3<Float>) -> Color {
        self.color / consts::PI
    }

    fn btdf(&self, _wo: Vector3<Float>, _wi: Vector3<Float>) -> Color {
        Color::black()
    }

    fn sample(&self, wo: Vector3<Float>) -> Option<(Color, Vector3<Float>, Float)> {
        let wi = util::cosine_sample_hemisphere(wo);
        let val = self.brdf(wo, wi);
        let pdf = util::cos_t(wi).abs() / consts::PI;
        Some((val, wi, pdf))
    }
}
