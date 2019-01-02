use cgmath::prelude::*;
use cgmath::Vector3;

use crate::color::Color;
use crate::consts;
use crate::float::*;
use crate::sample;

use super::fresnel::{self, FresnelBSDF};
use super::util;
use super::BSDFT;

/// GGX (Trowbridge-Reitz) microfacet distribution
#[derive(Clone, Debug)]
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

    fn g(&self, wo: Vector3<Float>, wi: Vector3<Float>) -> Float {
        let l1 = self.lambda(wo);
        let l2 = self.lambda(wi);
        1.0 / (1.0 + l1 + l2)
    }

    fn lambda(&self, w: Vector3<Float>) -> Float {
        let a2 = self.alpha.powi(2);
        let tan2_t = util::tan2_t(w);
        ((1.0 + a2 * tan2_t).sqrt() - 1.0) / 2.0
    }

    // https://agraphicsguy.wordpress.com/2015/11/01/sampling-microfacet-brdf/
    // TODO: Take shadowing into account
    fn sample_wh(&self, wo: Vector3<Float>) -> Vector3<Float> {
        let phi = 2.0 * consts::PI * rand::random::<Float>();
        let r1 = rand::random::<Float>();
        let a2 = self.alpha.powi(2);
        let cos2_t = (1.0 - r1) / (r1 * (a2 - 1.0) + 1.0);
        let sin_t = (1.0 - cos2_t).sqrt();
        let x = sin_t * phi.cos();
        let y = sin_t * phi.sin();
        let z = cos2_t.sqrt();
        let wh = Vector3::new(x, y, z);
        if util::same_hemisphere(wo, wh) {
            wh
        } else {
            -wh
        }
    }

    fn pdf_wh(&self, _wo: Vector3<Float>, wh: Vector3<Float>) -> Float {
        self.d_wh(wh) * util::cos_t(wh).abs()
    }
}

#[derive(Clone, Debug)]
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

    pub fn without_schlick(color: Color, shininess: Float) -> Self {
        Self {
            color,
            microfacets: GGX::from_shininess(shininess),
            use_schlick: false,
        }
    }
}

impl BSDFT for MicrofacetBRDF {
    fn is_specular(&self) -> bool {
        false
    }

    fn brdf(&self, wo: Vector3<Float>, wi: Vector3<Float>) -> Color {
        let g = self.microfacets.g(wo, wi);
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

    fn pdf(&self, wo: Vector3<Float>, wi: Vector3<Float>) -> Float {
        if !util::same_hemisphere(wo, wi) {
            return 0.0;
        }
        let wh = (wo + wi).normalize();
        self.microfacets.pdf_wh(wo, wh) / (4.0 * wo.dot(wh).abs())
    }

    fn sample(&self, wo: Vector3<Float>) -> Option<(Color, Vector3<Float>, Float)> {
        let wh = self.microfacets.sample_wh(wo);
        let wi = util::reflect(wo, wh);
        if !util::same_hemisphere(wo, wi) {
            return None;
        }
        let pdf = self.pdf(wo, wi);
        let val = self.brdf(wo, wi);
        Some((val, wi, pdf))
    }
}

/// Combines microfacet reflection with diffuse reflection using fresnel schlick.
#[derive(Clone, Debug)]
pub struct FresnelBlendBRDF {
    diffuse: Color,
    specular: Color,
    microfacets: GGX,
}

impl FresnelBlendBRDF {
    pub fn new(diffuse: Color, specular: Color, shininess: Float) -> Self {
        Self {
            diffuse,
            specular,
            microfacets: GGX::from_shininess(shininess),
        }
    }
}

impl BSDFT for FresnelBlendBRDF {
    fn is_specular(&self) -> bool {
        false
    }

    fn brdf(&self, wo: Vector3<Float>, wi: Vector3<Float>) -> Color {
        let wh = (wo + wi).normalize();
        let d = self.microfacets.d_wh(wh);
        let odn = util::cos_t(wo).abs();
        let idn = util::cos_t(wi).abs();
        let denom = 4.0 * wh.dot(wi).abs() * odn.max(idn);
        let f_specular = d * fresnel::schlick(wo, self.specular) / denom;
        let p5 = |xdn: Float| 1.0 - (1.0 - xdn / 2.0).powi(5);
        let factor = 28.0 * self.diffuse / (23.0 * consts::PI);
        let f_diffuse = factor * (Color::white() - self.specular) * p5(idn) * p5(odn);
        f_specular + f_diffuse
    }

