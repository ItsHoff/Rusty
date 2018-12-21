//! Floating point conversion that enable the support switching
//! between f64 and f32 as the primary float type.

use cgmath::{Matrix4, Point2, Point3, Vector3, Vector4};

/// Alias for the float type used by the renderer
pub type Float = f64;

pub trait ToFloat {
    fn to_float(self) -> Float;
}

impl ToFloat for f64 {
    #[allow(clippy::identity_conversion)]
    fn to_float(self) -> Float {
        self as Float
    }
}

impl ToFloat for f32 {
    #[allow(clippy::identity_conversion)]
    fn to_float(self) -> Float {
        self.into()
    }
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
