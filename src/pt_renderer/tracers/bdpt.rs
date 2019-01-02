use cgmath::prelude::*;

use crate::bvh::BVHNode;
use crate::camera::PTCamera;
use crate::color::Color;
use crate::config::*;
use crate::float::*;
use crate::intersect::{Interaction, Ray};
use crate::light::Light;
use crate::scene::Scene;

fn sample_light(
    isect: &Interaction,
    scene: &Scene,
    flash: &dyn Light,
    config: &RenderConfig,
) -> (Color, Ray, Float) {
    let (light, pdf) = match config.light_mode {
        LightMode::Scene => scene.sample_light().unwrap_or((flash, 1.0)),
        LightMode::Camera => (flash, 1.0),
    };
    let (li, ray, lpdf) = light.sample_li(isect);
    (li, ray, pdf * lpdf)
}

pub fn bdpt<'a>(
    camera_ray: Ray,
    scene: &'a Scene,
    camera: &PTCamera,
    config: &RenderConfig,
    node_stack: &mut Vec<(&'a BVHNode, Float)>,
) -> Color {
    let path = generate_path(camera_ray, scene, config, node_stack);
    let mut c = Color::black();
    for (i, (ray, beta, isect)) in path.iter().enumerate() {
        if i == 0 {
            c += *beta * isect.le(&ray);
        }
        if isect.is_specular() {
            if let Some((ray_n, beta_n, next)) = path.get(i + 1) {
                c += *beta_n * next.le(&ray_n);
            }
        } else {
            let (le, mut shadow_ray, light_pdf) =
                sample_light(&isect, scene, camera.flash(), config);
            let bsdf = isect.bsdf(&ray, &shadow_ray);
            if !bsdf.is_black() && scene.intersect(&mut shadow_ray, node_stack).is_none() {
                let cos_t = isect.ns.dot(shadow_ray.dir).abs();
                c += *beta * le * bsdf * cos_t / light_pdf;
            }
        }
    }
    c
}

fn generate_path<'a>(
    mut ray: Ray,
    scene: &'a Scene,
    config: &RenderConfig,
    node_stack: &mut Vec<(&'a BVHNode, Float)>,
) -> Vec<(Ray, Color, Interaction<'a>)> {
    let mut bounce = 0;
    let mut beta = Color::white();
    let mut path = Vec::new();
    while let Some(hit) = scene.intersect(&mut ray, node_stack) {
        path.push((ray.clone(), beta, hit.interaction(&config)));
        let (_, _, isect) = path.last().unwrap();
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
            if let Some((brdf, new_ray, brdf_pdf)) = isect.sample_bsdf(&ray) {
                ray = new_ray;
                pdf *= brdf_pdf;
                beta *= isect.ns.dot(ray.dir).abs() * brdf / pdf;
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
