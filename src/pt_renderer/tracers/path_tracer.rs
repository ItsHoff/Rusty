use cgmath::prelude::*;

use crate::bvh::BVHNode;
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
        LightMode::All => unimplemented!(), // TODO
    };
    let (li, ray, lpdf) = light.sample_li(isect);
    (li, ray, pdf * lpdf)
}

pub fn path_trace<'a>(
    mut ray: Ray,
    scene: &'a Scene,
    flash: &dyn Light,
    config: &RenderConfig,
    node_stack: &mut Vec<(&'a BVHNode, Float)>,
) -> Color {
    let mut c = Color::black();
    let mut beta = Color::white();
    let mut bounce = 0;
    let mut specular_bounce = false;
    while let Some(hit) = scene.intersect(&mut ray, node_stack) {
        let isect = hit.interaction(&config);
        if bounce == 0 || specular_bounce {
            c += beta * isect.le(&ray);
        }
        let (le, mut shadow_ray, light_pdf) = sample_light(&isect, scene, flash, config);
        if scene.intersect(&mut shadow_ray, node_stack).is_none() {
            let cos_t = isect.ns.dot(shadow_ray.dir).abs();
            c += beta * le * isect.bsdf(&ray, &shadow_ray) * cos_t / light_pdf;
        }
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
                specular_bounce = isect.is_specular();
            } else {
                break;
            }
        } else {
            break;
        }
    }
    c
}
