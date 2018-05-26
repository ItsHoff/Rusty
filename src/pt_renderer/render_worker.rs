use std::sync::{Arc, Mutex, mpsc::{Sender, Receiver, TryRecvError}};

use cgmath::{Vector4, prelude::*};

use glium::Rect;

use camera::Camera;
use pt_renderer::{Intersect, Ray, RenderCoordinator};
use scene::Scene;
use triangle::Hit;

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
                        let clip_x = 2.0 * (rect.left + w) as f32 / width as f32 - 1.0;
                        let clip_y = 2.0 * (rect.bottom + h) as f32 / height as f32 - 1.0;
                        let clip_p = Vector4::new(clip_x, clip_y, 1.0, 1.0);
                        let world_p = clip_to_world * clip_p;
                        let dir = ((world_p / world_p.w).truncate() - self.camera.pos.to_vec())
                            .normalize();
                        let ray = Ray::new(self.camera.pos, dir, 100.0);

                        let pixel_i = 3 * (h * rect.width + w) as usize;
                        if let Some(hit) = self.find_hit(ray) {
                            let light_sample = self.scene.sample_light();
                            let hit_to_light = light_sample - hit.pos();
                            let light_ray = Ray::new(hit.pos(), hit_to_light.normalize(),
                                                     hit_to_light.magnitude());
                            let e = if let Some(hit) = self.find_hit(light_ray) {
                                if 1e-5 < hit.t && hit.t < 1.0 - 1e-5 {
                                    0.0
                                } else {
                                    1.0
                                }

                            } else {
                                1.0
                            };
                            // TODO: This should account for sRBG
                            let mut c = hit.tri.diffuse(&self.scene.materials, hit.u, hit.v);
                            c *= dir.dot(hit.tri.normal(hit.u, hit.v)).abs();
                            block[pixel_i]     = e * c.x;
                            block[pixel_i + 1] = e * c.y;
                            block[pixel_i + 2] = e * c.z;
                        } else {
                            block[pixel_i]     = 0.1;
                            block[pixel_i + 1] = 0.1;
                            block[pixel_i + 2] = 0.1;
                        }
                    }
                }
                self.result_tx.send((rect, block)).expect("Receiver closed!");
            } else {
                return;
            }
        }
    }

    fn find_hit(&self, ray: Ray) -> Option<Hit> {
        let bvh = &self.scene.bvh;
        let mut node_stack = Vec::new();
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
