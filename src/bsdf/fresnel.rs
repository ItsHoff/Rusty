use cgmath::Vector3;

use crate::color::Color;
use crate::float::*;
use crate::pt_renderer::PathType;

use super::util;
use super::BsdfT;

/// Fresnel reflection for w
fn dielectric(w: Vector3<Float>, eta_mat: Float) -> Float {
    // Determine if w is entering or exiting the material
    let (eta_i, eta_t) = if w.z > 0.0 {
        (1.0, eta_mat)
    } else {
        (eta_mat, 1.0)
    };
    let cos_ti = util::cos_t(w).abs();
    let sin2_ti = (1.0 - cos_ti.powi(2)).max(0.0);
    let sin2_tt = (eta_i / eta_t).powi(2) * sin2_ti;
    // Total internal reflection
    if sin2_tt >= 1.0 {
        return 1.0;
    }
    let cos_tt = (1.0 - sin2_tt).sqrt();
    let paral = (eta_t * cos_ti - eta_i * cos_tt) / (eta_t * cos_ti + eta_i * cos_tt);
    let perp = (eta_i * cos_ti - eta_t * cos_tt) / (eta_i * cos_ti + eta_t * cos_tt);
    (paral.powi(2) + perp.powi(2)) / 2.0
}

pub fn schlick(w: Vector3<Float>, specular: Color) -> Color {
    let cos_t = util::cos_t(w).abs();
    specular + (1.0 - cos_t).powi(5) * (Color::white() - specular)
}

#[derive(Clone, Debug)]
pub struct FresnelBsdf<R: BsdfT, T: BsdfT> {
    pub brdf: R,
    pub btdf: T,
    pub eta: Float,
}

impl<R: BsdfT, T: BsdfT> BsdfT for FresnelBsdf<R, T> {
    fn is_specular(&self) -> bool {
        self.brdf.is_specular() || self.btdf.is_specular()
    }

    fn brdf(&self, wo: Vector3<Float>, wi: Vector3<Float>) -> Color {
        let fr = dielectric(wo, self.eta);
        fr * self.brdf.brdf(wo, wi)
    }

    fn btdf(&self, wo: Vector3<Float>, wi: Vector3<Float>, path_type: PathType) -> Color {
        let fr = dielectric(wo, self.eta);
        let ft = 1.0 - fr;
        ft * self.btdf.btdf(wo, wi, path_type)
    }

    fn pdf(&self, wo: Vector3<Float>, wi: Vector3<Float>) -> Float {
        let fr = dielectric(wo, self.eta);
        if util::same_hemisphere(wo, wi) {
            fr * self.brdf.pdf(wo, wi)
        } else {
            (1.0 - fr) * self.btdf.pdf(wo, wi)
        }
    }

    fn sample(
        &self,
        wo: Vector3<Float>,
        path_type: PathType,
    ) -> Option<(Color, Vector3<Float>, Float)> {
        let fr = dielectric(wo, self.eta);
        if rand::random::<Float>() < fr {
            let (color, wi, pdf) = self.brdf.sample(wo, path_type)?;
            Some((fr * color, wi, fr * pdf))
        } else {
            let (color, wi, pdf) = self.btdf.sample(wo, path_type)?;
            let ft = 1.0 - fr;
            Some((ft * color, wi, ft * pdf))
        }
    }
}
