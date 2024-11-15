use cgmath::Point2;

use crate::bsdf::Bsdf;
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
    fn local(&self, tex_coords: Point2<Float>) -> Bsdf {
        Bsdf::microfacet_brdf(self.texture.color(tex_coords), self.shininess)
    }

    fn preview_texture(&self) -> &Texture {
        &self.texture
    }
}

#[derive(Debug)]
pub struct GlossyBlend {
    diffuse: Texture,
    specular: Texture,
    shininess: Float,
}

impl GlossyBlend {
    pub fn new(diffuse: Texture, specular: Texture, shininess: Float) -> Self {
        Self {
            diffuse,
            specular,
            shininess,
        }
    }
}

impl ScatteringT for GlossyBlend {
    fn local(&self, tex_coords: Point2<Float>) -> Bsdf {
        let diffuse = self.diffuse.color(tex_coords);
        let specular = self.specular.color(tex_coords);
        Bsdf::fresnel_blend_brdf(diffuse, specular, self.shininess)
    }

    fn preview_texture(&self) -> &Texture {
        &self.diffuse
    }
}

#[derive(Debug)]
pub struct GlossyTransmission {
    reflective: Texture,
    transmissive: Texture,
    shininess: Float,
    eta: Float,
}

impl GlossyTransmission {
    pub fn new(reflective: Texture, transmissive: Texture, shininess: Float, eta: Float) -> Self {
        if (eta - 1.0).abs() < crate::consts::EPSILON {
            println!(
                "IOR is almost one ({:?}). Specular bsdf should be used instead of glossy.",
                eta
            );
        }
        Self {
            reflective,
            transmissive,
            shininess,
            eta,
        }
    }
}

impl ScatteringT for GlossyTransmission {
    fn local(&self, tex_coords: Point2<Float>) -> Bsdf {
        let reflect = self.reflective.color(tex_coords);
        let transmit = self.transmissive.color(tex_coords);
        Bsdf::microfacet_bsdf(reflect, transmit, self.shininess, self.eta)
    }

    fn preview_texture(&self) -> &Texture {
        &self.transmissive
    }
}
