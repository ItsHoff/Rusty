use cgmath::{Point2, Vector3};

use crate::color::Color;
use crate::consts;
use crate::obj_load;
use crate::texture::Texture;
use crate::Float;

use super::{ShadingModel, BSDF};

#[derive(Debug)]
pub struct LambertianReflection {
    texture: Texture,
}

impl LambertianReflection {
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

impl ShadingModel for LambertianReflection {
    fn bsdf(&self, tex_coords: Point2<Float>) -> Box<dyn BSDF> {
        Box::new(LambertianBRDF::new(self.texture.color(tex_coords)))
    }

    fn preview_texture(&self) -> &Texture {
        &self.texture
    }
}

pub struct LambertianBRDF {
    color: Color,
}

impl LambertianBRDF {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

impl BSDF for LambertianBRDF {
    fn eval(&self, _in_dir: Vector3<Float>, _out_dir: Vector3<Float>) -> Color {
        self.color / consts::PI
    }

    fn sample(&self, out_dir: Vector3<Float>) -> (Color, Vector3<Float>, Float) {
        let angle = 2.0 * consts::PI * rand::random::<Float>();
        let length = rand::random::<Float>().sqrt();
        let x = length * angle.cos();
        let y = length * angle.sin();
        let mut z = (1.0 - length.powi(2)).sqrt();
        let pdf = z / consts::PI;
        if out_dir.z < 0.0 {
            z *= -1.0;
        }
        let in_dir = Vector3::new(x, y, z);
        let val = self.eval(in_dir, out_dir);
        (val, in_dir, pdf)
    }
}
