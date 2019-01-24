use cgmath::prelude::*;
use cgmath::Vector3;

use crate::bvh::BVHNode;
use crate::camera::PTCamera;
use crate::color::Color;
use crate::config::*;
use crate::float::*;
use crate::intersect::{Interaction, Ray};
use crate::scene::Scene;

struct Vertex<'a> {
    /// Ray that generated this vertex
    ray: Ray,
    /// Attenuation for radiance scattered from this vertex
    beta: Color,
    isect: Interaction<'a>,
}

impl<'a> Vertex<'a> {
    fn new(ray: Ray, beta: Color, isect: Interaction<'a>) -> Self {
        Self {
            ray, beta, isect
        }
    }

    fn cos_t(&self, dir: Vector3<Float>) -> Float {
        self.isect.cos_t(dir)
    }

    /// Compute the connection ray to other
    fn connection_ray(&self, other: &Self) -> Ray {
        self.isect.shadow_ray(other.isect.p)
    }

    /// Emitted radiance towards previous vertex
    fn le(&self) -> Color {
        self.beta * self.isect.le(-self.ray.dir)
    }

    /// Evaluate bsdf for continuation ray
    fn bsdf(&self, wi: Vector3<Float>) -> Color {
        self.isect.bsdf(-self.ray.dir, wi)
    }
}

#[allow(clippy::if_same_then_else)]
pub fn bdpt<'a>(
    camera_ray: Ray,
    scene: &'a Scene,
    camera: &PTCamera,
    config: &RenderConfig,
    node_stack: &mut Vec<(&'a BVHNode, Float)>,
) -> Color {
    let camera_path = generate_path(camera_ray, Color::white(), scene, config, node_stack);
    let (light, light_pdf) = match config.light_mode {
        LightMode::Scene => scene.sample_light().unwrap_or((camera.flash(), 1.0)),
        LightMode::Camera => (camera.flash(), 1.0),
    };
    let (le, light_ray, light_n, area_pdf, dir_pdf) = light.sample_le();
    let light_beta = le * light_n.dot(light_ray.dir).abs() / (light_pdf * area_pdf * dir_pdf);
    let light_path = generate_path(light_ray, light_beta, scene, config, node_stack);
    let mut c = Color::black();
    // Paths contain vertices after the light / camera
    // 0 corresponds to no vertices from that subpath,
    // 1 is the implicit starting vertex
    // 2+ are regular path vertices
    for s in 0..light_path.len() + 2 {
        // Light path can't hit camera so start t from 1
        for t in 1..camera_path.len() + 2 {
            // No light vertices
            let radiance = if s == 0 {
                if let Some(c_vertex) = camera_path.get(t - 2) {
                    c_vertex.le()
                } else {
                    continue;
                }
            // Connect camera and light
            } else if s == 1 && t == 1 {
                // TODO
                continue;
            // Connect light vertex to camera
            } else if t == 1 {
                // TODO
                continue;
            // Connect camera vertex to light
            } else if s == 1 {
                let c_vertex = &camera_path[t - 2];
                let (li, mut shadow_ray, li_pdf) = light.sample_li(&c_vertex.isect);
                let bsdf = c_vertex.bsdf(shadow_ray.dir);
                if !bsdf.is_black() && scene.intersect(&mut shadow_ray, node_stack).is_none() {
                    let cos_t = c_vertex.cos_t(shadow_ray.dir);
                    c_vertex.beta * li * bsdf * cos_t / (li_pdf * light_pdf)
                } else {
                    continue;
                }
            // Everything else
            } else {
                let l_vertex = &light_path[s - 2];
                let c_vertex = &camera_path[t - 2];
                let mut connection = c_vertex.connection_ray(l_vertex);
                let radiance = c_vertex.beta * c_vertex.bsdf(connection.dir)
                    * l_vertex.beta * l_vertex.bsdf(-connection.dir);
                if !radiance.is_black() && scene.intersect(&mut connection, node_stack).is_none() {
                    let length = connection.length;
                    let dir = connection.dir;
                    let g = c_vertex.cos_t(dir) * l_vertex.cos_t(dir) / length.powi(2);
                    g * radiance
                } else {
                    continue;
                }
            };
            let n_scatter = s + t - 2;
            // There are currently n + 1 ways to construct path with n scattering events
            // TODO: implement MIS
            c += radiance / (n_scatter + 1).to_float();
        }
    }
    c
}

fn generate_path<'a>(
    mut ray: Ray,
    mut beta: Color,
    scene: &'a Scene,
    config: &RenderConfig,
    node_stack: &mut Vec<(&'a BVHNode, Float)>,
) -> Vec<Vertex<'a>> {
    let mut bounce = 0;
    let mut path = Vec::new();
    while let Some(hit) = scene.intersect(&mut ray, node_stack) {
        path.push(Vertex::new(ray.clone(), beta, hit.interaction(&config)));
        let isect = &path.last().unwrap().isect;
        let mut pdf = 1.0;
        let terminate = if bounce < config.bounces {
            false
        } else if config.russian_roulette {
            // Survival probability
            let prob = beta.luma().min(0.95);
            pdf *= prob;
            rand::random::<Float>() > prob
        } else {
            true
        };
        if !terminate {
            // TODO: account for non-symmetry
            if let Some((brdf, new_ray, brdf_pdf)) = isect.sample_bsdf(-ray.dir) {
                ray = new_ray;
                pdf *= brdf_pdf;
                beta *= isect.cos_t(ray.dir) * brdf / pdf;
                bounce += 1;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    path
}
