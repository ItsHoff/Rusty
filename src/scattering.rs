use std::ops::Deref;

use cgmath::Point2;

use crate::bsdf::BSDF;
use crate::obj_load;
use crate::texture::Texture;
use crate::Float;

mod diffuse;
mod specular;

use self::diffuse::*;
use self::specular::*;

/// Scattering model over the whole surface
pub trait ScatteringT {
    /// Get the local scattering functions
    fn local(&self, tex_coords: Point2<Float>) -> BSDF;
    /// The texture to use for preview rendering
    fn preview_texture(&self) -> &Texture;
}

#[derive(Debug)]
pub enum Scattering {
    DR(DiffuseReflection),
    SR(SpecularReflection),
    F(FresnelSpecular),
}

impl Scattering {
    pub fn from_obj(obj_mat: &obj_load::Material) -> Self {
        use self::Scattering::*;

        match obj_mat.illumination_model.unwrap_or(0) {
            5 => SR(SpecularReflection::new(obj_mat)),
            4 | 6 | 7 => F(FresnelSpecular::new(obj_mat)),
            i => {
                if i > 10 {
                    println!("Illumination model {} is not defined in the mtl spec!", i);
                    println!("Defaulting to diffuse BSDF.");
                }
                DR(DiffuseReflection::new(obj_mat))
            }
        }
    }
}

impl Deref for Scattering {
    type Target = dyn ScatteringT;

    fn deref(&self) -> &Self::Target {
        use self::Scattering::*;
        match self {
            DR(inner) => inner,
            SR(inner) => inner,
            F(inner) => inner,
        }
    }
}
