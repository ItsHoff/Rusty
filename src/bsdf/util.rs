#![allow(dead_code)]
//! Utility functions for operating on vectors in shading coordinates

use cgmath::prelude::*;
use cgmath::Vector3;

use crate::float::*;

/// Check if the vectors are in the same hemisphere
pub fn same_hemisphere(w1: Vector3<Float>, w2: Vector3<Float>) -> bool {
    w1.z * w2.z > 0.0
}

/// Reflect w around wh
pub fn reflect(w: Vector3<Float>, wh: Vector3<Float>) -> Vector3<Float> {
    -w + 2.0 * wh.dot(w) * wh
}

/// Reflect w around n
pub fn reflect_n(w: Vector3<Float>) -> Vector3<Float> {
    Vector3::new(-w.x, -w.y, w.z)
}

/// Refract w around n, where eta_mat defines the index of refraction
/// inside the material (outside is assumed to be air).
pub fn refract_n(w: Vector3<Float>, eta_mat: Float) -> Option<Vector3<Float>> {
    // Determine if w is entering or exiting the material
    let (n, eta) = if w.z > 0.0 {
        (Vector3::unit_z(), 1.0 / eta_mat)
    } else {
        (-Vector3::unit_z(), eta_mat)
    };
    let cos_ti = cos_t(w);
    let sin2_ti = (1.0 - cos_ti.powi(2)).max(0.0);
    let sin2_tt = eta.powi(2) * sin2_ti;
    // Total internal reflection
    if sin2_tt >= 1.0 {
        return None;
    }
    let cos_tt = (1.0 - sin2_tt).sqrt();
    Some(-w * eta + (eta * cos_ti - cos_tt) * n)
}

/// Fresnel reflection for w
pub fn fresnel_dielectric(w: Vector3<Float>, eta_mat: Float) -> Float {
    // Determine if w is entering or exiting the material
    let (eta_i, eta_t) = if w.z > 0.0 {
        (1.0, eta_mat)
    } else {
        (eta_mat, 1.0)
    };
    let cos_ti = cos_t(w).abs();
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

// Trigonometric functions

pub fn cos_t(vec: Vector3<Float>) -> Float {
    vec.z
}

pub fn cos2_t(vec: Vector3<Float>) -> Float {
    vec.z * vec.z
}

pub fn sin2_t(vec: Vector3<Float>) -> Float {
    1.0 - cos2_t(vec)
}

pub fn sin_t(vec: Vector3<Float>) -> Float {
    sin2_t(vec).sqrt()
}

pub fn tan2_t(vec: Vector3<Float>) -> Float {
    sin2_t(vec) / cos2_t(vec)
}
