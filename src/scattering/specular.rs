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

#[derive(Debug)]
pub struct SpecularTransmission {
    texture: Texture,
    eta: Float,
}

impl SpecularTransmission {
    pub fn new(texture: Texture, eta: Float) -> Self {
        Self { texture, eta }
    }
}

impl ScatteringT for SpecularTransmission {
    fn local(&self, tex_coords: Point2<Float>) -> BSDF {
        BSDF::specular_btdf(self.texture.color(tex_coords), self.eta)
    }

    fn preview_texture(&self) -> &Texture {
        &self.texture
    }
}

/// Fresnel modulated reflection and transmission
#[derive(Debug)]
pub struct FresnelSpecular {
    reflection: SpecularReflection,
    transmission: SpecularTransmission,
}

impl FresnelSpecular {
    pub fn new(specular: Texture, transmissive: Texture, eta: Float) -> Self {
        Self {
            reflection: SpecularReflection::new(specular),
            transmission: SpecularTransmission::new(transmissive, eta),
        }
    }
}

impl ScatteringT for FresnelSpecular {
    fn local(&self, tex_coords: Point2<Float>) -> BSDF {
        let reflect = self.reflection.texture.color(tex_coords);
        let transmit = self.transmission.texture.color(tex_coords);
        let eta = self.transmission.eta;
        BSDF::fresnel_specular(reflect, transmit, eta)
    }

    fn preview_texture(&self) -> &Texture {
        &self.transmission.texture
    }
}
