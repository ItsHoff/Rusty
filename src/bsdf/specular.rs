use cgmath::Vector3;

use crate::color::Color;
use crate::float::*;

use super::util;
use super::fresnel::FresnelBSDF;
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

    fn brdf(&self, _wo: Vector3<Float>, _wi: Vector3<Float>) -> Color {
        Color::black()
    }

    fn btdf(&self, _wo: Vector3<Float>, _wi: Vector3<Float>) -> Color {
        Color::black()
    }

    fn sample(&self, wo: Vector3<Float>) -> Option<(Color, Vector3<Float>, Float)> {
        let wi = util::reflect_n(wo);
        Some((self.color, wi, 1.0))
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

    fn brdf(&self, _wo: Vector3<Float>, _wi: Vector3<Float>) -> Color {
        Color::black()
    }

    fn btdf(&self, _wo: Vector3<Float>, _wi: Vector3<Float>) -> Color {
        Color::black()
    }

    fn sample(&self, wo: Vector3<Float>) -> Option<(Color, Vector3<Float>, Float)> {
        let wi = util::refract_n(wo, self.eta)?;
        Some((self.color / util::cos_t(wi).abs(), wi, 1.0))
    }
}

pub type SpecularBSDF = FresnelBSDF<SpecularBRDF, SpecularBTDF>;

impl SpecularBSDF {
    pub fn new(reflect: Color, transmit: Color, eta: Float) -> Self {
        let brdf = SpecularBRDF::new(reflect);
        let btdf = SpecularBTDF::new(transmit, eta);
        Self { brdf, btdf, eta }
    }
}
