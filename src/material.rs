use cgmath::Point2;

use glium::backend::Facade;
use glium::texture::SrgbTexture2d;

use crate::color::Color;
use crate::obj_load;
use crate::texture::Texture;
use crate::Float;

#[derive(Clone, Debug)]
pub enum BSDF {
    Diffuse,
    Specular,
}

/// Material for CPU rendering
#[derive(Clone, Debug)]
pub struct Material {
    bsdf: BSDF,
    texture: Texture,
    pub emissive: Option<Color>,
}

/// Material for GPU rendering
pub struct GPUMaterial {
    pub color: [f32; 3],
    pub has_texture: bool,
    pub texture: SrgbTexture2d, // Texture on the GPU
    pub is_emissive: bool,
}

impl Material {
    /// Create a new material based on a material loaded from the scene file
    pub fn new(obj_mat: &obj_load::Material) -> Material {
        let texture = match obj_mat.tex_diffuse {
            Some(ref path) => Texture::from_image_path(path),
            None => {
                let color = Color::from(obj_mat.c_diffuse.unwrap());
                Texture::from_color(color)
            }
        };
        let emissive = obj_mat.c_emissive.and_then(|e| {
            if e == [0.0, 0.0, 0.0] {
                None
            } else {
                Some(Color::from(e))
            }
        });
        Material {
            bsdf: BSDF::Diffuse,
            texture,
            emissive,
        }
    }

    /// Upload textures to the GPU
    pub fn upload_textures<F: Facade>(&self, facade: &F) -> GPUMaterial {
        let (color, texture) = self.texture.upload(facade);
        GPUMaterial {
            color: color.into(),
            has_texture: texture.width() > 0,
            texture,
            is_emissive: self.emissive.is_some(),
        }
    }

    pub fn diffuse(&self, tex_coords: Point2<Float>) -> Color {
        self.texture.color(tex_coords)
    }
}
