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

/// Refract w around wh, which is assumed to be in the same hemisphere as w.
/// eta_mat defines the index of refraction inside the material (outside is assumed to be air).
pub fn refract(w: Vector3<Float>, wh: Vector3<Float>, eta_mat: Float) -> Option<Vector3<Float>> {
    // Determine if w is entering or exiting the material
    let eta = eta(w, eta_mat);
    let cos_ti = w.dot(wh).abs();
    let sin2_ti = (1.0 - cos_ti.powi(2)).max(0.0);
    let sin2_tt = eta.powi(2) * sin2_ti;
    // Total internal reflection
    if sin2_tt >= 1.0 {
        return None;
    }
    let cos_tt = (1.0 - sin2_tt).sqrt();
    Some(-w * eta + (eta * cos_ti - cos_tt) * wh)
}

/// Refract w around the shading normal (0, 0, 1)
pub fn refract_n(w: Vector3<Float>, eta_mat: Float) -> Option<Vector3<Float>> {
    // Make sure normal is in the same hemisphere as w
    let n = w.z.signum() * Vector3::unit_z();
    refract(w, n, eta_mat)
}

/// Compute the total index of refraction for incident direction w.
pub fn eta(w: Vector3<Float>, eta_mat: Float) -> Float {
    if w.z > 0.0 {
        1.0 / eta_mat
    } else {
        eta_mat
    }
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
