use crate::bvh::BVHNode;
use crate::camera::PTCamera;
use crate::color::Color;
use crate::config::*;
use crate::float::*;
use crate::intersect::Ray;
use crate::light::Light;
use crate::scene::Scene;

mod vertex;

use self::vertex::*;

/// Convert path pdfs to area measure and compute reverse pdfs.
fn precompute_mis(path: &mut Vec<Vertex>) -> Vec<Float> {
    let mut pdf_rev = Vec::new();
    for i in 0..path.len() - 1 {
        // Split to satisfy borrow check
        let (left, right) = path.split_at_mut(i + 1);
        let v = &left[i];
        let vn = &mut right[0];
        vn.convert_pdf(&v);
        if i > 0 {
            let vp = &left[i - 1];
            pdf_rev.push(Vertex::pdf(vn, v, vp));
        }
    }
    // Add last two elements
    pdf_rev.extend_from_slice(&[0.0, 0.0]);
    pdf_rev
}

#[allow(clippy::if_same_then_else)]
pub fn bdpt<'a>(
    camera_ray: Ray,
    scene: &'a Scene,
    camera: &'a PTCamera,
    config: &RenderConfig,
    node_stack: &mut Vec<(&'a BVHNode, Float)>,
) -> Color {
    let camera_vertex = Vertex::camera(camera, camera_ray);
    let mut camera_path = generate_path(camera_vertex, scene, config, node_stack);
    let (light, light_pdf) = match config.light_mode {
        LightMode::Scene => scene.sample_light().unwrap_or((camera.flash(), 1.0)),
        LightMode::Camera => (camera.flash(), 1.0),
    };
    let (light_pos, pos_pdf) = light.sample_pos();
    let light_vertex = Vertex::light(light, light_pos, light_pdf * pos_pdf);
    let mut light_path = generate_path(light_vertex, scene, config, node_stack);
    let mut light_rev = precompute_mis(&mut light_path);
    if light_rev[0].is_nan() {
        dbg!(&light_rev);
        dbg!(&light_path);
    }
    let camera_rev = precompute_mis(&mut camera_path);
    let mut c = Color::black();
    // Paths contain vertices after the light / camera
    // 0 corresponds to no vertices from that subpath,
    // 1 is the starting vertex
    // 2+ are regular path vertices
    for s in (0..=light_path.len()).rev() {
        // Get a fresh each iteration
        let mut camera_rev = camera_rev.clone();
        // Light path can't hit camera so start t from 1
        for t in (1..=camera_path.len()).rev() {
            // TODO: handle rr
            if s + t - 2 > config.bounces {
                continue;
            }
            // No light vertices
            let radiance = if s == 0 {
                if t > 1 {
                    let vertex = camera_path[t - 1].get_surface().unwrap();
                    if let Some(light) = vertex.get_light() {
                        let pdf_light = scene.pdf_light(light);
                        camera_rev[t - 1] = pdf_light * light.pdf_pos();
                        if t > 2 {
                            camera_rev[t - 2] = light.pdf_dir(vertex.ray.dir);
                        }
                        vertex.path_radiance()
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
            // Everything else
            } else {
                let l_vertex = &light_path[s - 1];
                let c_vertex = &camera_path[t - 1];
                let (mut connection_ray, radiance) = l_vertex.connect_to_surface(c_vertex);
                if !radiance.is_black()
                    && scene.intersect(&mut connection_ray, node_stack).is_none()
                {
                    // Update reverse pdfs
                    // Since we iterate in reverse overwriting is fine.
                    if s > 1 {
                        let l_prev = &light_path[s - 2];
                        light_rev[s - 2] = Vertex::pdf(c_vertex, l_vertex, l_prev);
                        camera_rev[t - 1] = Vertex::pdf(l_prev, l_vertex, c_vertex);
                    }
                    if t > 1 {
                        let c_prev = &camera_path[t - 2];
                        camera_rev[t - 2] = Vertex::pdf(l_vertex, c_vertex, c_prev);
                        light_rev[s - 1] = Vertex::pdf(c_prev, c_vertex, l_vertex);
                    }
                    radiance
                } else {
                    continue;
                }
            };
            // Compute mis weight
            let mut sum_ri = 0.0;
            let mut ri = 1.0;
            let remap0 = |x| if x == 0.0 { 1.0 } else { x };
            for i in (2..t - 1).rev() {
                ri *= remap0(camera_rev[i]) / remap0(camera_path[i].pdf_fwd());
                if !camera_path[i].is_delta() && !camera_path[i - 1].is_delta() {
                    sum_ri += ri;
                }
            }
            ri = 1.0;
            if s > 0 {
                for i in (0..s - 1).rev() {
                    ri *= remap0(light_rev[i]) / remap0(light_path[i].pdf_fwd());
                    let prev_delta = if i == 0 {
                        false
                    } else {
                        light_path[i - 1].is_delta()
                    };
                    if !light_path[i].is_delta() && !prev_delta {
                        sum_ri += ri;
                    }
                }
            }
            if true {
                // mis
                c += radiance / (1.0 + sum_ri);
            } else {
                // uniform scale
                c += radiance / (s + t - 1).to_float();
            }
        }
    }
    c
}

fn generate_path<'a>(
    vertex: Vertex<'a>,
    scene: &'a Scene,
    config: &RenderConfig,
    node_stack: &mut Vec<(&'a BVHNode, Float)>,
) -> Vec<Vertex<'a>> {
    let (mut beta, mut ray, mut pdf_fwd) = vertex.sample_next().unwrap();
    let mut bounce = 0;
    let mut path = vec![vertex];
    while let Some(hit) = scene.intersect(&mut ray, node_stack) {
        path.push(Vertex::surface(
            ray.clone(),
            beta,
            hit.interaction(&config),
            pdf_fwd,
        ));
        let vertex = path.last().unwrap();
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
            if let Some((brdf, new_ray, brdf_pdf)) = vertex.sample_next() {
                pdf_fwd = brdf_pdf;
                ray = new_ray;
                pdf *= brdf_pdf;
                beta *= vertex.cos_t(ray.dir).abs() * brdf / pdf;
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
