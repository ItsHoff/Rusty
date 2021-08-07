use cgmath::Point2;

use crate::bvh::BvhNode;
use crate::camera::PtCamera;
use crate::color::Color;
use crate::config::*;
use crate::float::*;
use crate::intersect::Ray;
use crate::pt_renderer::PathType;
use crate::scene::Scene;

mod vertex;

use self::vertex::*;

// TODO: avoid allocations
pub fn bdpt<'a>(
    camera_ray: Ray,
    scene: &'a Scene,
    camera: &'a PtCamera,
    config: &RenderConfig,
    node_stack: &mut Vec<(&'a BvhNode, Float)>,
    splats: &mut Vec<(Point2<Float>, Color)>,
) -> Color {
    let camera_vertex = CameraVertex::new(camera, camera_ray);
    let (beta, ray) = camera_vertex.sample_next();
    let camera_path = generate_path(beta, ray, PathType::Camera, scene, config, node_stack);
    let (light, light_pdf) = match config.light_mode {
        LightMode::Scene => scene.sample_light().unwrap_or((camera.flash(), 1.0)),
        LightMode::Camera => (camera.flash(), 1.0),
    };
    let (light_pos, pos_pdf) = light.sample_pos();
    let light_vertex = LightVertex::new(light, light_pos, light_pdf * pos_pdf);
    let (beta, ray) = light_vertex.sample_next();
    let light_path = generate_path(beta, ray, PathType::Light, scene, config, node_stack);
    let bd_path = BdPath::new(
        &light_vertex,
        &light_path,
        &camera_vertex,
        &camera_path,
        config,
    );
    let mut c = Color::black();
    // Paths contain vertices after the light / camera
    // 0 corresponds to no vertices from that subpath,
    // 1 is the starting vertex
    // 2+ are regular path vertices
    for s in (0..=light_path.len() + 1).rev() {
        // Light path can't hit camera so start t from 1
        for t in (1..=camera_path.len() + 1).rev() {
            let length = s + t;
            if length < 2 || length - 2 > config.max_bounces {
                continue;
            }
            let mut splat = None;
            // No light vertices
            let (mut radiance, path) = if s == 0 {
                if let Some(vertex) = camera_path.get(t - 2) {
                    if let Some(light_vertex) = vertex.to_light_vertex(scene) {
                        (
                            vertex.path_radiance(),
                            bd_path.subpath_with_light(light_vertex, t),
                        )
                    } else {
                        continue;
                    }
                } else {
                    continue;
                }
            // Connect camera and light
            } else if s == 1 && t == 1 {
                // This should be sampled well enough by strategy (0, 2)
                continue;
            // Everything else
            } else {
                let l_vertex: &dyn Vertex = if s == 1 {
                    &light_vertex
                } else {
                    &light_path[s - 2]
                };
                let c_vertex: &dyn Vertex = if t == 1 {
                    &camera_vertex
                } else {
                    &camera_path[t - 2]
                };
                // Connect camera vertex to light vertex since shadow rays
                // from the camera are simpler than those from the light
                let (mut connection_ray, radiance) = c_vertex.connect_to(l_vertex);
                if !radiance.is_black() && !scene.intersect_shadow(&mut connection_ray, node_stack)
                {
                    if t == 1 {
                        // Splat is always valid if radiance is not black
                        splat = camera_vertex.camera.clip_pos(connection_ray.dir);
                    }
                    (radiance, bd_path.subpath(s, t))
                } else {
                    continue;
                }
            };
            radiance *= path.weight();
            if let Some(clip_p) = splat.take() {
                splats.push((clip_p, radiance));
            } else {
                c += radiance;
            }
        }
    }
    c
}

fn generate_path<'a>(
    mut beta: Color,
    mut ray: Ray,
    path_type: PathType,
    scene: &'a Scene,
    config: &RenderConfig,
    node_stack: &mut Vec<(&'a BvhNode, Float)>,
) -> Vec<SurfaceVertex<'a>> {
    let mut bounce = 0;
    let mut path = Vec::new();
    while let Some(hit) = scene.intersect(&mut ray, node_stack) {
        path.push(SurfaceVertex::new(
            ray.clone(),
            beta,
            path_type,
            hit.interaction(config),
        ));
        let isect = &path.last().unwrap().isect;
        let mut pdf = 1.0;
        let terminate = if bounce >= config.max_bounces {
            true
        } else if bounce >= config.pre_rr_bounces {
            match config.russian_roulette {
                RussianRoulette::Dynamic => panic!("Bdpt does not support dynamic RR"),
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
            if let Some((bsdf, new_ray, bsdf_pdf)) = isect.sample_bsdf(-ray.dir, path_type) {
                pdf *= bsdf_pdf;
                beta *= isect.cos_s(new_ray.dir).abs() * bsdf / pdf;
                ray = new_ray;
                bounce += 1;
                if !beta.is_black() {
                    continue;
                }
            }
        }
        break;
    }
    path
}
