use std::ops::Deref;

use cgmath::Vector3;

use crate::color::Color;
use crate::float::*;
use crate::pt_renderer::PathType;

mod fresnel;
mod lambertian;
mod microfacet;
mod specular;
mod util;

use self::lambertian::*;
use self::microfacet::*;
use self::specular::*;

/// Trait for handling local light transport.
/// Directions should both point away from the intersection.
/// Directions should be given in a surface local coordinate system,
/// where (0, 0, 1) is the normal pointing outwards
pub trait BsdfT {
    fn is_specular(&self) -> bool;
    /// Evaluate reflected radiance
    fn brdf(&self, wo: Vector3<Float>, wi: Vector3<Float>) -> Color;
    /// Evaluate transmitted radiance
    fn btdf(&self, wo: Vector3<Float>, wi: Vector3<Float>, path_type: PathType) -> Color;
    /// Evaluate the pdf
    fn pdf(&self, wo: Vector3<Float>, wi: Vector3<Float>) -> Float;
    /// Sample the distribution
    fn sample(
        &self,
        wo: Vector3<Float>,
        path_type: PathType,
    ) -> Option<(Color, Vector3<Float>, Float)>;
}

#[derive(Clone, Debug)]
pub enum Bsdf {
    Fbr(FresnelBlendBrdf),
    Lr(LambertianBrdf),
    Mr(MicrofacetBrdf),
    Ms(MicrofacetBsdf),
    Sr(SpecularBrdf),
    Ss(SpecularBsdf),
}

impl Bsdf {
    pub fn fresnel_blend_brdf(diffuse: Color, specular: Color, shininess: Float) -> Self {
        Bsdf::Fbr(FresnelBlendBrdf::new(diffuse, specular, shininess))
    }

    pub fn lambertian_brdf(color: Color) -> Self {
        Bsdf::Lr(LambertianBrdf::new(color))
    }

    pub fn microfacet_brdf(color: Color, shininess: Float) -> Self {
        Bsdf::Mr(MicrofacetBrdf::with_schlick(color, shininess))
    }

    pub fn microfacet_bsdf(reflect: Color, transmit: Color, shininess: Float, eta: Float) -> Self {
        Bsdf::Ms(MicrofacetBsdf::new(reflect, transmit, shininess, eta))
    }

    pub fn specular_brdf(color: Color) -> Self {
        Bsdf::Sr(SpecularBrdf::with_schlick(color))
    }

    pub fn specular_bsdf(reflect: Color, transmit: Color, eta: Float) -> Self {
        Bsdf::Ss(SpecularBsdf::new(reflect, transmit, eta))
    }
}

impl Deref for Bsdf {
    type Target = dyn BsdfT;

    fn deref(&self) -> &Self::Target {
        use self::Bsdf::*;
        match self {
            Fbr(inner) => inner,
            Lr(inner) => inner,
            Mr(inner) => inner,
            Ms(inner) => inner,
            Sr(inner) => inner,
            Ss(inner) => inner,
        }
    }
}
