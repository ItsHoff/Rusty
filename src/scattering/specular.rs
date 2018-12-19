use cgmath::Point2;

use crate::bsdf::BSDF;
use crate::color::Color;
use crate::obj_load;
use crate::texture::Texture;
use crate::Float;

use super::ScatteringT;

#[derive(Debug)]
pub struct SpecularReflection {
    texture: Texture,
}

impl SpecularReflection {
    // TODO: use specular color to figure out fresnel coeffs or use Schlick
    pub fn new(obj_mat: &obj_load::Material) -> Self {
        let texture = match &obj_mat.tex_specular {
            Some(path) => Texture::from_image_path(path),
            None => {
                let color = Color::from(obj_mat.c_specular.unwrap());
                Texture::from_color(color)
            }
        };
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
    pub fn new(obj_mat: &obj_load::Material) -> Self {
        let filter = Color::from(
            obj_mat
                .c_translucency
                .expect("No translucent color for translucent material"),
        );
        // TODO: not sure if which is the correct interpretation
        // or if it is even scene dependant
        // let color = Color::white() - filter;
        let color = filter;
        let texture = Texture::from_color(color);
        let eta = obj_mat
            .refraction_i
            .expect("No index of refraction for translucent material")
            .into();
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
    pub fn new(obj_mat: &obj_load::Material) -> Self {
        Self {
            reflection: SpecularReflection::new(obj_mat),
            transmission: SpecularTransmission::new(obj_mat),
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
