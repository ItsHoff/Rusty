use std::f32::consts::PI;
use std::sync::{Arc, Mutex, mpsc::{Sender, Receiver, TryRecvError}};

use cgmath::{Point3, Vector3, Vector4, prelude::*};

use glium::Rect;

use rand::{self, prelude::*};

use crate::bvh::BVHNode;
use crate::camera::Camera;
use crate::material::Material;
use crate::pt_renderer::{Intersect, Ray, RenderCoordinator};
use crate::scene::Scene;
use crate::triangle::{Hit, RTTriangle};

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
}

impl RenderWorker {
    pub fn new(scene: Arc<Scene>, camera: Camera, coordinator: Arc<Mutex<RenderCoordinator>>,
               message_rx: Receiver<()>, result_tx: Sender<(Rect, Vec<f32>)>) -> RenderWorker {
        RenderWorker {
            scene, camera, coordinator,
            message_rx, result_tx,
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
            if let Some(rect) = self.coordinator.lock().unwrap().next_block() {
                let mut block = vec![0.0f32; (3 * rect.width * rect.height) as usize];
                for h in 0..rect.height {
                    for w in 0..rect.width {
                        let samples_per_dir = 2u32;
                        let mut c = Vector3::zero();
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
                                let ray = Ray::new(self.camera.pos, dir, 100.0);
                                c += self.trace_ray(&ray, &mut node_stack, 0);
                            }
                        }
                        c /= samples_per_dir.pow(2) as f32;
                        let pixel_i = 3 * (h * rect.width + w) as usize;
                        block[pixel_i]     = c.x;
                        block[pixel_i + 1] = c.y;
                        block[pixel_i + 2] = c.z;
                    }
                }
                self.result_tx.send((rect, block)).expect("Receiver closed!");
            } else {
                return;
            }
        }
    }

    fn trace_ray(&'a self, ray: &Ray, node_stack: &mut Vec<(&'a BVHNode, f32)>, bounce: u32) -> Vector3<f32> {
        let mut c = Vector3::zero();
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
            let (light, light_pos, light_normal, light_pdf) = self.sample_light();
            let bump_pos = hit.pos() + EPSILON * normal;
            let hit_to_light = light_pos - bump_pos;
            let light_dir = hit_to_light.normalize();
            let shadow_ray = Ray::new(bump_pos, light_dir, hit_to_light.magnitude() - EPSILON);
            if shadow_ray.length > 0.1 && self.find_hit(&shadow_ray, node_stack).is_none() {
                let cos_l = light_normal.dot(-light_dir).max(0.0);
                let cos_t = normal.dot(light_dir).max(0.0);
                let light_material = &self.scene.materials[light.material_i];
                let emissive = light_material.emissive.expect("Light wasn't emissive");
                c += emissive.mul_element_wise(self.brdf(&ray, &shadow_ray, material))
                    * cos_l * cos_t / (hit_to_light.magnitude2() * light_pdf);
            }
            let rr: f32 = rand::random();
            if rr < RR_PROB {
                let (new_dir, mut pdf) = self.sample_dir(normal);
                pdf *= RR_PROB;
                let new_ray = Ray::new(bump_pos, new_dir, 100.0);
                c += normal.dot(new_dir) * self.brdf(&ray, &new_ray, material)
                    .mul_element_wise(self.trace_ray(&new_ray, node_stack, bounce + 1)) / pdf;
            }
        }
        c
    }

    fn brdf(&self, _in_ray: &Ray, _out_ray: &Ray, material: &Material) -> Vector3<f32> {
        material.diffuse / PI
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

    pub fn sample_light(&self) -> (&RTTriangle, Point3<f32>, Vector3<f32>, f32) {
        if self.scene.lights.is_empty() {
            panic!("Rendered scene has no lights!");
        } else {
            let i = rand::thread_rng().gen_range(0, self.scene.lights.len());
            let light = &self.scene.lights[i];
            let pdf = 1.0 / (self.scene.lights.len() as f32 * light.area());
            let (point, normal) = light.random_point();
            (light, point, normal, pdf)
        }
    }

    fn find_hit(&'a self, ray: &Ray, node_stack: &mut Vec<(&'a BVHNode, f32)>) -> Option<Hit> {
        let bvh = &self.scene.bvh;
        node_stack.push((bvh.root(), 0.0f32));
        let mut closest_hit: Option<Hit> = None;
        while let Some((node, t)) = node_stack.pop() {
            // We've already found closer hit
            if closest_hit.as_ref().map_or(false, |hit| hit.t <= t) { continue }
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
                // TODO: put closest hit on top of the stack
                let (left, right) = bvh.get_children(node).expect("Non leaf node had no child nodes!");
                if let Some(t_left) = left.intersect(&ray) {
                    node_stack.push((left, t_left));
                }
                if let Some(t_right) = right.intersect(&ray) {
                    node_stack.push((right, t_right));
                }
            }
        }
        closest_hit
    }
}
