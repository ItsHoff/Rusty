use crate::bvh::BVHNode;
use crate::color::Color;
use crate::config::*;
use crate::float::*;
use crate::intersect::{Interaction, Ray};
use crate::light::Light;
use crate::pt_renderer::PathType;
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
    let (li, ray, lpdf) = light.sample_towards(isect);
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
            c += beta * isect.le(-ray.dir);
        }
        let (le, mut shadow_ray, light_pdf) = sample_light(&isect, scene, flash, config);
        let bsdf = isect.bsdf(-ray.dir, shadow_ray.dir, PathType::Camera);
        if !bsdf.is_black() && !scene.intersect_shadow(&mut shadow_ray, node_stack) {
            let cos_t = isect.cos_s(shadow_ray.dir).abs();
            c += beta * le * bsdf * cos_t / light_pdf;
        }
        let mut pdf = 1.0;
        let terminate = if bounce >= config.max_bounces {
            true
        } else if bounce >= config.pre_rr_bounces {
            match config.russian_roulette {
                RussianRoulette::Dynamic => {
                    // Survival probability
                    let prob = beta.luma().min(0.95);
                    pdf *= prob;
                    rand::random::<Float>() > prob
                }
                RussianRoulette::Static(prob) => {
                    pdf *= prob;
                    rand::random::<Float>() > prob
                }
                RussianRoulette::Off => false,
            }
        } else {
            false
        };
        if !terminate {
            if let Some((bsdf, new_ray, bsdf_pdf)) = isect.sample_bsdf(-ray.dir, PathType::Camera) {
                pdf *= bsdf_pdf;
                beta *= isect.cos_s(new_ray.dir).abs() * bsdf / pdf;
                ray = new_ray;
                bounce += 1;
                specular_bounce = isect.is_specular();
                if !beta.is_black() {
                    continue;
                }
            }
        }
        break;
    }
    c
}
