use std::path::Path;

use glium::{texture::RawImage2d, Rect};

pub struct TracedImage {
    raw_image: Vec<f32>,
    n_samples: Vec<u32>,
    width: u32,
    height: u32,
}

impl TracedImage {
    pub fn empty(width: u32, height: u32) -> TracedImage {
        let raw_image = vec![0.0; (3 * width * height) as usize];
        let n_samples = vec![0; (width * height) as usize];
        TracedImage {
            raw_image,
            n_samples,
            width,
            height,
        }
    }

    pub fn update_block(&mut self, rect: Rect, block: &[f32]) -> (Rect, RawImage2d<f32>) {
        let mut updated_block = vec![0.0f32; (3 * rect.width * rect.height) as usize];
        for h in 0..rect.height {
            for w in 0..rect.width {
                let i_image = ((h + rect.bottom) * self.width + w + rect.left) as usize;
                let i_block = (h * rect.width + w) as usize;
                let n = self.n_samples[i_image] + 1;
                self.n_samples[i_image] = n;
                for c in 0..3 {
                    let old_val = self.raw_image[3 * i_image + c];
                    let new_val = old_val + 1.0 / (n as f32) * (block[3 * i_block + c] - old_val);
                    updated_block[3 * i_block + c] = new_val;
                    self.raw_image[3 * i_image + c] = new_val;
                }
            }
        }
        let block_to_upload = RawImage2d::from_raw_rgb(updated_block, (rect.width, rect.height));
        (rect, block_to_upload)
    }

    pub fn get_texture_source(&self) -> RawImage2d<f32> {
        RawImage2d::from_raw_rgb(self.raw_image.clone(), (self.width, self.height))
    }

    pub fn save_image(&self, path: &Path) {
        let mapped_image: Vec<u8> = self
            .raw_image
            .iter()
            .map(|p| (255.0 * p.powf(1.0 / 2.2).min(1.0)) as u8)
            .collect();
        let image = image::DynamicImage::ImageRgb8(
            image::RgbImage::from_raw(self.width, self.height, mapped_image).unwrap(),
        );
        let image = image.flipv();
        image.save(path).unwrap();
    }
}
