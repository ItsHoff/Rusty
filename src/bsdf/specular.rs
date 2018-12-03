use cgmath::{Point2, Vector3};

use crate::color::Color;
use crate::obj_load;
use crate::texture::Texture;
use crate::Float;

use super::{BSDF, ShadingModel};

#[derive(Debug)]
pub struct SpecularModel {
    color: Texture,
}

impl SpecularModel {
    // TODO: use specular color to figure out fresnel coeffs or use Schlick
    pub fn new(obj_mat: &obj_load::Material) -> Self {
        let color = match &obj_mat.tex_specular {
            Some(path) => Texture::from_image_path(path),
            None => {
                let color = Color::from(obj_mat.c_specular.unwrap());
                Texture::from_color(color)
            }
        };
        Self { color }
    }
}

impl ShadingModel for SpecularModel {
    fn bsdf(&self, tex_coords: Point2<Float>) -> Box<dyn BSDF> {
        Box::new(SpecularBRDF::new(self.color.color(tex_coords)))
    }

    fn preview_texture(&self) -> &Texture {
        &self.color
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

impl BSDF for SpecularBRDF {
    fn eval(&self, _in_dir: Vector3<Float>, _out_dir: Vector3<Float>) -> Color {
        Color::black()
    }

    fn sample(&self, out_dir: Vector3<Float>) -> (Color, Vector3<Float>, Float) {
        let in_dir = Vector3::new(-out_dir.x, -out_dir.y, out_dir.z);
        (self.color, in_dir, 1.0)
    }
}
