use std::ops::Deref;

use cgmath::Point2;

use crate::bsdf::BSDF;
use crate::color::Color;
use crate::float::*;
use crate::obj_load;
use crate::texture::Texture;

mod diffuse;
mod glossy;
mod specular;

use self::diffuse::*;
use self::glossy::*;
use self::specular::*;

/// Scattering model over the whole surface
pub trait ScatteringT {
    /// Get the local scattering functions
    fn local(&self, tex_coords: Point2<Float>) -> BSDF;
    /// The texture to use for preview rendering
    fn preview_texture(&self) -> &Texture;
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum Scattering {
    DR(DiffuseReflection),
    GB(GlossyBlend),
    GR(GlossyReflection),
    GT(GlossyTransmission),
    SR(SpecularReflection),
    ST(SpecularTransmission),
}

fn diffuse_texture(obj_mat: &obj_load::Material) -> Texture {
    match &obj_mat.tex_diffuse {
        Some(path) => Texture::from_image_path(path),
        None => {
            let color = Color::from(obj_mat.c_diffuse.unwrap_or([0.0, 0.0, 0.0]));
            Texture::from_color(color)
        }
    }
}

fn specular_texture(obj_mat: &obj_load::Material) -> Texture {
    match &obj_mat.tex_specular {
        Some(path) => Texture::from_image_path(path),
        None => {
            let color = Color::from(obj_mat.c_specular.unwrap_or([0.0, 0.0, 0.0]));
            Texture::from_color(color)
        }
    }
}

fn transmissive_texture(obj_mat: &obj_load::Material) -> Texture {
    let color = Color::from(
        obj_mat
            .c_translucency
            .expect("No translucent color for translucent material"),
    );
    // MTL spec states that transmissive color defines light that is able to pass through
    // but some scenes seem to interpret it as a filter that removes light
    let color = Color::white() - color;
    Texture::from_color(color)
}

impl Scattering {
    pub fn from_obj(obj_mat: &obj_load::Material) -> Self {
        use self::Scattering::*;

        let diffuse = diffuse_texture(obj_mat);
        let specular = specular_texture(obj_mat);
        match obj_mat.illumination_model.unwrap_or(0) {
            2 => {
                let shininess = obj_mat.shininess.unwrap().to_float();
                if diffuse.is_black() {
                    GR(GlossyReflection::new(specular, shininess))
                } else if specular.is_black() {
                    DR(DiffuseReflection::new(diffuse))
                } else {
                    GB(GlossyBlend::new(diffuse, specular, shininess))
                }
            }
            5 => {
                let texture = specular_texture(obj_mat);
                SR(SpecularReflection::new(texture))
            }
            4 | 6 | 7 => {
                let transmissive = transmissive_texture(obj_mat);
                let shininess = obj_mat.shininess.unwrap().to_float();
                let eta = obj_mat
                    .refraction_i
                    .expect("No index of refraction for translucent material")
                    .to_float();
                if (eta - 1.0) < crate::consts::EPSILON {
                    // Glossy does not handle eta = 1 properly
                    // and the distribution would be the same anyways
                    ST(SpecularTransmission::new(specular, transmissive, eta))
                } else {
                    GT(GlossyTransmission::new(
                        specular,
                        transmissive,
                        shininess,
                        eta,
                    ))
                }
            }
            i => {
                if i > 10 {
                    println!("Illumination model {} is not defined in the mtl spec!", i);
                    println!("Defaulting to diffuse BSDF.");
                }
                DR(DiffuseReflection::new(diffuse))
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
            GB(inner) => inner,
            GR(inner) => inner,
            GT(inner) => inner,
            SR(inner) => inner,
            ST(inner) => inner,
        }
    }
}
