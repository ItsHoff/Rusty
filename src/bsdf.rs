use std::ops::Deref;

use cgmath::{Point2, Vector3};

use crate::color::Color;
use crate::obj_load;
use crate::texture::Texture;
use crate::Float;

mod lambertian;
mod specular;

use self::lambertian::*;
use self::specular::*;

/// Shading model over the whole surface
pub trait ShadingModelT: std::fmt::Debug + Send + Sync {
    fn bsdf(&self, tex_coords: Point2<Float>) -> BSDF;
    fn preview_texture(&self) -> &Texture;
}

#[derive(Debug)]
pub enum ShadingModel {
    LR(LambertianReflection),
    SR(SpecularReflection),
    ST(SpecularTransmission),
}

impl ShadingModel {
    pub fn from_obj(obj_mat: &obj_load::Material) -> Self {
        use self::ShadingModel::*;

        match obj_mat.illumination_model.unwrap_or(0) {
            5 => SR(SpecularReflection::new(obj_mat)),
            7 => ST(SpecularTransmission::new(obj_mat)),
            i => {
                if i > 10 {
                    println!("Illumination model {} is not defined in the mtl spec!", i);
                    println!("Defaulting to diffuse BSDF.");
                }
                LR(LambertianReflection::new(obj_mat))
            }
        }
    }
}

impl Deref for ShadingModel {
    type Target = dyn ShadingModelT;

    fn deref(&self) -> &Self::Target {
        use self::ShadingModel::*;
        match self {
            LR(inner) => inner,
            SR(inner) => inner,
            ST(inner) => inner,
        }
    }
}

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
}

impl Deref for BSDF {
    type Target = dyn BSDFT;

    fn deref(&self) -> &Self::Target {
        use self::BSDF::*;
        match self {
            LR(inner) => inner,
            SR(inner) => inner,
            ST(inner) => inner,
        }
    }
}
