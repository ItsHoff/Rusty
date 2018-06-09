use glium::{Rect, backend::Facade, texture::{RawImage2d, Texture2d}};

pub struct TracedImage {
    pub texture: Texture2d,
}

impl TracedImage {
    pub fn empty<F: Facade>(facade: &F, width: u32, height: u32) -> TracedImage {
        let empty_image = RawImage2d::from_raw_rgb(vec![0.0; (3 * width * height) as usize], (width, height));
        let texture = Texture2d::new(facade, empty_image).expect("Failed to upload traced image!");
        TracedImage { texture }
    }

    pub fn update_block(&mut self, rect: Rect, block: Vec<f32>) {
        let raw_block = RawImage2d::from_raw_rgb(block, (rect.width, rect.height));
        self.texture.write(rect, raw_block);
    }
}
