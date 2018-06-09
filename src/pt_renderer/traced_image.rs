use glium::{Rect, backend::Facade, texture::{RawImage2d, Texture2d}};

pub struct TracedImage {
    pub texture: Texture2d,
    raw_image: Vec<f32>,
    n_samples: Vec<u32>,
    width: u32,
}

impl TracedImage {
    pub fn empty<F: Facade>(facade: &F, width: u32, height: u32) -> TracedImage {
        let raw_image = vec![0.0; (3 * width * height) as usize];
        let n_samples = vec![0; (width * height) as usize];
        let texture_base = RawImage2d::from_raw_rgb(raw_image.clone(), (width, height));
        let texture = Texture2d::new(facade, texture_base).expect("Failed to upload traced image!");
        TracedImage { texture, raw_image, n_samples, width, }
    }

    pub fn update_block(&mut self, rect: Rect, block: &[f32]) {
        let mut updated_block = vec![0.0f32; (3 * rect.width * rect.height) as usize];
        for h in 0..rect.height {
            for w in 0..rect.width {
                let i_image = ((h + rect.bottom) * self.width + w + rect.left) as usize;
                let i_block = (h * rect.width + w) as usize;
                let n = self.n_samples[i_image] + 1;
                self.n_samples[i_image] = n;
                for c in 0..3 {
                    let old_val = self.raw_image[3 * i_image + c];
                    let new_val = old_val + 1.0 / (n as f32) * (block[3*i_block + c] - old_val);
                    updated_block[3 * i_block + c] = new_val;
                    self.raw_image[3 * i_image + c] = new_val;
                }
            }
        }
        let block_to_upload = RawImage2d::from_raw_rgb(updated_block, (rect.width, rect.height));
        self.texture.write(rect, block_to_upload);
    }
}
