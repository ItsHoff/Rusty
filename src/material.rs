use cgmath::{Point2, Vector3};

use glium::backend::Facade;
use glium::texture::SrgbTexture2d;

use crate::bsdf::{BSDF, LambertianBRDF};
use crate::color::Color;
use crate::obj_load;
use crate::texture::{self, NormalMap, Texture};
use crate::Float;

/// Material for CPU rendering
#[derive(Clone, Debug)]
pub struct Material {
    texture: Texture,
    normal_map: Option<NormalMap>,
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
        // let bsdf = match obj_mat.illumination_model.unwrap_or(0) {
        //     5 => BSDF::Specular,
        //     i if i <= 10 => BSDF::Diffuse,
        //     i => {
        //         println!("Illumination model {} is not defined in the mtl spec!", i);
        //         println!("Defaulting to diffuse BSDF.");
        //         BSDF::Diffuse
        //     }
        // };
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
        let normal_map = if let Some(path) = &obj_mat.tex_bump {
            Some(texture::load_normal_map(path))
        } else {
            None
        };
        Material {
            texture,
            normal_map,
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

    pub fn bsdf(&self, tex_coords: Point2<Float>) -> Box<dyn BSDF> {
        let bsdf = LambertianBRDF::new(self.texture.color(tex_coords));
        Box::new(bsdf)
    }

    pub fn normal(&self, tex_coords: Point2<Float>) -> Option<Vector3<Float>> {
        if let Some(map) = &self.normal_map {
            Some(map.normal(tex_coords))
        } else {
            None
        }
    }
}
