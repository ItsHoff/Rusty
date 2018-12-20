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

    fn eval(&self, _wo: Vector3<Float>, _wi: Vector3<Float>) -> Color {
        self.color / consts::PI
    }

    fn sample(&self, wo: Vector3<Float>) -> Option<(Color, Vector3<Float>, Float)> {
        let phi = 2.0 * consts::PI * rand::random::<Float>();
        let r = rand::random::<Float>().sqrt();
        let x = r * phi.cos();
        let y = r * phi.sin();
        let mut z = (1.0 - r.powi(2)).sqrt();
        let pdf = z / consts::PI;
        if wo.z < 0.0 {
            z *= -1.0;
        }
        let wi = Vector3::new(x, y, z);
        let val = self.eval(wo, wi);
        Some((val, wi, pdf))
    }
}