    fn btdf(&self, _wo: Vector3<Float>, _wi: Vector3<Float>) -> Color {
        Color::black()
    }

    fn pdf(&self, wo: Vector3<Float>, wi: Vector3<Float>) -> Float {
        if !util::same_hemisphere(wo, wi) {
            return 0.0;
        }
        let wh = (wo + wi).normalize();
        let d_pdf = sample::cosine_hemisphere_pdf(wi);
        let mf_pdf = self.microfacets.pdf_wh(wo, wh) / (4.0 * wo.dot(wh).abs());
        (d_pdf + mf_pdf) / 2.0
    }

    fn sample(&self, wo: Vector3<Float>) -> Option<(Color, Vector3<Float>, Float)> {
        let wi = if rand::random::<Float>() < 0.5 {
            let wh = self.microfacets.sample_wh(wo);
            let wi = util::reflect(wo, wh);
            if !util::same_hemisphere(wo, wi) {
                return None;
            }
            wi
        } else {
            sample::cosine_sample_hemisphere(wo.z)
        };
        let pdf = self.pdf(wo, wi);
        let val = self.brdf(wo, wi);
        Some((val, wi, pdf))
    }
}

#[derive(Clone, Debug)]
pub struct MicrofacetBTDF {
    color: Color,
    microfacets: GGX,
    eta: Float,
}

impl MicrofacetBTDF {
    pub fn new(color: Color, shininess: Float, eta: Float) -> Self {
        Self {
            color,
            microfacets: GGX::from_shininess(shininess),
            eta,
        }
    }

    /// Compute the half vector that will refract wo to wi and the inverse index of refraction
    fn refraction_values(&self, wo: Vector3<Float>, wi: Vector3<Float>) -> (Vector3<Float>, Float) {
        // This is inverse of the standard definition of eta
        let eta_inv = if wo.z > 0.0 { self.eta } else { 1.0 / self.eta };
        let mut wh = (wo + eta_inv * wi).normalize();
        if !util::same_hemisphere(wo, wh) {
            wh = -wh
        }
        (wh, eta_inv)
    }
}

impl BSDFT for MicrofacetBTDF {
    fn is_specular(&self) -> bool {
        false
    }

    fn brdf(&self, _wo: Vector3<Float>, _wi: Vector3<Float>) -> Color {
        Color::black()
    }

    fn btdf(&self, wo: Vector3<Float>, wi: Vector3<Float>) -> Color {
        let (wh, eta_inv) = self.refraction_values(wo, wi);
        let g = self.microfacets.g(wo, wi);
        let d = self.microfacets.d_wh(wh);
        let cos_ti = util::cos_t(wi).abs();
        let cos_to = util::cos_t(wo).abs();
        let idh = wi.dot(wh);
        let odh = wo.dot(wh);
        let denom = (odh + eta_inv * idh).powi(2) * cos_to * cos_ti;
        if denom < consts::EPSILON {
            Color::black()
        } else {
            self.color * eta_inv.powi(2) * d * g * idh.abs() * odh.abs() / denom
        }
    }

    fn pdf(&self, wo: Vector3<Float>, wi: Vector3<Float>) -> Float {
        if util::same_hemisphere(wo, wi) {
            return 0.0;
        }
        let (wh, eta_inv) = self.refraction_values(wo, wi);
        let idh = wi.dot(wh);
        let odh = wo.dot(wh);
        let denom = (odh + eta_inv * idh).powi(2);
        let cov = if denom < consts::EPSILON {
            1.0
        } else {
            ((eta_inv.powi(2) * idh) / denom).abs()
        };
        self.microfacets.pdf_wh(wo, wh) * cov
    }

    fn sample(&self, wo: Vector3<Float>) -> Option<(Color, Vector3<Float>, Float)> {
        let wh = self.microfacets.sample_wh(wo);
        let wi = util::refract(wo, wh, self.eta)?;
        let val = self.btdf(wo, wi);
        let pdf = self.pdf(wo, wi);
        Some((val, wi, pdf))
    }
}

pub type MicrofacetBSDF = FresnelBSDF<MicrofacetBRDF, MicrofacetBTDF>;

impl MicrofacetBSDF {
    pub fn new(reflect: Color, transmit: Color, shininess: Float, eta: Float) -> Self {
        let brdf = MicrofacetBRDF::without_schlick(reflect, shininess);
        let btdf = MicrofacetBTDF::new(transmit, shininess, eta);
        Self { brdf, btdf, eta }
    }
}
