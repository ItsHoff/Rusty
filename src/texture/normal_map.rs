use std::path::Path;

use cgmath::prelude::*;
use cgmath::{Point2, Vector3};

use image::{GrayImage, Rgb, RgbImage};

use crate::Float;

use super::GetColor;

#[derive(Clone, Debug)]
pub struct NormalMap {
    map: RgbImage
}

impl NormalMap {
    pub fn normal(&self, tex_coords: Point2<Float>) -> Vector3<Float> {
        let n = super::bilinear_interp(&self.map, tex_coords).to_vec();
        (2.0 * n).sub_element_wise(1.0).normalize()
    }
}

/// MTL bump map might refer to bump map or normal map.
/// Normal maps are returned as is and bump maps are converted to normal maps.
pub fn load_normal_map(path: &Path) -> NormalMap {
    use image::DynamicImage::*;

    let image = super::load_image(path).unwrap();
    // TODO: implement caching for converted maps
    let map = match image {
        ImageLuma8(map) => bump_to_normal(&map),
        ImageLumaA8(_) => bump_to_normal(&image.to_luma()),
        _ => {
            let rgb_image = image.to_rgb();
            if is_gray_scale(&rgb_image) {
                println!("Found gray scale rgb image {:?}", path);
                bump_to_normal(&image.to_luma())
            } else {
                rgb_image
            }
        }

    };
    // if let Some(name) = path.file_name() {
    //     let mut s = name.to_str().unwrap().to_string();
    //     s.insert_str(0, "to_normal_");
    //     let save_path = path.with_file_name(s);
    //     map.save(&save_path).unwrap();
    //     println!("saved {:?}", save_path);
    // }
    NormalMap { map }
}

/// Detect if an RgbImage is infact a gray scale image
fn is_gray_scale(image: &RgbImage) -> bool {
    let w = image.width();
    let h = image.height();
    // Check some points
    let c1 = image.get_color(0, 0);
    let c2 = image.get_color(w / 2, h / 2);
    let c3 = image.get_color(w / 4, h / 3);
    // If all points are gray the image is probably gray scale
    c1.is_gray() && c2.is_gray() && c3.is_gray()
}

fn float_to_u8(f: Float) -> u8 {
    (f * Float::from(std::u8::MAX)) as u8
}

fn normal_to_pixel(n: Vector3<Float>) -> Rgb<u8> {
    let p = (0.5 * n).add_element_wise(0.5);
    Rgb([float_to_u8(p.x), float_to_u8(p.y), float_to_u8(p.z)])
}

pub fn bump_to_normal(bump: &GrayImage) -> RgbImage {
    let w = bump.width();
    let h = bump.height();
    let mut nm = RgbImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let dx = if x == 0 {
                let p1 = bump.get_color(x, y);
                let p2 = bump.get_color(x + 1, y);
                p2 - p1
            } else if x == bump.width() - 1 {
                let p1 = bump.get_color(x - 1, y);
                let p2 = bump.get_color(x, y);
                p2 - p1
            } else {
                let p1 = bump.get_color(x - 1, y);
                let p2 = bump.get_color(x + 1, y);
                (p2 - p1) / 2.0
            };
            let dy = if y == 0 {
                let p1 = bump.get_color(x, y);
                let p2 = bump.get_color(x, y + 1);
                p2 - p1
            } else if y == bump.height() - 1 {
                let p1 = bump.get_color(x, y - 1);
                let p2 = bump.get_color(x, y);
                p2 - p1
            } else {
                let p1 = bump.get_color(x, y - 1);
                let p2 = bump.get_color(x, y + 1);
                (p2 - p1) / 2.0
            };
            // dx and dy need to be flipped to match the reference maps
            // TODO: implement better z scaling
            let n = Vector3::new(-dx, -dy, 0.1).normalize();
            nm.put_pixel(x, y, normal_to_pixel(n));
        }
    }
    nm
}
