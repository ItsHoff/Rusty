use cgmath::Point2;

use crate::bsdf::BSDF;
use crate::color::Color;
use crate::obj_load;
use crate::texture::Texture;
use crate::Float;

use super::ScatteringT;

#[derive(Debug)]
pub struct DiffuseReflection {
    texture: Texture,
}

impl DiffuseReflection {
    pub fn new(obj_mat: &obj_load::Material) -> Self {
        let texture = match &obj_mat.tex_diffuse {
            Some(path) => Texture::from_image_path(path),
            None => {
                let color = Color::from(obj_mat.c_diffuse.unwrap());
                Texture::from_color(color)
            }
        };
        Self { texture }
    }
}

impl ScatteringT for DiffuseReflection {
    fn local(&self, tex_coords: Point2<Float>) -> BSDF {
        BSDF::lambertian_brdf(self.texture.color(tex_coords))
    }

    fn preview_texture(&self) -> &Texture {
        &self.texture
    }
}
