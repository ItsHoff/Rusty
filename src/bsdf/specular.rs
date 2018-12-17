use cgmath::prelude::*;
use cgmath::{Point2, Vector3};

use crate::color::Color;
use crate::obj_load;
use crate::texture::Texture;
use crate::Float;

use super::{ShadingModelT, BSDF, BSDFT};

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

impl ShadingModelT for SpecularReflection {
    fn bsdf(&self, tex_coords: Point2<Float>) -> BSDF {
        BSDF::SR(SpecularBRDF::new(self.texture.color(tex_coords)))
    }

    fn preview_texture(&self) -> &Texture {
        &self.texture
    }
}

pub struct SpecularBRDF {
    color: Color,
}

impl SpecularBRDF {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

impl BSDFT for SpecularBRDF {
    fn eval(&self, _in_dir: Vector3<Float>, _out_dir: Vector3<Float>) -> Color {
        Color::black()
    }

    fn sample(&self, out_dir: Vector3<Float>) -> (Color, Vector3<Float>, Float) {
        let in_dir = Vector3::new(-out_dir.x, -out_dir.y, out_dir.z);
        (self.color, in_dir, 1.0)
    }
}

#[derive(Debug)]
pub struct SpecularTransmission {
    texture: Texture,
    eta: Float,
}

impl SpecularTransmission {
    pub fn new(obj_mat: &obj_load::Material) -> Self {
        let color = Color::from(
            obj_mat
                .c_translucency
                .expect("No translucent color for translucent material"),
        );
        let texture = Texture::from_color(color);
        let eta = obj_mat
            .refraction_i
            .expect("No index of refraction for translucent material")
            .into();
        Self { texture, eta }
    }
}

impl ShadingModelT for SpecularTransmission {
    fn bsdf(&self, tex_coords: Point2<Float>) -> BSDF {
        BSDF::ST(SpecularBTDF::new(self.texture.color(tex_coords), self.eta))
    }

    fn preview_texture(&self) -> &Texture {
        &self.texture
    }
}

pub struct SpecularBTDF {
    color: Color,
    eta: Float,
}

impl SpecularBTDF {
    pub fn new(color: Color, eta: Float) -> Self {
        Self { color, eta }
    }
}

impl BSDFT for SpecularBTDF {
    fn eval(&self, _in_dir: Vector3<Float>, _out_dir: Vector3<Float>) -> Color {
        Color::black()
    }

    fn sample(&self, out_dir: Vector3<Float>) -> (Color, Vector3<Float>, Float) {
        let (n, eta) = if out_dir.z > 0.0 {
            (Vector3::unit_z(), 1.0 / self.eta)
        } else {
            (-Vector3::unit_z(), self.eta)
        };
        let cos_i = out_dir.dot(n);
        let sin2_i = (1.0 - cos_i.powi(2)).max(0.0);
        let sin2_t = eta.powi(2) * sin2_i;
        // Total internal reflection
        if sin2_t >= 1.0 {
            return (Color::black(), Vector3::unit_z(), 1.0);
        }
        let cos_t = (1.0 - sin2_t).sqrt();
        let in_dir = -out_dir * eta + (eta * cos_i - cos_t) * n;
        (self.color / in_dir.z.abs(), in_dir, 1.0)
    }
}
