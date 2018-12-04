use cgmath::{Point2, Vector3};

use crate::color::Color;
use crate::obj_load;
use crate::texture::Texture;
use crate::Float;

mod lambertian;
mod specular;

use self::lambertian::*;
use self::specular::*;

pub fn model_from_obj(obj_mat: &obj_load::Material) -> Box<dyn ShadingModel> {
    match obj_mat.illumination_model.unwrap_or(0) {
        5 => {
            Box::new(SpecularReflection::new(obj_mat))
        },
        7 => {
            Box::new(SpecularTransmission::new(obj_mat))
        }
        i => {
            if i > 10 {
                println!("Illumination model {} is not defined in the mtl spec!", i);
                println!("Defaulting to diffuse BSDF.");
            }
            Box::new(LambertianReflection::new(obj_mat))
        }
    }
}

/// Shading model over the whole surface
pub trait ShadingModel: std::fmt::Debug + Send + Sync {
    fn bsdf(&self, tex_coords: Point2<Float>) -> Box<dyn BSDF>;
    fn preview_texture(&self) -> &Texture;
}

/// Trait for handling local light transport.
/// Directions should both point away from the intersection.
/// in_dir corresponds to the direction photons arrive from and
/// out_dir refer to the direction of the photons scatter towards.
/// Directions should be given in a surface local coordinate system,
/// where (0, 0, 1) is the normal
pub trait BSDF {
    fn eval(&self, in_dir: Vector3<Float>, out_dir: Vector3<Float>) -> Color;
    fn sample(&self, out_dir: Vector3<Float>) -> (Color, Vector3<Float>, Float);
}
