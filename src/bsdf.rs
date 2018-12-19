use std::ops::Deref;

use cgmath::Vector3;

use crate::color::Color;
use crate::Float;

mod lambertian;
mod specular;

use self::lambertian::*;
use self::specular::*;

/// Trait for handling local light transport.
/// Directions should both point away from the intersection.
/// in_dir corresponds to the direction photons arrive from and
/// out_dir refer to the direction of the photons scatter towards.
/// Directions should be given in a surface local coordinate system,
/// where (0, 0, 1) is the normal
pub trait BSDFT {
    fn is_specular(&self) -> bool;
    fn eval(&self, in_dir: Vector3<Float>, out_dir: Vector3<Float>) -> Color;
    fn sample(&self, out_dir: Vector3<Float>) -> (Color, Vector3<Float>, Float);
}

#[derive(Debug)]
pub enum BSDF {
    LR(LambertianBRDF),
    SR(SpecularBRDF),
    ST(SpecularBTDF),
    F(FresnelBSDF),
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
        }
    }
}
