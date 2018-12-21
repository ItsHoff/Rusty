use cgmath::Point2;

use crate::bsdf::BSDF;
use crate::color::Color;
use crate::float::*;
use crate::obj_load;
use crate::texture::Texture;

use super::ScatteringT;

#[derive(Debug)]
pub struct GlossyReflection {
    texture: Texture,
    shininess: Float,
}

impl GlossyReflection {
    pub fn new(obj_mat: &obj_load::Material) -> Self {
        // TODO: Specular and diffuse components
        let texture = match &obj_mat.tex_diffuse {
            Some(path) => Texture::from_image_path(path),
            None => {
                let color = Color::from(obj_mat.c_diffuse.unwrap());
                Texture::from_color(color)
            }
        };
        let shininess = obj_mat.shininess.unwrap().to_float();
        Self { texture, shininess }
    }
}

impl ScatteringT for GlossyReflection {
    fn local(&self, tex_coords: Point2<Float>) -> BSDF {
        BSDF::microfacet_brdf(self.texture.color(tex_coords), self.shininess)
    }

    fn preview_texture(&self) -> &Texture {
        &self.texture
    }
}
