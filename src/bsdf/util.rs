#![allow(dead_code)]

use cgmath::Vector3;

use crate::Float;

// Trigonometric functions for vectors in shading coordinates

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
