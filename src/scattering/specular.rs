use cgmath::Point2;

use crate::bsdf::BSDF;
use crate::float::*;
use crate::texture::Texture;

use super::ScatteringT;

#[derive(Debug)]
pub struct SpecularReflection {
    texture: Texture,
}

impl SpecularReflection {
    pub fn new(texture: Texture) -> Self {
        Self { texture }
    }
}

impl ScatteringT for SpecularReflection {
    fn local(&self, tex_coords: Point2<Float>) -> BSDF {
        BSDF::specular_brdf(self.texture.color(tex_coords))
    }

    fn preview_texture(&self) -> &Texture {
        &self.texture
    }
}

/// Fresnel modulated reflection and transmission
#[derive(Debug)]
pub struct SpecularTransmission {
    reflective: Texture,
    transmissive: Texture,
    eta: Float,
}

impl SpecularTransmission {
    pub fn new(reflective: Texture, transmissive: Texture, eta: Float) -> Self {
        Self {
            reflective,
            transmissive,
            eta,
        }
    }
}

impl ScatteringT for SpecularTransmission {
    fn local(&self, tex_coords: Point2<Float>) -> BSDF {
        let reflect = self.reflective.color(tex_coords);
        let transmit = self.transmissive.color(tex_coords);
        let eta = self.eta;
        BSDF::specular_bsdf(reflect, transmit, eta)
    }

    fn preview_texture(&self) -> &Texture {
        &self.transmissive
    }
}
