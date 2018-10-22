use std::ops::{Add, AddAssign, Div, DivAssign, Mul};

use crate::Float;

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: Float,
    pub g: Float,
    pub b: Float,
}

impl Color {
    fn new(r: Float, g: Float, b: Float) -> Self {
        Color { r, g, b }
    }

    pub fn black() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn white() -> Self {
        Self::new(1.0, 1.0, 1.0)
    }

    pub fn from_srgb(rgba: image::Rgba<u8>) -> Self {
        let arr: Vec<Float> = rgba
            .data
            .into_iter()
            .map(|c| (Float::from(*c) / 255.0).powf(2.2))
            .collect();
        Color {
            r: arr[0],
            g: arr[1],
            b: arr[2],
        }
    }
}

impl Add for Color {
    type Output = Color;

    fn add(self, rhs: Color) -> Color {
        let mut c = self;
        c += rhs;
        c
    }
}

impl AddAssign for Color {
    fn add_assign(&mut self, rhs: Color) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
    }
}

impl Div<Float> for Color {
    type Output = Color;

    fn div(self, rhs: Float) -> Self::Output {
        let mut c = self;
        c /= rhs;
        c
    }
}

impl DivAssign<Float> for Color {
    fn div_assign(&mut self, rhs: Float) {
        let recip = rhs.recip();
        self.r *= recip;
        self.g *= recip;
        self.b *= recip;
    }
}

impl Mul for Color {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        Color {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
        }
    }
}

impl Mul<Float> for Color {
    type Output = Color;

    fn mul(self, rhs: Float) -> Self::Output {
        Color {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
        }
    }
}

impl Mul<Color> for Float {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        rhs * self
    }
}

impl From<[f32; 3]> for Color {
    #[allow(clippy::identity_conversion)]
    fn from(arr: [f32; 3]) -> Color {
        Color {
            r: arr[0].into(),
            g: arr[1].into(),
            b: arr[2].into(),
        }
    }
}

impl Into<[f32; 3]> for Color {
    fn into(self) -> [f32; 3] {
        [self.r as f32, self.g as f32, self.b as f32]
    }
}
