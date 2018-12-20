use cgmath::Vector3;

use crate::color::Color;
use crate::Float;

use super::BSDFT;
use super::util;

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

    fn eval(&self, _wo: Vector3<Float>, _wi: Vector3<Float>) -> Color {
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

    fn eval(&self, _wo: Vector3<Float>, _wi: Vector3<Float>) -> Color {
        Color::black()
    }

    fn sample(&self, wo: Vector3<Float>) -> Option<(Color, Vector3<Float>, Float)> {
        let wi = util::refract_n(wo, self.eta)?;
        Some((self.color / util::cos_t(wi).abs(), wi, 1.0))
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

impl BSDFT for FresnelBSDF {
    fn is_specular(&self) -> bool {
        true
    }

    fn eval(&self, _wo: Vector3<Float>, _wi: Vector3<Float>) -> Color {
        Color::black()
    }

    fn sample(&self, wo: Vector3<Float>) -> Option<(Color, Vector3<Float>, Float)> {
        let fr = util::fresnel_dielectric(wo, self.btdf.eta);
        if rand::random::<Float>() < fr {
            let (color, wi, pdf) = self.brdf.sample(wo)?;
            Some((fr * color, wi, fr * pdf))
        } else {
            let (color, wi, pdf) = self.btdf.sample(wo)?;
            let ft = 1.0 - fr;
            Some((ft * color, wi, ft * pdf))
        }
    }
}
