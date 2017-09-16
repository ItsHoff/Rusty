extern crate image;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use glium::backend::Facade;
use glium::texture::{RawImage2d, SrgbTexture2d};

use scene::obj_load;

/// Renderer representation of a material
pub struct Material {
    pub diffuse: [f32; 3],
    pub diffuse_image: Option<image::RgbaImage>,    // Texture on the CPU
    pub diffuse_texture: Option<SrgbTexture2d>      // Texture on the GPU
}

impl Material {
    /// Create a new material based on a material loaded from the scene file
    pub fn new(obj_mat: &obj_load::Material) -> Material {
        // Create diffuse texture and load it to the GPU
        let diffuse_image = match obj_mat.tex_diffuse {
            Some(ref tex_path) => Some(Material::load_image(tex_path)),
            None => None
        };
        Material {
            diffuse: obj_mat.c_diffuse.expect("No diffuse color!"),
            diffuse_image: diffuse_image,
            diffuse_texture: None
        }
    }

    /// Load an image at
    fn load_image(path: &Path) -> image::RgbaImage {
        let tex_reader = BufReader::new(File::open(path).expect("Failed to open image!"));
        image::load(tex_reader, image::PNG).expect("Failed to load image!").to_rgba()
    }

    /// Upload textures to the GPU
    pub fn upload_textures<F: Facade>(&mut self, facade: &F) {
        self.diffuse_texture = match self.diffuse_image {
            Some(ref image) => {
                let image_dim = image.dimensions();
                let tex_image = RawImage2d::from_raw_rgba_reversed(&image.clone().into_raw(), image_dim);
                Some(SrgbTexture2d::new(facade, tex_image).expect("Failed to upload texture!"))
            }
            // Use empty texture as a placeholder
            None => Some(SrgbTexture2d::empty(facade, 0, 0).expect("Failed to upload empty texture!"))
        }
    }
}
