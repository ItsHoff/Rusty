use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use cgmath::Point2;

use glium::backend::Facade;
use glium::texture::{RawImage2d, SrgbTexture2d};

use image::{DynamicImage, GenericImage, GrayImage, ImageFormat, RgbImage};

use crate::color::{self, Color, SrgbColor};
use crate::Float;

mod normal_map;

pub use self::normal_map::{load_normal_map, NormalMap};

#[derive(Clone, Debug)]
pub enum Texture {
    Solid(Color),
    Image(RgbImage),
}

// Bring enum variants to scope
use self::Texture::*;

impl Texture {
    pub fn from_color(color: Color) -> Self {
        Solid(color)
    }

    pub fn from_image_path(path: &Path) -> Self {
        Image(load_image(path).unwrap().to_rgb())
    }

    pub fn color(&self, tex_coords: Point2<Float>) -> Color {
        match self {
            Solid(color) => *color,
            Image(image) => bilinear_interp(image, tex_coords).to_linear(),
        }
    }

    pub fn upload<F: Facade>(&self, facade: &F) -> (Color, SrgbTexture2d) {
        match self {
            Image(image) => {
                let image_dim = image.dimensions();
                let tex_image =
                    RawImage2d::from_raw_rgb_reversed(&image.clone().into_raw(), image_dim);
                (
                    Color::black(),
                    SrgbTexture2d::new(facade, tex_image).unwrap(),
                )
            }
            // Use empty texture as a placeholder
            Solid(color) => (*color, SrgbTexture2d::empty(facade, 0, 0).unwrap()),
        }
    }
}

trait GetColor<T> {
    fn get_color(&self, x: u32, y: u32) -> T;
}

impl GetColor<SrgbColor> for RgbImage {
    fn get_color(&self, x: u32, y: u32) -> SrgbColor {
        SrgbColor::from_pixel(*self.get_pixel(x, y))
    }
}

impl GetColor<Float> for GrayImage {
    fn get_color(&self, x: u32, y: u32) -> Float {
        color::to_float(self.get_pixel(x, y)[0])
    }
}

#[allow(clippy::cast_lossless)]
fn bilinear_interp<T, I>(image: &I, tex_coords: Point2<Float>) -> T
where
    T: std::ops::Mul<Float, Output = T> + std::ops::Add<Output = T>,
    I: GetColor<T> + GenericImage,
{
    let (width, height) = image.dimensions();
    // Map wrapping coordinates to interval [0, 1)
    let x = tex_coords.x.mod_euc(1.0) * (width - 1) as Float;
    let y = (1.0 - tex_coords.y.mod_euc(1.0)) * (height - 1) as Float;
    let x_fract = x.fract();
    let y_fract = y.fract();
    // Make sure that pixel coordinates don't overflow
    let (left, right) = if x >= (width - 1) as Float {
        (width - 1, width - 1)
    } else {
        (x.floor() as u32, x.ceil() as u32)
    };
    let (top, bottom) = if y >= (height - 1) as Float {
        (height - 1, height - 1)
    } else {
        (y.floor() as u32, y.ceil() as u32)
    };
    // Get pixels
    let tl = image.get_color(left, top);
    let bl = image.get_color(left, bottom);
    let tr = image.get_color(right, top);
    let br = image.get_color(right, bottom);
    // Interpolate
    let top_c = tr * x_fract + tl * (1.0 - x_fract);
    let bottom_c = br * x_fract + bl * (1.0 - x_fract);
    bottom_c * y_fract + top_c * (1.0 - y_fract)
}

/// Load an image from path
fn load_image(path: &Path) -> Result<DynamicImage, Box<dyn Error>> {
    let image_format = match path.extension().unwrap().to_str().unwrap() {
        "png" => ImageFormat::PNG,
        "jpg" | "jpeg" => ImageFormat::JPEG,
        "gif" => ImageFormat::GIF,
        "webp" => ImageFormat::WEBP,
        "pnm" => ImageFormat::PNM,
        "tiff" => ImageFormat::TIFF,
        "tga" => ImageFormat::TGA,
        "bmp" => ImageFormat::BMP,
        "ico" => ImageFormat::ICO,
        "hdr" => ImageFormat::HDR,
        ext => {
            return Err(format!("Unknown image extension {}", ext).into());
        }
    };
    let reader = BufReader::new(File::open(path)?);
    image::load(reader, image_format).map_err(|e| e.into())
}
