use cgmath::prelude::*;
use cgmath::{Point2, Vector3};

use crate::color::Color;
use crate::obj_load;
use crate::texture::Texture;
use crate::Float;

use super::{ShadingModelT, BSDF, BSDFT};

#[derive(Debug)]
pub struct SpecularReflection {
    texture: Texture,
}

impl SpecularReflection {
    // TODO: use specular color to figure out fresnel coeffs or use Schlick
    pub fn new(obj_mat: &obj_load::Material) -> Self {
        let texture = match &obj_mat.tex_specular {
            Some(path) => Texture::from_image_path(path),
            None => {
                let color = Color::from(obj_mat.c_specular.unwrap());
                Texture::from_color(color)
            }
        };
        Self { texture }
    }

    fn scattering(&self, tex_coords: Point2<Float>) -> SpecularBRDF {
        SpecularBRDF::new(self.texture.color(tex_coords))
    }
}

impl ShadingModelT for SpecularReflection {
    fn bsdf(&self, tex_coords: Point2<Float>) -> BSDF {
        BSDF::SR(self.scattering(tex_coords))
    }

    fn preview_texture(&self) -> &Texture {
        &self.texture
    }
}

#[derive(Debug)]
pub struct SpecularBRDF {
    color: Color,
}

impl SpecularBRDF {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

impl BSDFT for SpecularBRDF {
    fn is_specular(&self) -> bool {
        true
    }

    fn eval(&self, _in_dir: Vector3<Float>, _out_dir: Vector3<Float>) -> Color {
        Color::black()
    }

    fn sample(&self, out_dir: Vector3<Float>) -> (Color, Vector3<Float>, Float) {
        let in_dir = Vector3::new(-out_dir.x, -out_dir.y, out_dir.z);
        (self.color, in_dir, 1.0)
    }
}

#[derive(Debug)]
pub struct SpecularTransmission {
    texture: Texture,
    eta: Float,
}

impl SpecularTransmission {
    pub fn new(obj_mat: &obj_load::Material) -> Self {
        let filter = Color::from(
            obj_mat
                .c_translucency
                .expect("No translucent color for translucent material"),
        );
        // TODO: not sure if which is the correct interpretation
        // or if it is even scene dependant
        let color = Color::white() - filter;
        // let color = filter;
        let texture = Texture::from_color(color);
        let eta = obj_mat
            .refraction_i
            .expect("No index of refraction for translucent material")
            .into();
        Self { texture, eta }
    }

    fn scattering(&self, tex_coords: Point2<Float>) -> SpecularBTDF {
        SpecularBTDF::new(self.texture.color(tex_coords), self.eta)
    }
}

#[derive(Debug)]
pub struct SpecularBTDF {
    color: Color,
    eta: Float,
}

impl SpecularBTDF {
    pub fn new(color: Color, eta: Float) -> Self {
        Self { color, eta }
    }
}

impl BSDFT for SpecularBTDF {
    fn is_specular(&self) -> bool {
        true
    }

    fn eval(&self, _in_dir: Vector3<Float>, _out_dir: Vector3<Float>) -> Color {
        Color::black()
    }

    fn sample(&self, out_dir: Vector3<Float>) -> (Color, Vector3<Float>, Float) {
        let (n, eta) = if out_dir.z > 0.0 {
            (Vector3::unit_z(), 1.0 / self.eta)
        } else {
            (-Vector3::unit_z(), self.eta)
        };
        let cos_i = out_dir.dot(n);
        let sin2_i = (1.0 - cos_i.powi(2)).max(0.0);
        let sin2_t = eta.powi(2) * sin2_i;
        // Total internal reflection
        if sin2_t >= 1.0 {
            return (Color::black(), Vector3::unit_z(), 1.0);
        }
        let cos_t = (1.0 - sin2_t).sqrt();
        let in_dir = -out_dir * eta + (eta * cos_i - cos_t) * n;
        (self.color / in_dir.z.abs(), in_dir, 1.0)
    }
}

#[derive(Debug)]
pub struct FresnelSpecular {
    reflection: SpecularReflection,
    transmission: SpecularTransmission,
}

impl FresnelSpecular {
    pub fn new(obj_mat: &obj_load::Material) -> Self {
        Self {
            reflection: SpecularReflection::new(obj_mat),
            transmission: SpecularTransmission::new(obj_mat),
        }
    }
}

impl ShadingModelT for FresnelSpecular {
    fn bsdf(&self, tex_coords: Point2<Float>) -> BSDF {
        let brdf = self.reflection.scattering(tex_coords);
        let btdf = self.transmission.scattering(tex_coords);
        BSDF::F(FresnelBSDF::new(brdf, btdf))
    }

    fn preview_texture(&self) -> &Texture {
        &self.reflection.preview_texture()
    }
}

#[derive(Debug)]
pub struct FresnelBSDF {
    brdf: SpecularBRDF,
    btdf: SpecularBTDF,
}

impl FresnelBSDF {
    fn new(brdf: SpecularBRDF, btdf: SpecularBTDF) -> Self {
        Self {
            brdf, btdf
        }
    }
}

fn fresnel_dielectric(mut cos_i: Float, eta_mat: Float) -> Float {
    let (eta_i, eta_t) = if cos_i > 0.0 {
        (eta_mat, 1.0)
    } else {
        cos_i = -cos_i;
        (1.0, eta_mat)
    };
    let sin2_i = (1.0 - cos_i.powi(2)).max(0.0);
    let sin2_t = (eta_i / eta_t).powi(2) * sin2_i;
    // Total internal reflection
    if sin2_t >= 1.0 {
        return 1.0
    }
    let cos_t = (1.0 - sin2_t).sqrt();
    let paral = (eta_t * cos_i - eta_i * cos_t) / (eta_t * cos_i + eta_i * cos_t);
    let perp = (eta_i * cos_i - eta_t * cos_t) / (eta_i * cos_i + eta_t * cos_t);
    (paral.powi(2) + perp.powi(2)) / 2.0
}

impl BSDFT for FresnelBSDF {
    fn is_specular(&self) -> bool {
        true
    }

    fn eval(&self, _in_dir: Vector3<Float>, _out_dir: Vector3<Float>) -> Color {
        Color::black()
    }

    fn sample(&self, out_dir: Vector3<Float>) -> (Color, Vector3<Float>, Float) {
        let f = fresnel_dielectric(out_dir.z, self.btdf.eta);
        if rand::random::<Float>() < f {
            // println!("reflect: {:?}", f);
            let (color, in_dir, pdf) = self.brdf.sample(out_dir);
            (f * color, in_dir, f * pdf)
        } else {
            // println!("transmit: {:?}, {:?}", f, out_dir.z);
            let (color, in_dir, pdf) = self.btdf.sample(out_dir);
            let ft = 1.0 - f;
            (ft * color, in_dir, ft * pdf)
        }
    }
}
