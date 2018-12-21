use cgmath::Point2;

use crate::bsdf::BSDF;
use crate::float::*;
use crate::texture::Texture;

use super::ScatteringT;

#[derive(Debug)]
pub struct DiffuseReflection {
    texture: Texture,
}

impl DiffuseReflection {
    pub fn new(texture: Texture) -> Self {
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
