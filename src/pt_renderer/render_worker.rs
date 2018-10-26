use std::sync::{
    mpsc::{Receiver, Sender, TryRecvError},
    Arc,
};

use cgmath::prelude::*;
use cgmath::{Point3, Vector4};

use glium::Rect;

use crate::bvh::BVHNode;
use crate::camera::Camera;
use crate::color::Color;
use crate::intersect::{Interaction, Ray};
use crate::pt_renderer::RenderCoordinator;
use crate::scene::Scene;
use crate::Float;

// Desired expectation value of bounces
const _EB: Float = 5.0;
// The matching survival probability from negative binomial distribution
const RR_PROB: Float = _EB / (_EB + 1.0);

pub struct RenderWorker {
    scene: Arc<Scene>,
    camera: Camera,
    coordinator: Arc<RenderCoordinator>,
    message_rx: Receiver<()>,
    result_tx: Sender<(Rect, Vec<f32>)>,
}

impl RenderWorker {
    pub fn new(
        scene: Arc<Scene>,
        camera: Camera,
        coordinator: Arc<RenderCoordinator>,
        message_rx: Receiver<()>,
        result_tx: Sender<(Rect, Vec<f32>)>,
    ) -> RenderWorker {
        RenderWorker {
            scene,
            camera,
            coordinator,
            message_rx,
            result_tx,
        }
    }

    #[allow(clippy::cast_lossless)]
    pub fn run(&self) {
        let (width, height) = (self.coordinator.width, self.coordinator.height);
        let clip_to_world = self.camera.world_to_clip().invert().unwrap();
        let mut node_stack: Vec<(&BVHNode, Float)> = Vec::new();
        loop {
            match self.message_rx.try_recv() {
                Err(TryRecvError::Empty) => (),
                Ok(_) => return,
                Err(TryRecvError::Disconnected) => {
                    println!("Threads were not properly stopped before disconnecting channel!");
                    return;
                }
            }
            if let Some(rect) = self.coordinator.next_block() {
                let mut block = vec![0.0f32; (3 * rect.width * rect.height) as usize];
                for h in 0..rect.height {
                    for w in 0..rect.width {
                        let samples_per_dir = 2u32;
                        let mut c = Color::black();
                        for j in 0..samples_per_dir {
                            for i in 0..samples_per_dir {
                                let dx = (i as Float + rand::random::<Float>())
                                    / samples_per_dir as Float;
                                let dy = (j as Float + rand::random::<Float>())
                                    / samples_per_dir as Float;
                                let clip_x =
                                    2.0 * ((rect.left + w) as Float + dx) / width as Float - 1.0;
                                let clip_y =
                                    2.0 * ((rect.bottom + h) as Float + dy) / height as Float - 1.0;
                                let clip_p = Vector4::new(clip_x, clip_y, 1.0, 1.0);
                                let world_p = Point3::from_homogeneous(clip_to_world * clip_p);
                                let ray = Ray::from_point(self.camera.pos, world_p);
                                c += self.trace_ray(ray, &mut node_stack, 0);
                            }
                        }
                        c /= samples_per_dir.pow(2) as Float;
                        let pixel_i = 3 * (h * rect.width + w) as usize;
                        block[pixel_i] = c.r as f32;
                        block[pixel_i + 1] = c.g as f32;
                        block[pixel_i + 2] = c.b as f32;
                    }
                }
                self.result_tx
                    .send((rect, block))
                    .expect("Receiver closed!");
            } else {
                return;
            }
        }
    }

    fn trace_ray<'a>(
        &'a self,
        mut ray: Ray,
        node_stack: &mut Vec<(&'a BVHNode, Float)>,
        bounce: u32,
    ) -> Color {
        let mut c = Color::black();
        if let Some(mut isect) = self.scene.intersect(&mut ray, node_stack) {
            // Flip the normal if its pointing to the opposite side from the hit
            if isect.n.dot(ray.dir) > 0.0 {
                isect.n *= -1.0;
            }
            if bounce == 0 {
                c += isect.le(-ray.dir);
            }
            let (le, light_p, light_pdf) = self.sample_light(&isect);
            let mut shadow_ray = isect.shadow_ray(light_p);
            if self.scene.intersect(&mut shadow_ray, node_stack).is_none() {
                let cos_t = isect.n.dot(shadow_ray.dir).max(0.0);
                c += le * isect.brdf() * cos_t / light_pdf;
            }
            let rr = rand::random::<Float>();
            if rr < RR_PROB {
                let (brdf, new_dir, mut pdf) = isect.sample_brdf();
                pdf *= RR_PROB;
                let new_ray = isect.ray(new_dir);
                c += isect.n.dot(new_dir) * brdf * self.trace_ray(new_ray, node_stack, bounce + 1)
                    / pdf;
            }
        }
        c
    }

    pub fn sample_light(&self, isect: &Interaction) -> (Color, Point3<Float>, Float) {
        let (light, pdf) = self.scene.sample_light().unwrap_or((&self.camera, 1.0));
        let (li, p, lpdf) = light.sample_li(isect);
        (li, p, pdf * lpdf)
    }
}
