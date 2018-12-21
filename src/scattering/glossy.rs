use cgmath::Point2;

use crate::bsdf::BSDF;
use crate::float::*;
use crate::texture::Texture;

use super::ScatteringT;

#[derive(Debug)]
pub struct GlossyReflection {
    texture: Texture,
    shininess: Float,
}

impl GlossyReflection {
    pub fn new(texture: Texture, shininess: Float) -> Self {
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
