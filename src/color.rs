use std::ops::{Add, AddAssign, Div, DivAssign, Mul};

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub fn black() -> Color {
        Color { r: 0.0, g: 0.0, b: 0.0 }
    }

    pub fn white() -> Color {
        Color { r: 1.0, g: 1.0, b: 1.0 }
    }

    pub fn from_srgb(rgba: image::Rgba<u8>) -> Color {
        let arr: Vec<f32> = rgba.data.into_iter()
            .map( |c| (f32::from(*c) / 255.0).powf(2.2) ).collect();
        Color { r: arr[0], g: arr[1], b: arr[2] }
    }
}

impl Add for Color {
    type Output = Color;

    fn add(self, rhs: Color) -> Color {
        Color { r: self.r + rhs.r,
                g: self.g + rhs.g,
                b: self.b + rhs.b }
    }
}

impl AddAssign for Color {
    fn add_assign(&mut self, rhs: Color) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
    }
}

impl Div<f32> for Color {
    type Output = Color;

    fn div(self, rhs: f32) -> Self::Output {
        Color { r: self.r / rhs, g: self.g / rhs, b: self.b / rhs }
    }
}

impl DivAssign<f32> for Color {
    fn div_assign(&mut self, rhs: f32) {
        self.r /= rhs;
        self.g /= rhs;
        self.b /= rhs;
    }
}

impl Mul for Color {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        Color { r: self.r * rhs.r, g: self.g * rhs.g, b: self.b * rhs.b }
    }
}

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, rhs: f32) -> Self::Output {
        Color { r: self.r * rhs, g: self.g * rhs, b: self.b * rhs }
    }
}

impl Mul<Color> for f32 {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        rhs * self
    }
}

impl From<[f32; 3]> for Color {
    fn from(arr: [f32; 3]) -> Color {
        Color { r: arr[0], g: arr[1], b: arr[2] }
    }
}

impl Into<[f32; 3]> for Color {
    fn into(self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }
}
