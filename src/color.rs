use std::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign};

use cgmath::prelude::*;
use cgmath::Vector3;

use crate::util::ConvArr;
use crate::Float;

/// Convert u8 color to float color in range [0, 1]
pub fn to_float(c: u8) -> Float {
    Float::from(c) / 255.0
}

/// Convert srgb color to linear color
fn to_linear(c: Float) -> Float {
    c.powf(2.2)
}

#[derive(Clone, Copy, Debug)]
pub struct SrgbColor(BaseColor);

impl SrgbColor {
    pub fn from_pixel(rgb: image::Rgb<u8>) -> Self {
        Self(BaseColor::from_pixel(rgb))
    }

    pub fn is_gray(&self) -> bool {
        self.0.is_gray()
    }

    pub fn to_linear(self) -> Color {
        Color(self.0.to_linear())
    }

    pub fn to_vec(self) -> Vector3<Float> {
        self.0.color
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Color(BaseColor);

impl Color {
    pub fn black() -> Self {
        Self(BaseColor::black())
    }

    pub fn white() -> Self {
        Self(BaseColor::white())
    }

    pub fn from_normal(n: Vector3<Float>) -> Self {
        let c_vec = (0.5 * n).add_element_wise(0.5);
        Self(BaseColor::from(c_vec))
    }

    pub fn luma(&self) -> Float {
        self.0.luma()
    }

    pub fn is_black(&self) -> bool {
        self.0.is_black()
    }

    pub fn r(&self) -> Float {
        self.0.r()
    }

    pub fn g(&self) -> Float {
        self.0.g()
    }

    pub fn b(&self) -> Float {
        self.0.b()
    }
}

#[derive(Clone, Copy, Debug)]
struct BaseColor {
    color: Vector3<Float>,
}

impl BaseColor {
    fn new(r: Float, g: Float, b: Float) -> Self {
        Self {
            color: Vector3::new(r, g, b),
        }
    }

    fn black() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    fn white() -> Self {
        Self::new(1.0, 1.0, 1.0)
    }

    fn from_pixel(rgb: image::Rgb<u8>) -> Self {
        Self::new(
            to_float(rgb.data[0]),
            to_float(rgb.data[1]),
            to_float(rgb.data[2]),
        )
    }

    fn to_linear(self) -> Self {
        Self::new(
            to_linear(self.color[0]),
            to_linear(self.color[1]),
            to_linear(self.color[2]),
        )
    }

    fn luma(&self) -> Float {
        let luma_vec = Vector3::new(0.2126, 0.7152, 0.0722);
        luma_vec.dot(self.color)
    }

    fn is_black(&self) -> bool {
        self.color.x == 0.0 && self.color.y == 0.0 && self.color.z == 0.0
    }

    fn is_gray(&self) -> bool {
        self.color.x == self.color.y && self.color.y == self.color.z
    }

    fn r(&self) -> Float {
        self.color.x
    }

    fn g(&self) -> Float {
        self.color.y
    }

    fn b(&self) -> Float {
        self.color.z
    }
}

impl Index<usize> for BaseColor {
    type Output = Float;

    fn index(&self, i: usize) -> &Float {
        &self.color[i]
    }
}

impl IndexMut<usize> for BaseColor {
    fn index_mut(&mut self, i: usize) -> &mut Float {
        &mut self.color[i]
    }
}

impl From<Vector3<Float>> for BaseColor {
    #[allow(clippy::identity_conversion)]
    fn from(vec: Vector3<Float>) -> Self {
        Self { color: vec }
    }
}

impl From<[f32; 3]> for BaseColor {
    #[allow(clippy::identity_conversion)]
    fn from(arr: [f32; 3]) -> Self {
        Self {
            color: Vector3::from_arr(arr),
        }
    }
}

impl Into<[f32; 3]> for BaseColor {
    fn into(self) -> [f32; 3] {
        self.color.into_arr()
    }
}

impl From<[f32; 3]> for Color {
    #[allow(clippy::identity_conversion)]
    fn from(arr: [f32; 3]) -> Self {
        Self(BaseColor::from(arr))
    }
}

impl Into<[f32; 3]> for Color {
    fn into(self) -> [f32; 3] {
        self.0.into()
    }
}

// Arithmetic operations

impl Add for BaseColor {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self {
        self += rhs;
        self
    }
}

impl AddAssign for BaseColor {
    fn add_assign(&mut self, rhs: Self) {
        self.color += rhs.color;
    }
}

impl Div<Float> for BaseColor {
    type Output = Self;

    fn div(mut self, rhs: Float) -> Self {
        self /= rhs;
        self
    }
}

impl DivAssign<Float> for BaseColor {
    fn div_assign(&mut self, rhs: Float) {
        let recip = rhs.recip();
        self.color *= recip;
    }
}

impl Mul for BaseColor {
    type Output = Self;

    fn mul(mut self, rhs: Self) -> Self {
        self *= rhs;
        self
    }
}

impl MulAssign for BaseColor {
    fn mul_assign(&mut self, rhs: Self) {
        self.color.mul_assign_element_wise(rhs.color);
    }
}

impl Mul<Float> for BaseColor {
    type Output = Self;

    fn mul(mut self, rhs: Float) -> Self {
        self *= rhs;
        self
    }
}

impl MulAssign<Float> for BaseColor {
    fn mul_assign(&mut self, rhs: Float) {
        self.color.mul_assign_element_wise(rhs);
    }
}

impl Mul<BaseColor> for Float {
    type Output = BaseColor;

    // Delegate to BaseColor Mul
    fn mul(self, rhs: BaseColor) -> Self::Output {
        rhs * self
    }
}

// Color operations delegated to BaseColor

impl Add for Color {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self {
        self.0 += rhs.0;
        self
    }
}

impl AddAssign for Color {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Div<Float> for Color {
    type Output = Self;

    fn div(mut self, rhs: Float) -> Self {
        self.0 /= rhs;
        self
    }
}

impl DivAssign<Float> for Color {
    fn div_assign(&mut self, rhs: Float) {
        let recip = rhs.recip();
        self.0 *= recip;
    }
}

impl Mul for Color {
    type Output = Self;

    fn mul(mut self, rhs: Self) -> Self {
        self.0 *= rhs.0;
        self
    }
}

impl Mul<Float> for Color {
    type Output = Self;

    fn mul(mut self, rhs: Float) -> Self {
        self.0 *= rhs;
        self
    }
}

impl Mul<Color> for Float {
    type Output = Color;

    // Delegate to BaseColor Mul
    fn mul(self, rhs: Color) -> Self::Output {
        rhs * self
    }
}

// SrgbColor operations delegated to BaseColor

impl Add for SrgbColor {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self {
        self.0 += rhs.0;
        self
    }
}

impl AddAssign for SrgbColor {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Div<Float> for SrgbColor {
    type Output = Self;

    fn div(mut self, rhs: Float) -> Self {
        self.0 /= rhs;
        self
    }
}

impl DivAssign<Float> for SrgbColor {
    fn div_assign(&mut self, rhs: Float) {
        let recip = rhs.recip();
        self.0 *= recip;
    }
}

impl Mul for SrgbColor {
    type Output = Self;

    fn mul(mut self, rhs: Self) -> Self {
        self.0 *= rhs.0;
        self
    }
}

impl Mul<Float> for SrgbColor {
    type Output = Self;

    fn mul(mut self, rhs: Float) -> Self {
        self.0 *= rhs;
        self
    }
}

impl Mul<SrgbColor> for Float {
    type Output = SrgbColor;

    // Delegate to BaseColor Mul
    fn mul(self, rhs: SrgbColor) -> Self::Output {
        rhs * self
    }
}
