use cgmath::Vector3;

use crate::color::Color;
use crate::float::*;
use crate::pt_renderer::PathType;

use super::fresnel::{self, FresnelBsdf};
use super::util;
use super::BsdfT;

#[derive(Clone, Debug)]
pub struct SpecularBrdf {
    color: Color,
    use_schlick: bool,
}

impl SpecularBrdf {
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

impl BsdfT for SpecularBrdf {
    fn is_specular(&self) -> bool {
        true
    }

    fn brdf(&self, _wo: Vector3<Float>, _wi: Vector3<Float>) -> Color {
        Color::black()
    }

    fn btdf(&self, _wo: Vector3<Float>, _wi: Vector3<Float>, _path_type: PathType) -> Color {
        Color::black()
    }

    fn pdf(&self, _wo: Vector3<Float>, _wi: Vector3<Float>) -> Float {
        0.0
    }

    fn sample(
        &self,
        wo: Vector3<Float>,
        _path_type: PathType,
    ) -> Option<(Color, Vector3<Float>, Float)> {
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
pub struct SpecularBtdf {
    color: Color,
    eta: Float,
}

impl SpecularBtdf {
    pub fn new(color: Color, eta: Float) -> Self {
        Self { color, eta }
    }
}

impl BsdfT for SpecularBtdf {
    fn is_specular(&self) -> bool {
        true
    }

    fn brdf(&self, _wo: Vector3<Float>, _wi: Vector3<Float>) -> Color {
        Color::black()
    }

    fn btdf(&self, _wo: Vector3<Float>, _wi: Vector3<Float>, _path_type: PathType) -> Color {
        Color::black()
    }

    fn pdf(&self, _wo: Vector3<Float>, _wi: Vector3<Float>) -> Float {
        0.0
    }

    fn sample(
        &self,
        wo: Vector3<Float>,
        path_type: PathType,
    ) -> Option<(Color, Vector3<Float>, Float)> {
        let wi = util::refract_n(wo, self.eta)?;
        let mut color = self.color / util::cos_t(wi).abs();
        // Account for non-symmetry
        if path_type.is_camera() {
            let eta = util::eta(wo, self.eta);
            color *= eta.powi(2);
        }
        Some((color, wi, 1.0))
    }
}

pub type SpecularBsdf = FresnelBsdf<SpecularBrdf, SpecularBtdf>;

impl SpecularBsdf {
    pub fn new(reflect: Color, transmit: Color, eta: Float) -> Self {
        let brdf = SpecularBrdf::without_schlick(reflect);
        let btdf = SpecularBtdf::new(transmit, eta);
        Self { brdf, btdf, eta }
    }
}
