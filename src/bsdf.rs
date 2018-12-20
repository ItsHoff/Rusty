use std::ops::Deref;

use cgmath::Vector3;

use crate::color::Color;
use crate::Float;

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
/// where (0, 0, 1) is the normal
pub trait BSDFT {
    fn is_specular(&self) -> bool;
    fn eval(&self, wo: Vector3<Float>, wi: Vector3<Float>) -> Color;
    fn sample(&self, wo: Vector3<Float>) -> Option<(Color, Vector3<Float>, Float)>;
}

#[derive(Debug)]
pub enum BSDF {
    LR(LambertianBRDF),
    SR(SpecularBRDF),
    ST(SpecularBTDF),
    F(FresnelBSDF),
    MR(MicrofacetBRDF),
}

impl BSDF {
    pub fn lambertian_brdf(color: Color) -> Self {
        BSDF::LR(LambertianBRDF::new(color))
    }

    pub fn specular_brdf(color: Color) -> Self {
        BSDF::SR(SpecularBRDF::new(color))
    }

    pub fn specular_btdf(color: Color, eta: Float) -> Self {
        BSDF::ST(SpecularBTDF::new(color, eta))
    }

    pub fn fresnel_specular(reflect: Color, transmit: Color, eta: Float) -> Self {
        BSDF::F(FresnelBSDF::new(reflect, transmit, eta))
    }

    pub fn microfacet_brdf(color: Color, shininess: Float) -> Self {
        BSDF::MR(MicrofacetBRDF::new(color, shininess))
    }
}

impl Deref for BSDF {
    type Target = dyn BSDFT;

    fn deref(&self) -> &Self::Target {
        use self::BSDF::*;
        match self {
            LR(inner) => inner,
            SR(inner) => inner,
            ST(inner) => inner,
            F(inner) => inner,
            MR(inner) => inner,
        }
    }
}
