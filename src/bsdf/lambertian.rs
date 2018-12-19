use cgmath::Vector3;

use crate::color::Color;
use crate::consts;
use crate::Float;

use super::BSDFT;

#[derive(Debug)]
pub struct LambertianBRDF {
    color: Color,
}

impl LambertianBRDF {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

impl BSDFT for LambertianBRDF {
    fn is_specular(&self) -> bool {
        false
    }

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
