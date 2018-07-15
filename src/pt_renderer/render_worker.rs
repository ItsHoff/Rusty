use std::f32::consts::PI;
use std::sync::{Arc, Mutex,
                mpsc::{Sender, Receiver, TryRecvError},
                atomic::{AtomicUsize, Ordering},
};

use cgmath::{Point3, Vector3, Vector4, prelude::*};

use glium::Rect;

use rand::{self, prelude::*};

use crate::bvh::BVHNode;
use crate::camera::Camera;
use crate::color::Color;
use crate::material::Material;
use crate::pt_renderer::{Intersect, Ray, RenderCoordinator};
use crate::scene::Scene;
use crate::triangle::Hit;

// TODO: tune EPSILON since crytek-sponza has shadow acne
const EPSILON: f32 = 1e-5;
const _EB: f32 = 5.0; // Desired expectation value of bounces
// The matching survival probability from negative binomial distribution
const RR_PROB: f32 = _EB / (_EB + 1.0);

pub struct RenderWorker {
    scene: Arc<Scene>,
    camera: Camera,
    coordinator: Arc<Mutex<RenderCoordinator>>,
    message_rx: Receiver<()>,
    result_tx: Sender<(Rect, Vec<f32>)>,
    ray_count: Arc<AtomicUsize>,
}

impl RenderWorker {
    pub fn new(scene: Arc<Scene>, camera: Camera, coordinator: Arc<Mutex<RenderCoordinator>>,
               message_rx: Receiver<()>, result_tx: Sender<(Rect, Vec<f32>)>,
               ray_count: Arc<AtomicUsize>) -> RenderWorker {
        RenderWorker {
            scene, camera, coordinator,
            message_rx, result_tx,
            ray_count,
        }
    }

    pub fn run(&self) {
        let (width, height) = {
            let coordinator = self.coordinator.lock().unwrap();
            (coordinator.width, coordinator.height)
        };
        let clip_to_world = self.camera.get_world_to_clip().invert().unwrap();
        let mut node_stack: Vec<(&BVHNode, f32)> = Vec::new();
        loop {
            match self.message_rx.try_recv() {
                Err(TryRecvError::Empty) => (),
                Ok(_) => return,
                Err(TryRecvError::Disconnected) => {
                    println!("Threads were not properly stopped before disconnecting channel!");
                    return;
                }
            }
            let block = {
                let mut coordinator = self.coordinator.lock().unwrap();
                coordinator.next_block()
            };
            if let Some(rect) = block {
                let mut block = vec![0.0f32; (3 * rect.width * rect.height) as usize];
                for h in 0..rect.height {
                    for w in 0..rect.width {
                        let samples_per_dir = 2u32;
                        let mut c = Color::black();
                        for j in 0..samples_per_dir {
                            for i in 0..samples_per_dir {
                                let dx = (i as f32 + rand::random::<f32>()) / samples_per_dir as f32;
                                let dy = (j as f32 + rand::random::<f32>()) / samples_per_dir as f32;
                                let clip_x = 2.0 * ((rect.left + w) as f32 + dx) / width as f32 - 1.0;
                                let clip_y = 2.0 * ((rect.bottom + h) as f32 + dy) / height as f32 - 1.0;
                                let clip_p = Vector4::new(clip_x, clip_y, 1.0, 1.0);
                                let world_p = clip_to_world * clip_p;
                                let dir = ((world_p / world_p.w).truncate() - self.camera.pos.to_vec())
                                    .normalize();
                                let ray = Ray::new(self.camera.pos, dir, 10.0 * self.camera.scale);
                                c += self.trace_ray(&ray, &mut node_stack, 0);
                            }
                        }
                        c /= samples_per_dir.pow(2) as f32;
                        let pixel_i = 3 * (h * rect.width + w) as usize;
                        block[pixel_i]     = c.r;
                        block[pixel_i + 1] = c.g;
                        block[pixel_i + 2] = c.b;
                    }
                }
                self.result_tx.send((rect, block)).expect("Receiver closed!");
            } else {
                return;
            }
        }
    }

