#![allow(clippy::cast_lossless)]

use cgmath::{Matrix4, Point2, Point3, Vector3, Vector4};

use crate::Float;

pub trait ConvArr<T> {
    fn from_arr(arr: T) -> Self;
    fn into_arr(&self) -> T;
}

pub trait ToF32<T> {
    fn to_f32(&self) -> T;
}

impl ConvArr<[f32; 3]> for Vector3<Float> {
    fn from_arr(arr: [f32; 3]) -> Self {
        Self::new(arr[0] as Float, arr[1] as Float, arr[2] as Float)
    }

    fn into_arr(&self) -> [f32; 3] {
        [self.x as f32, self.y as f32, self.z as f32]
    }
}

impl ConvArr<[f32; 3]> for Point3<Float> {
    fn from_arr(arr: [f32; 3]) -> Self {
        Self::new(arr[0] as Float, arr[1] as Float, arr[2] as Float)
    }

    fn into_arr(&self) -> [f32; 3] {
        [self.x as f32, self.y as f32, self.z as f32]
    }
}

impl ConvArr<[f32; 2]> for Point2<Float> {
    fn from_arr(arr: [f32; 2]) -> Self {
        Self::new(arr[0] as Float, arr[1] as Float)
    }

    fn into_arr(&self) -> [f32; 2] {
        [self.x as f32, self.y as f32]
    }
}

impl ToF32<Vector4<f32>> for Vector4<Float> {
    fn to_f32(&self) -> Vector4<f32> {
        Vector4::new(self.x as f32, self.y as f32, self.z as f32, self.w as f32)
    }
}

impl ToF32<Matrix4<f32>> for Matrix4<Float> {
    fn to_f32(&self) -> Matrix4<f32> {
        Matrix4::from_cols(
            self.x.to_f32(),
            self.y.to_f32(),
            self.z.to_f32(),
            self.w.to_f32(),
        )
    }
}
