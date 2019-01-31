use crate::bvh::BVHNode;
use crate::camera::PTCamera;
use crate::color::Color;
use crate::config::*;
use crate::float::*;
use crate::intersect::Ray;
use crate::scene::Scene;

mod vertex;

use self::vertex::*;

#[allow(clippy::if_same_then_else)]
pub fn bdpt<'a>(
    camera_ray: Ray,
    scene: &'a Scene,
    camera: &'a PTCamera,
    config: &RenderConfig,
    node_stack: &mut Vec<(&'a BVHNode, Float)>,
) -> Color {
    let camera_vertex = CameraVertex::new(camera, camera_ray);
    let (beta, ray) = camera_vertex.sample_next();
    let camera_path = generate_path(beta, ray, scene, config, node_stack);
    let (light, light_pdf) = match config.light_mode {
        LightMode::Scene => scene.sample_light().unwrap_or((camera.flash(), 1.0)),
        LightMode::Camera => (camera.flash(), 1.0),
    };
    let (light_pos, pos_pdf) = light.sample_pos();
    let light_vertex = LightVertex::new(light, light_pos, light_pdf * pos_pdf);
    let (beta, ray) = light_vertex.sample_next();
    let light_path = generate_path(beta, ray, scene, config, node_stack);
    let mut c = Color::black();
    // Paths contain vertices after the light / camera
    // 0 corresponds to no vertices from that subpath,
    // 1 is the starting vertex
    // 2+ are regular path vertices
    for s in (0..light_path.len() + 2).rev() {
        // Light path can't hit camera so start t from 1
        for t in (1..camera_path.len() + 2).rev() {
            // TODO: handle rr
            if s + t < 2 || s + t - 2 > config.bounces {
                continue;
            }
            // No light vertices
            let (radiance, path) = if s == 0 {
                if let Some(vertex) = camera_path.get(t - 2) {
                    if let Some(light_vertex) = vertex.to_light_vertex(&scene) {
                        let path = BDPath::new(light_vertex, &[],
                                               &camera_vertex, &camera_path[0..t - 2]);
                        (vertex.path_radiance(), path)
                    } else {
                        continue;
                    }
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
                let (mut connection_ray, radiance) = light_vertex.connect_to(c_vertex);
                if !radiance.is_black() && scene.intersect(&mut connection_ray, node_stack).is_none() {
                    let path = BDPath::new(light_vertex.clone(), &[],
                                           &camera_vertex, &camera_path[0..=t - 2]);
                    (radiance, path)
                } else {
                    continue;
                }
            // Everything else
            } else {
                let l_vertex = &light_path[s - 2];
                let c_vertex = &camera_path[t - 2];
                let (mut connection_ray, radiance) = l_vertex.connect_to(c_vertex);
                if !radiance.is_black() && scene.intersect(&mut connection_ray, node_stack).is_none() {
                    let path = BDPath::new(light_vertex.clone(), &light_path[0..=s - 2],
                                           &camera_vertex, &camera_path[0..=t - 2]);
                    (radiance, path)
                } else {
                    continue;
                }
            };
            if true {
                // MIS
                let weight = if s + t == 2 {
                    1.0
                } else {
                    let pdf_strat = path.pdf(s, t).unwrap();
                    let mut sum_pdf = 0.0;
                    for i in 2..=s + t {
                        if let Some(pdf) = path.pdf(s + t - i, i) {
                            sum_pdf += pdf;
                        }
                    }
                    pdf_strat / sum_pdf
                };
                c += weight * radiance;
            } else {
                // uniform scale
                c += radiance / (s + t - 1).to_float();
            }
        }
    }
    c
}

fn generate_path<'a>(
    mut beta: Color,
    mut ray: Ray,
    scene: &'a Scene,
    config: &RenderConfig,
    node_stack: &mut Vec<(&'a BVHNode, Float)>,
) -> Vec<SurfaceVertex<'a>> {
    let mut bounce = 0;
    let mut path = Vec::new();
    while let Some(hit) = scene.intersect(&mut ray, node_stack) {
        path.push(SurfaceVertex::new(
            ray.clone(),
            beta,
            hit.interaction(&config),
        ));
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
            if let Some((bsdf, new_ray, bsdf_pdf)) = isect.sample_bsdf(-ray.dir) {
                ray = new_ray;
                pdf *= bsdf_pdf;
                beta *= isect.cos_t(ray.dir).abs() * bsdf / pdf;
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
