use std::sync::{
    mpsc::{Receiver, Sender, TryRecvError},
    Arc,
};

use cgmath::prelude::*;
use cgmath::{Point3, Vector4};

use glium::Rect;

use crate::bvh::BVHNode;
use crate::camera::PTCamera;
use crate::color::Color;
use crate::config::*;
use crate::float::*;
use crate::intersect::Ray;
use crate::scene::Scene;

use super::tracers;
use super::RenderCoordinator;

pub struct RenderWorker {
    scene: Arc<Scene>,
    camera: PTCamera,
    config: RenderConfig,
    coordinator: Arc<RenderCoordinator>,
    message_rx: Receiver<()>,
    result_tx: Sender<(Rect, Vec<f32>)>,
}

impl RenderWorker {
    pub fn new(
        scene: Arc<Scene>,
        camera: PTCamera,
        config: RenderConfig,
        coordinator: Arc<RenderCoordinator>,
        message_rx: Receiver<()>,
        result_tx: Sender<(Rect, Vec<f32>)>,
    ) -> RenderWorker {
        RenderWorker {
            scene,
            camera,
            config,
            coordinator,
            message_rx,
            result_tx,
        }
    }

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
                        let mut c = Color::black();
                        for j in 0..self.config.samples_per_dir {
                            for i in 0..self.config.samples_per_dir {
                                let dx = (i.to_float() + rand::random::<Float>())
                                    / self.config.samples_per_dir.to_float();
                                let dy = (j.to_float() + rand::random::<Float>())
                                    / self.config.samples_per_dir.to_float();
                                let clip_x = 2.0 * ((rect.left + w).to_float() + dx)
                                    / width.to_float()
                                    - 1.0;
                                let clip_y = 2.0 * ((rect.bottom + h).to_float() + dy)
                                    / height.to_float()
                                    - 1.0;
                                let clip_p = Vector4::new(clip_x, clip_y, 1.0, 1.0);
                                let world_p = Point3::from_homogeneous(clip_to_world * clip_p);
                                let ray = Ray::from_point(self.camera.pos, world_p);
                                c += match &self.config.render_mode {
                                    RenderMode::Debug(mode) => tracers::debug_trace(
                                        ray,
                                        mode,
                                        &self.scene,
                                        &self.config,
                                        &mut node_stack,
                                    ),
                                    RenderMode::PathTracing => tracers::path_trace(
                                        ray,
                                        &self.scene,
                                        // TODO: What is the cleanest way to use the flash?
                                        self.camera.flash(),
                                        &self.config,
                                        &mut node_stack,
                                    ),
                                    RenderMode::BDPT => tracers::bdpt(
                                        ray,
                                        &self.scene,
                                        &self.camera,
                                        &self.config,
                                        &mut node_stack,
                                    ),
                                }
                            }
                        }
                        c /= self.config.samples_per_dir.pow(2).to_float();
                        let pixel_i = 3 * (h * rect.width + w) as usize;
                        let data: [f32; 3] = c.into();
                        block[pixel_i..pixel_i + 3].copy_from_slice(&data);
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
}
