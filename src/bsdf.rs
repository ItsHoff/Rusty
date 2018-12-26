use std::ops::Deref;

use cgmath::Vector3;

use crate::color::Color;
use crate::float::*;

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
pub trait BSDFT {
    fn is_specular(&self) -> bool;
    /// Evaluate reflected irradiance
    fn brdf(&self, wo: Vector3<Float>, wi: Vector3<Float>) -> Color;
    /// Evaluate transmitted irradiance
    fn btdf(&self, wo: Vector3<Float>, wi: Vector3<Float>) -> Color;
    /// Sample the distribution
    fn sample(&self, wo: Vector3<Float>) -> Option<(Color, Vector3<Float>, Float)>;
}

#[derive(Debug)]
pub enum BSDF {
    LR(LambertianBRDF),
    MR(MicrofacetBRDF),
    SR(SpecularBRDF),
    SS(SpecularBSDF),
}

impl BSDF {
    pub fn lambertian_brdf(color: Color) -> Self {
        BSDF::LR(LambertianBRDF::new(color))
    }

    pub fn microfacet_brdf(color: Color, shininess: Float) -> Self {
        BSDF::MR(MicrofacetBRDF::new(color, shininess))
    }

    pub fn specular_brdf(color: Color) -> Self {
        BSDF::SR(SpecularBRDF::new(color))
    }

    pub fn specular_bsdf(reflect: Color, transmit: Color, eta: Float) -> Self {
        BSDF::SS(SpecularBSDF::new(reflect, transmit, eta))
    }
}

impl Deref for BSDF {
    type Target = dyn BSDFT;

    fn deref(&self) -> &Self::Target {
        use self::BSDF::*;
        match self {
            LR(inner) => inner,
            MR(inner) => inner,
            SR(inner) => inner,
            SS(inner) => inner,
        }
    }
}
