use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use cgmath::Point2;
use cgmath::conv::*;

use glium::backend::Facade;
use glium::texture::{RawImage2d, SrgbTexture2d};

use image::ImageFormat;

use crate::obj_load;
use crate::color::Color;

/// Material for CPU rendering
pub struct Material {
    pub diffuse: Color,
    pub diffuse_image: Option<image::RgbaImage>,    // Texture on the CPU
    pub emissive: Option<Color>,
}

/// Material for GPU rendering
pub struct GPUMaterial {
    pub diffuse: [f32; 3],
    pub has_diffuse: bool,
    pub diffuse_texture: SrgbTexture2d,      // Texture on the GPU
    pub has_emissive: bool,
}

impl Material {
    /// Create a new material based on a material loaded from the scene file
    pub fn new(obj_mat: &obj_load::Material) -> Material {
        // Create diffuse texture and load it to the GPU
        let diffuse_image = match obj_mat.tex_diffuse {
            Some(ref tex_path) => Some(Material::load_image(tex_path)),
            None => None
        };
        let emissive = obj_mat.c_emissive.and_then(
            |e| if e == [0.0, 0.0, 0.0] { None }
            else { Some(Color::from(e)) });

        Material {
            diffuse: Color::from(obj_mat.c_diffuse.expect("No diffuse color!")),
            diffuse_image,
            emissive,
        }
    }

    /// Load an image from path
    fn load_image(path: &Path) -> image::RgbaImage {
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
            ext => panic!("Unknown image extension {}", ext),
        };
        // TODO: dont panic just warn and ignore texture
        let tex_reader = BufReader::new(File::open(path).unwrap_or_else(
            |_| panic!("Failed to open image {:?}!", path)));
        image::load(tex_reader, image_format).expect("Failed to load image!").to_rgba()
    }

    /// Upload textures to the GPU
    pub fn upload_textures<F: Facade>(&self, facade: &F) -> GPUMaterial {
        let diffuse_texture = match self.diffuse_image {
            Some(ref image) => {
                let image_dim = image.dimensions();
                let tex_image = RawImage2d::from_raw_rgba_reversed(&image.clone().into_raw(), image_dim);
                SrgbTexture2d::new(facade, tex_image).expect("Failed to upload texture!")
            }
            // Use empty texture as a placeholder
            None => SrgbTexture2d::empty(facade, 0, 0).expect("Failed to upload empty texture!")
        };
        GPUMaterial {
            diffuse: array3(self.diffuse),
            has_diffuse: self.diffuse_image.is_some(),
            diffuse_texture,
            has_emissive: self.emissive.is_some(),
        }
    }

    pub fn diffuse(&self, tex_coords: Point2<f32>) -> Color {
        if let Some(tex) = &self.diffuse_image {
            let (width, height) = tex.dimensions();
            let x = tex_coords.x.mod_euc(1.0) * (width - 1) as f32;
            let y = (1.0 - tex_coords.y.mod_euc(1.0)) * (height - 1) as f32;
            let x_fract = x.fract();
            let y_fract = y.fract();
            let (left, right) = if x >= (width - 1) as f32{
                (width - 1, width - 1)
            } else {
                (x.floor() as u32, x.ceil() as u32)
            };
            let (top, bottom) = if y >= (height - 1) as f32{
                (height - 1, height - 1)
            } else {
                (y.floor() as u32, y.ceil() as u32)
            };
            let tl = Color::from(tex.get_pixel(left, top));
            let bl = Color::from(tex.get_pixel(left, bottom));
            let tr = Color::from(tex.get_pixel(right, top));
            let br = Color::from(tex.get_pixel(right, bottom));
            let top_c = x_fract * tr + (1.0 - x_fract) * tl;
            let bottom_c = x_fract * br + (1.0 - x_fract) * bl;
            y_fract * bottom_c + (1.0 - y_fract) * top_c
        } else {
            self.diffuse
        }
    }
}