    fn trace_ray(&'a self, ray: &Ray, node_stack: &mut Vec<(&'a BVHNode, f32)>, bounce: u32) -> Color {
        let mut c = Color::black();
        if let Some(hit) = self.find_hit(&ray, node_stack) {
            let material = &self.scene.materials[hit.tri.material_i];
            let mut normal = hit.normal();
            // Flip the normal if its pointing to the opposite side from the hit
            if normal.dot(ray.dir) > 0.0 {
                normal *= -1.0;
            }
            if bounce == 0 {
                if let Some(emissive) = material.emissive {
                    c += normal.dot(-ray.dir).max(0.0) * emissive;
                }
            }
            let (emissive, light_pos, light_normal, light_pdf) = self.sample_light();
            let bump_pos = hit.pos() + EPSILON * normal;
            let hit_to_light = light_pos - bump_pos;
            let light_dir = hit_to_light.normalize();
            let shadow_ray = Ray::new(bump_pos, light_dir, hit_to_light.magnitude() - EPSILON);
            if self.find_hit(&shadow_ray, node_stack).is_none() {
                let cos_l = light_normal.dot(-light_dir).max(0.0);
                let cos_t = normal.dot(light_dir).max(0.0);
                c += emissive * self.brdf(&hit, &ray, &shadow_ray, material)
                    * cos_l * cos_t / (hit_to_light.magnitude2() * light_pdf);
            }
            let rr: f32 = rand::random();
            if rr < RR_PROB {
                let (new_dir, mut pdf) = self.sample_dir(normal);
                pdf *= RR_PROB;
                let new_ray = Ray::new(bump_pos, new_dir, 10.0 * self.camera.scale);
                c += normal.dot(new_dir) * self.brdf(&hit, &ray, &new_ray, material)
                    * self.trace_ray(&new_ray, node_stack, bounce + 1) / pdf;
            }
        }
        c
    }

    fn brdf(&self, hit: &Hit, _in_ray: &Ray, _out_ray: &Ray, material: &Material) -> Color {
        let tex_coords = hit.tex_coords();
        material.diffuse(tex_coords) / PI
    }

    fn sample_dir(&self, normal: Vector3<f32>) -> (Vector3<f32>, f32) {
        let dir = 2.0 * PI * rand::random::<f32>();
        let length: f32 = rand::random();
        let x = length * dir.cos();
        let y = length * dir.sin();
        let z = (1.0 - length.powi(2)).sqrt();
        let nx = if normal.x.abs() > normal.y.abs() {
            Vector3::new(normal.z, 0.0, -normal.x).normalize()
        } else {
            Vector3::new(0.0, -normal.z, normal.y).normalize()
        };
        let ny = normal.cross(nx);
        (x * nx + y * ny + z * normal, z / PI)
    }

    pub fn sample_light(&self) -> (Color, Point3<f32>, Vector3<f32>, f32) {
        if self.scene.lights.is_empty() {
            let light_intensity = 10.0 * self.camera.scale;
            (light_intensity * Color::white(), self.camera.pos, self.camera.dir, 1.0)
        } else {
            let i = rand::thread_rng().gen_range(0, self.scene.lights.len());
            let light = &self.scene.lights[i];
            let pdf = 1.0 / (self.scene.lights.len() as f32 * light.area());
            let (point, normal) = light.random_point();
            let light_material = &self.scene.materials[light.material_i];
            let emissive = light_material.emissive.expect("Light wasn't emissive");
            (emissive, point, normal, pdf)
        }
    }

    fn find_hit(&'a self, ray: &Ray, node_stack: &mut Vec<(&'a BVHNode, f32)>) -> Option<Hit> {
        self.ray_count.fetch_add(1, Ordering::Relaxed);
        let bvh = &self.scene.bvh;
        node_stack.push((bvh.root(), 0.0f32));
        let mut closest_hit: Option<Hit> = None;
        while let Some((node, t)) = node_stack.pop() {
            // We've already found closer hit
            if closest_hit.as_ref().map_or(false, |closest| closest.t <= t) { continue }
            if node.is_leaf() {
                for tri in &self.scene.triangles[node.start_i..node.end_i] {
                    if let Some(hit) = tri.intersect(&ray) {
                        if let Some(closest) = closest_hit.take() {
                            if hit.t < closest.t {
                                closest_hit = Some(hit);
                            } else {
                                closest_hit = Some(closest);
                            }
                        } else {
                            closest_hit = Some(hit);
                        }
                    }
                }
            } else {
                let (left, right) = bvh.get_children(node)
                    .expect("Non leaf node had no child nodes!");
                // TODO: benchmark this properly on the larger scenes
                if false { // not sorted
                    if let Some(t_left) = left.intersect(&ray) {
                        node_stack.push((left, t_left));
                    }
                    if let Some(t_right) = right.intersect(&ray) {
                        node_stack.push((right, t_right));
                    }
                } else { // sort nodes
                    let left_intersect = left.intersect(&ray);
                    let right_intersect = right.intersect(&ray);
                    if let Some(t_left) = left_intersect {
                        if let Some(t_right) = right_intersect {
                            // Put the closer hit on top
                            if t_left >= t_right {
                                node_stack.push((left, t_left));
                                node_stack.push((right, t_right));
                            } else {
                                node_stack.push((right, t_right));
                                node_stack.push((left, t_left));
                            }
                        } else {
                            node_stack.push((left, t_left));
                        }
                    } else if let Some(t_right) = right_intersect {
                        node_stack.push((right, t_right));
                    }
                }
            }
        }
        closest_hit
    }
}
