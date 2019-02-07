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
pub trait BSDFT {
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
pub enum BSDF {
    FBR(FresnelBlendBRDF),
    LR(LambertianBRDF),
    MR(MicrofacetBRDF),
    MS(MicrofacetBSDF),
    SR(SpecularBRDF),
    SS(SpecularBSDF),
}

impl BSDF {
    pub fn fresnel_blend_brdf(diffuse: Color, specular: Color, shininess: Float) -> Self {
        BSDF::FBR(FresnelBlendBRDF::new(diffuse, specular, shininess))
    }

    pub fn lambertian_brdf(color: Color) -> Self {
        BSDF::LR(LambertianBRDF::new(color))
    }

    pub fn microfacet_brdf(color: Color, shininess: Float) -> Self {
        BSDF::MR(MicrofacetBRDF::with_schlick(color, shininess))
    }

    pub fn microfacet_bsdf(reflect: Color, transmit: Color, shininess: Float, eta: Float) -> Self {
        BSDF::MS(MicrofacetBSDF::new(reflect, transmit, shininess, eta))
    }

    pub fn specular_brdf(color: Color) -> Self {
        BSDF::SR(SpecularBRDF::with_schlick(color))
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
            FBR(inner) => inner,
            LR(inner) => inner,
            MR(inner) => inner,
            MS(inner) => inner,
            SR(inner) => inner,
            SS(inner) => inner,
        }
    }
}
