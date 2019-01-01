use cgmath::prelude::*;
use cgmath::{Matrix3, Vector3};

use crate::consts;
use crate::float::*;

/// Compute an orthonormal coordinate frame where n defines is the z-axis
pub fn local_to_world(n: Vector3<Float>) -> Matrix3<Float> {
    let nx = if n.x.abs() > n.y.abs() {
        Vector3::new(n.z, 0.0, -n.x).normalize()
    } else {
        Vector3::new(0.0, -n.z, n.y).normalize()
    };
    let ny = n.cross(nx).normalize();
    Matrix3::from_cols(nx, ny, n)
}

#[allow(clippy::many_single_char_names)]
/// Cosine sample either (0, 0, 1) or (0, 0, -1) hemisphere decided by sign
pub fn cosine_sample_hemisphere(sign: Float) -> Vector3<Float> {
    let phi = 2.0 * consts::PI * rand::random::<Float>();
    let r = rand::random::<Float>().sqrt();
    let x = r * phi.cos();
    let y = r * phi.sin();
    // Make sure sampled vector is in the correct hemisphere
    // Use signum to ensure correct length
    let z = sign.signum() * (1.0 - r.powi(2)).sqrt();
    Vector3::new(x, y, z)
}

pub fn cosine_hemisphere_pdf(w: Vector3<Float>) -> Float {
    w.z.abs() / consts::PI
}

pub fn uniform_sample_sphere() -> Vector3<Float> {
    let phi = 2.0 * consts::PI * rand::random::<Float>();
    let z = 1.0 - 2.0 * rand::random::<Float>();
    let r = (1.0 - z.powi(2)).sqrt();
    Vector3::new(r * phi.cos(), r * phi.sin(), z)
}

pub fn uniform_sphere_pdf(_w: Vector3<Float>) -> Float {
    1.0 / (4.0 * consts::PI)
}
