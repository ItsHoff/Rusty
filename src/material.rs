use cgmath::{Point2, Vector3};

use glium::backend::Facade;
use glium::texture::SrgbTexture2d;

use crate::bsdf::BSDF;
use crate::color::Color;
use crate::float::*;
use crate::obj_load;
use crate::scattering::Scattering;
use crate::texture::{self, NormalMap};

/// Material for CPU rendering
#[derive(Debug)]
pub struct Material {
    scattering: Scattering,
    normal_map: Option<NormalMap>,
    pub emissive: Option<Color>,
}

/// Material for GPU rendering
pub struct GPUMaterial {
    pub texture: SrgbTexture2d, // Texture on the GPU
    pub is_emissive: bool,
}

// TODO: handle opaqueness_texture
impl Material {
    /// Create a new material based on a material loaded from the scene file
    pub fn new(obj_mat: &obj_load::Material) -> Material {
        let scattering = Scattering::from_obj(obj_mat);
        // TODO: handle emissive textures
        let emissive = obj_mat.emissive_color.and_then(|e| {
            if e == [0.0, 0.0, 0.0] {
                None
            } else {
                Some(Color::from(e))
            }
        });
        let normal_map = if let Some(path) = &obj_mat.bump_map {
            Some(texture::load_normal_map(path))
        } else {
            None
        };
        Material {
            scattering,
            normal_map,
            emissive,
        }
    }

    /// Upload textures to the GPU
    pub fn upload<F: Facade>(&self, facade: &F) -> GPUMaterial {
        let preview = self.scattering.preview_texture();
        let texture = preview.upload(facade);
        GPUMaterial {
            texture,
            is_emissive: self.emissive.is_some(),
        }
    }

    pub fn bsdf(&self, tex_coords: Point2<Float>) -> BSDF {
        self.scattering.local(tex_coords)
    }

    pub fn normal(&self, tex_coords: Point2<Float>) -> Option<Vector3<Float>> {
        if let Some(map) = &self.normal_map {
            Some(map.normal(tex_coords))
        } else {
            None
        }
    }
}
