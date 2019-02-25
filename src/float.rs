//! Floating point conversion that enable the support switching
//! between f64 and f32 as the primary float type.

use cgmath::{Matrix4, Point2, Point3, Vector3, Vector4};

use crate::consts;

#[cfg(not(feature = "single_precision"))]
pub use self::double::*;
#[cfg(feature = "single_precision")]
pub use self::single::*;

pub trait ToFloat {
    fn to_float(self) -> Float;
}

#[cfg(not(feature = "single_precision"))]
mod double {
    pub type Float = f64;
    use super::*;

    impl ToFloat for f32 {
        fn to_float(self) -> Float {
            self.into()
        }
    }

    impl ToFloat for f64 {
        fn to_float(self) -> Float {
            self
        }
    }
}

#[cfg(feature = "single_precision")]
mod single {
    pub type Float = f32;
    use super::*;

    impl ToFloat for f32 {
        fn to_float(self) -> Float {
            self
        }
    }

    impl ToFloat for f64 {
        fn to_float(self) -> Float {
            self as Float
        }
    }
}

/// Evaluate gamma for floating point errors
#[allow(dead_code)]
pub fn gamma(n: u32) -> Float {
    let n = n.to_float();
    n * consts::MACHINE_EPSILON / (1.0 - n * consts::MACHINE_EPSILON)
}

#[allow(dead_code)]
pub fn next_ulp(mut x: Float) -> Float {
    if x.is_infinite() && x > 0.0 {
        return x;
    }
    if x == -0.0 {
        x = 0.0
    }
    let mut bits = x.to_bits();
    bits = if x >= 0.0 { bits + 1 } else { bits - 1 };
    Float::from_bits(bits)
}

#[allow(dead_code)]
pub fn previous_ulp(mut x: Float) -> Float {
    if x.is_infinite() && x < 0.0 {
        return x;
    }
    if x == 0.0 {
        x = -0.0
    }
    let mut bits = x.to_bits();
    bits = if x >= 0.0 { bits - 1 } else { bits + 1 };
    Float::from_bits(bits)
}

impl ToFloat for u8 {
    fn to_float(self) -> Float {
        self.into()
    }
}

impl ToFloat for u32 {
    #[allow(clippy::cast_lossless)]
    fn to_float(self) -> Float {
        self as Float
    }
}

impl ToFloat for usize {
    fn to_float(self) -> Float {
        self as Float
    }
}

pub trait IntoArray {
    type Array;
    fn into_array(&self) -> Self::Array;
}

pub trait FromArray: IntoArray {
    fn from_array(array: Self::Array) -> Self;
}

impl IntoArray for Matrix4<Float> {
    type Array = [[f32; 4]; 4];

    fn into_array(&self) -> Self::Array {
        [
            self.x.into_array(),
            self.y.into_array(),
            self.z.into_array(),
            self.w.into_array(),
        ]
    }
}

impl IntoArray for Vector4<Float> {
    type Array = [f32; 4];

    fn into_array(&self) -> Self::Array {
        [self.x as f32, self.y as f32, self.z as f32, self.w as f32]
    }
}

impl IntoArray for Vector3<Float> {
    type Array = [f32; 3];

    fn into_array(&self) -> Self::Array {
        [self.x as f32, self.y as f32, self.z as f32]
    }
}

impl FromArray for Vector3<Float> {
    fn from_array(array: Self::Array) -> Self {
        Self::new(
            array[0].to_float(),
            array[1].to_float(),
            array[2].to_float(),
        )
    }
}

impl IntoArray for Point3<Float> {
    type Array = [f32; 3];

    fn into_array(&self) -> Self::Array {
        [self.x as f32, self.y as f32, self.z as f32]
    }
}

impl FromArray for Point3<Float> {
    fn from_array(array: Self::Array) -> Self {
        Self::new(
            array[0].to_float(),
            array[1].to_float(),
            array[2].to_float(),
        )
    }
}

impl IntoArray for Point2<Float> {
    type Array = [f32; 2];

    fn into_array(&self) -> Self::Array {
        [self.x as f32, self.y as f32]
    }
}

impl FromArray for Point2<Float> {
    fn from_array(array: Self::Array) -> Self {
        Self::new(array[0].to_float(), array[1].to_float())
    }
}
