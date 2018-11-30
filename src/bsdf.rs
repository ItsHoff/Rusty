use cgmath::Vector3;

use crate::consts;
use crate::color::Color;
use crate::Float;

/// Trait for handling local light transport.
/// in_dir and out_dir refer to the direction of the photons.
/// Directions should be given in a surface local coordinate system,
/// where (0, 0, 1) is the normal
pub trait BSDF {
    fn eval(&self, in_dir: Vector3<Float>, out_dir: Vector3<Float>) -> Color;
    fn sample(&self, out_dir: Vector3<Float>) -> (Color, Vector3<Float>, Float);
}

pub struct LambertianBRDF {
    diffuse: Color,
}

impl LambertianBRDF {
    pub fn new(diffuse: Color) -> Self {
        Self { diffuse }
    }
}


impl BSDF for LambertianBRDF {
    fn eval(&self, _in_dir: Vector3<Float>, _out_dir: Vector3<Float>) -> Color {
        self.diffuse / consts::PI
    }

    fn sample(&self, out_dir: Vector3<Float>) -> (Color, Vector3<Float>, Float) {
        let angle = 2.0 * consts::PI * rand::random::<Float>();
        let length = rand::random::<Float>().sqrt();
        let x = length * angle.cos();
        let y = length * angle.sin();
        let z = (1.0 - length.powi(2)).sqrt();
        let in_dir = Vector3::new(x, y, z);
        let val = self.eval(in_dir, out_dir);
        let pdf = z / consts::PI;
        (val, in_dir, pdf)
    }
}
