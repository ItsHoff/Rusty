use cgmath::prelude::*;
use cgmath::Vector3;

use crate::color::Color;
use crate::consts;
use crate::Float;

use super::util;
use super::BSDFT;

#[derive(Debug)]
pub struct MicrofacetBRDF {
    color: Color,
    microfacets: GGX,
}

impl MicrofacetBRDF {
    pub fn new(color: Color, shininess: Float) -> Self {
        Self {
            color,
            microfacets: GGX::from_shininess(shininess),
        }
    }

    fn g(&self, in_dir: Vector3<Float>, out_dir: Vector3<Float>) -> Float {
        let l1 = self.microfacets.lambda(in_dir);
        let l2 = self.microfacets.lambda(out_dir);
        1.0 / (1.0 + l1 + l2)
    }
}

impl BSDFT for MicrofacetBRDF {
    fn is_specular(&self) -> bool {
        false
    }

    fn eval(&self, in_dir: Vector3<Float>, out_dir: Vector3<Float>) -> Color {
        let g = self.g(in_dir, out_dir);
        let half = (in_dir + out_dir).normalize();
        let d = self.microfacets.d(half);
        let denom = 4.0 * in_dir.z * out_dir.z;
        self.color * d * g / denom
    }

    fn sample(&self, out_dir: Vector3<Float>) -> (Color, Vector3<Float>, Float) {
        let half = self.microfacets.sample_half(out_dir);
        // Reflect around half vector
        let in_dir = -out_dir + 2.0 * half.dot(out_dir) * half;
        // Reflected direction not in the same hemisphere
        if out_dir.z * in_dir.z < 0.0 {
            return (Color::black(), Vector3::unit_z(), 1.0);
        }
        let pdf = self.microfacets.pdf_half(out_dir, half) / (4.0 * out_dir.dot(half).abs());
        let val = self.eval(in_dir, out_dir);
        (val, in_dir, pdf)
    }
}

/// GGX (Trowbridge-Reitz) microfacet distribution
#[derive(Debug)]
struct GGX {
    alpha: Float,
}

// TODO: maybe just keep alpha^2
impl GGX {
    fn from_shininess(shininess: Float) -> Self {
        // Shininess to alpha conversion from
        // http://graphicrants.blogspot.com/2013/08/specular-brdf-reference.html
        Self {
            alpha: (2.0 / (shininess + 2.0)).sqrt(),
        }
    }

    fn d(&self, half: Vector3<Float>) -> Float {
        let cos2_t = util::cos2_t(half);
        let a2 = self.alpha.powi(2);
        let denom = consts::PI * (cos2_t * (a2 - 1.0) + 1.0).powi(2);
        a2 / denom
    }

    fn lambda(&self, dir: Vector3<Float>) -> Float {
        let a2 = self.alpha.powi(2);
        let tan2_t = util::tan2_t(dir);
        ((1.0 + a2 * tan2_t).sqrt() - 1.0) / 2.0
    }

    // https://agraphicsguy.wordpress.com/2015/11/01/sampling-microfacet-brdf/
    // TODO: Take shadowing into account
    fn sample_half(&self, _out_dir: Vector3<Float>) -> Vector3<Float> {
        let phi = 2.0 * consts::PI * rand::random::<Float>();
        let r1 = rand::random::<Float>();
        let a2 = self.alpha.powi(2);
        let cos2_t = (1.0 - r1) / (r1 * (a2 - 1.0) + 1.0);
        let sin_t = (1.0 - cos2_t).sqrt();
        let x = sin_t * phi.cos();
        let y = sin_t * phi.sin();
        let z = cos2_t.sqrt();
        Vector3::new(x, y, z)
    }

    fn pdf_half(&self, _out_dir: Vector3<Float>, half: Vector3<Float>) -> Float {
        self.d(half) * half.z
    }
}
