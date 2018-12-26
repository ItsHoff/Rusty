use cgmath::prelude::*;
use cgmath::Vector3;

use crate::color::Color;
use crate::consts;
use crate::float::*;

use super::fresnel;
use super::util;
use super::BSDFT;

#[derive(Debug)]
pub struct MicrofacetBRDF {
    color: Color,
    microfacets: GGX,
    use_schlick: bool,
}

impl MicrofacetBRDF {
    pub fn with_schlick(color: Color, shininess: Float) -> Self {
        Self {
            color,
            microfacets: GGX::from_shininess(shininess),
            use_schlick: true,
        }
    }

    // TODO: Implement microfacet bsdf
    // pub fn without_schlick(color: Color, shininess: Float) -> Self {
    //     Self {
    //         color,
    //         microfacets: GGX::from_shininess(shininess),
    //         use_schlick: false,
    //     }
    // }

    fn g(&self, wo: Vector3<Float>, wi: Vector3<Float>) -> Float {
        let l1 = self.microfacets.lambda(wo);
        let l2 = self.microfacets.lambda(wi);
        1.0 / (1.0 + l1 + l2)
    }
}

impl BSDFT for MicrofacetBRDF {
    fn is_specular(&self) -> bool {
        false
    }

    fn brdf(&self, wo: Vector3<Float>, wi: Vector3<Float>) -> Color {
        let g = self.g(wo, wi);
        let wh = (wo + wi).normalize();
        let d = self.microfacets.d_wh(wh);
        let denom = 4.0 * wo.z * wi.z;
        let color = if self.use_schlick {
            fresnel::schlick(wo, self.color)
        } else {
            self.color
        };
        color * d * g / denom
    }

    fn btdf(&self, _wo: Vector3<Float>, _wi: Vector3<Float>) -> Color {
        Color::black()
    }

    fn sample(&self, wo: Vector3<Float>) -> Option<(Color, Vector3<Float>, Float)> {
        let wh = self.microfacets.sample_wh(wo);
        let wi = util::reflect(wo, wh);
        if !util::same_hemisphere(wo, wi) {
            return None;
        }
        let pdf = self.microfacets.pdf_wh(wo, wh) / (4.0 * wo.dot(wh).abs());
        let val = self.brdf(wo, wi);
        Some((val, wi, pdf))
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

    fn d_wh(&self, wh: Vector3<Float>) -> Float {
        let cos2_t = util::cos2_t(wh);
        let a2 = self.alpha.powi(2);
        let denom = consts::PI * (cos2_t * (a2 - 1.0) + 1.0).powi(2);
        a2 / denom
    }

    fn lambda(&self, w: Vector3<Float>) -> Float {
        let a2 = self.alpha.powi(2);
        let tan2_t = util::tan2_t(w);
        ((1.0 + a2 * tan2_t).sqrt() - 1.0) / 2.0
    }

    // https://agraphicsguy.wordpress.com/2015/11/01/sampling-microfacet-brdf/
    // TODO: Take shadowing into account
    fn sample_wh(&self, _wo: Vector3<Float>) -> Vector3<Float> {
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

    fn pdf_wh(&self, _wo: Vector3<Float>, wh: Vector3<Float>) -> Float {
        self.d_wh(wh) * util::cos_t(wh)
    }
}
