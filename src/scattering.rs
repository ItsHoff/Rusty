use std::ops::Deref;

use cgmath::Point2;

use crate::bsdf::Bsdf;
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
    fn local(&self, tex_coords: Point2<Float>) -> Bsdf;
    /// The texture to use for preview rendering
    fn preview_texture(&self) -> &Texture;
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum Scattering {
    Dr(DiffuseReflection),
    Gb(GlossyBlend),
    Gr(GlossyReflection),
    Gt(GlossyTransmission),
    Sr(SpecularReflection),
    St(SpecularTransmission),
}

fn diffuse_texture(obj_mat: &obj_load::Material) -> Texture {
    match &obj_mat.diffuse_texture {
        Some(path) => Texture::from_image_path(path),
        None => {
            let color = Color::from(obj_mat.diffuse_color.unwrap_or([0.0, 0.0, 0.0]));
            Texture::from_color(color)
        }
    }
}

fn specular_texture(obj_mat: &obj_load::Material) -> Texture {
    match &obj_mat.specular_texture {
        Some(path) => Texture::from_image_path(path),
        None => {
            let color = Color::from(obj_mat.specular_color.unwrap_or([0.0, 0.0, 0.0]));
            Texture::from_color(color)
        }
    }
}

fn transmission_filter(obj_mat: &obj_load::Material) -> Texture {
    let mut color = Color::from(
        obj_mat
            .transmission_filter
            .expect("No transmission filter for transmissive material"),
    );
    // MTL spec states that transmission filter defines the fraction of light
    // that is able to pass through the surface, but some scenes seem to interpret
    // it as the opposite. So we flip all low valued filters.
    if color.r() < 0.4 && color.g() < 0.4 && color.b() < 0.4 {
        println!("Flipped transmission filter!");
        color = Color::white() - color;
    }
    Texture::from_color(color)
}

impl Scattering {
    pub fn from_obj(obj_mat: &obj_load::Material) -> Self {
        use self::Scattering::*;

        let diffuse = diffuse_texture(obj_mat);
        let specular = specular_texture(obj_mat);
        match obj_mat.illumination_model {
            Some(2) => {
                let exponent = obj_mat.specular_exponent.map(ToFloat::to_float);
                if diffuse.is_black() {
                    Gr(GlossyReflection::new(specular, exponent.unwrap()))
                } else if specular.is_black() {
                    Dr(DiffuseReflection::new(diffuse))
                } else {
                    Gb(GlossyBlend::new(diffuse, specular, exponent.unwrap()))
                }
            }
            Some(5) => {
                let texture = specular_texture(obj_mat);
                Sr(SpecularReflection::new(texture))
            }
            Some(4) | Some(9) => {
                let filter = transmission_filter(obj_mat);
                let eta = obj_mat
                    .index_of_refraction
                    .expect("No index of refraction for translucent material")
                    .to_float();
                St(SpecularTransmission::new(specular, filter, eta))
            }
            Some(6) | Some(7) => {
                let filter = transmission_filter(obj_mat);
                let eta = obj_mat
                    .index_of_refraction
                    .expect("No index of refraction for translucent material")
                    .to_float();
                if (eta - 1.0).abs() < crate::consts::EPSILON {
                    // Glossy does not handle eta = 1 properly
                    // and the distribution would be the same anyways
                    St(SpecularTransmission::new(specular, filter, eta))
                } else {
                    let exponent = obj_mat.specular_exponent.unwrap().to_float();
                    Gt(GlossyTransmission::new(specular, filter, exponent, eta))
                }
            }
            Some(i) => {
                if i > 10 {
                    println!("Illumination model {} is not defined in the mtl spec!", i);
                    println!("Defaulting to diffuse reflection.");
                } else if i != 1 {
                    println!("Unimplemented illumination model {}!", i);
                    println!("Defaulting to diffuse reflection.");
                }
                Dr(DiffuseReflection::new(diffuse))
            }
            None => Dr(DiffuseReflection::new(diffuse)),
        }
    }
}

impl Deref for Scattering {
    type Target = dyn ScatteringT;

    fn deref(&self) -> &Self::Target {
        use self::Scattering::*;
        match self {
            Dr(inner) => inner,
            Gb(inner) => inner,
            Gr(inner) => inner,
            Gt(inner) => inner,
            Sr(inner) => inner,
            St(inner) => inner,
        }
    }
}
