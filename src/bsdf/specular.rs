use cgmath::Vector3;

use crate::color::Color;
use crate::float::*;

use super::fresnel::{self, FresnelBSDF};
use super::util;
use super::BSDFT;

#[derive(Clone, Debug)]
pub struct SpecularBRDF {
    color: Color,
    use_schlick: bool,
}

impl SpecularBRDF {
    pub fn with_schlick(color: Color) -> Self {
        Self {
            color,
            use_schlick: true,
        }
    }

    pub fn without_schlick(color: Color) -> Self {
        Self {
            color,
            use_schlick: false,
        }
    }
}

impl BSDFT for SpecularBRDF {
    fn is_specular(&self) -> bool {
        true
    }

    fn brdf(&self, _wo: Vector3<Float>, _wi: Vector3<Float>) -> Color {
        Color::black()
    }

    fn btdf(&self, _wo: Vector3<Float>, _wi: Vector3<Float>) -> Color {
        Color::black()
    }

    fn pdf(&self, _wo: Vector3<Float>, _wi: Vector3<Float>) -> Float {
        0.0
    }

    fn sample(&self, wo: Vector3<Float>) -> Option<(Color, Vector3<Float>, Float)> {
        let wi = util::reflect_n(wo);
        let color = if self.use_schlick {
            fresnel::schlick(wo, self.color)
        } else {
            self.color
        };
        Some((color, wi, 1.0))
    }
}

#[derive(Clone, Debug)]
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

    fn brdf(&self, _wo: Vector3<Float>, _wi: Vector3<Float>) -> Color {
        Color::black()
    }

    fn btdf(&self, _wo: Vector3<Float>, _wi: Vector3<Float>) -> Color {
        Color::black()
    }

    fn pdf(&self, _wo: Vector3<Float>, _wi: Vector3<Float>) -> Float {
        0.0
    }

    fn sample(&self, wo: Vector3<Float>) -> Option<(Color, Vector3<Float>, Float)> {
        let wi = util::refract_n(wo, self.eta)?;
        // TODO: account for non-symmetry
        Some((self.color / util::cos_t(wi).abs(), wi, 1.0))
    }
}

pub type SpecularBSDF = FresnelBSDF<SpecularBRDF, SpecularBTDF>;

impl SpecularBSDF {
    pub fn new(reflect: Color, transmit: Color, eta: Float) -> Self {
        let brdf = SpecularBRDF::without_schlick(reflect);
        let btdf = SpecularBTDF::new(transmit, eta);
        Self { brdf, btdf, eta }
    }
}
