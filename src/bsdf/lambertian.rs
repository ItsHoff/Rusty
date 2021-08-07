use cgmath::Vector3;

use crate::color::Color;
use crate::consts;
use crate::float::*;
use crate::pt_renderer::PathType;
use crate::sample;

use super::util;
use super::BsdfT;

#[derive(Clone, Debug)]
pub struct LambertianBrdf {
    color: Color,
}

impl LambertianBrdf {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

impl BsdfT for LambertianBrdf {
    fn is_specular(&self) -> bool {
        false
    }

    fn brdf(&self, _wo: Vector3<Float>, _wi: Vector3<Float>) -> Color {
        self.color / consts::PI
    }

    fn btdf(&self, _wo: Vector3<Float>, _wi: Vector3<Float>, _path_type: PathType) -> Color {
        Color::black()
    }

    fn pdf(&self, wo: Vector3<Float>, wi: Vector3<Float>) -> Float {
        if util::same_hemisphere(wo, wi) {
            sample::cosine_hemisphere_pdf(util::cos_t(wi).abs())
        } else {
            0.0
        }
    }

    fn sample(
        &self,
        wo: Vector3<Float>,
        _path_type: PathType,
    ) -> Option<(Color, Vector3<Float>, Float)> {
        let wi = sample::cosine_sample_hemisphere(wo.z);
        let val = self.brdf(wo, wi);
        let pdf = sample::cosine_hemisphere_pdf(util::cos_t(wi).abs());
        Some((val, wi, pdf))
    }
}
