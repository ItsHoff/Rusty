use std::path::Path;
use std::sync::{
    mpsc::{self, Receiver, Sender},
    Arc,
};
use std::thread::{self, JoinHandle};

use glium::backend::Facade;
use glium::{Rect, Surface};

use crate::camera::{Camera, PTCamera};
use crate::config::RenderConfig;
use crate::scene::Scene;
use crate::stats;

mod coordinator;
mod render_worker;
mod traced_image;
mod tracers;

use self::coordinator::RenderCoordinator;
use self::render_worker::RenderWorker;
use self::traced_image::TracedImage;

pub struct PTRenderer {
    image: TracedImage,
    result_rx: Receiver<(Rect, Vec<f32>)>,
    message_txs: Vec<Sender<()>>,
    thread_handles: Vec<JoinHandle<()>>,
}

impl PTRenderer {
    pub fn start_render<F: Facade>(
        facade: &F,
        scene: &Arc<Scene>,
        camera: &Camera,
        config: &RenderConfig,
    ) -> Self {
        stats::start_render();
        let image = TracedImage::new(facade, config);
        let coordinator = Arc::new(RenderCoordinator::new(config));
        let mut message_txs = Vec::new();
        let mut thread_handles = Vec::new();

        let (result_tx, result_rx) = mpsc::channel();
        for _ in 0..num_cpus::get().min(config.max_threads) {
            let result_tx = result_tx.clone();
            let (message_tx, message_rx) = mpsc::channel();
            message_txs.push(message_tx);
            let coordinator = coordinator.clone();
            let camera = PTCamera::new(camera.clone());
            let config = config.clone();
            let scene = scene.clone();
            let handle = thread::spawn(move || {
                let worker =
                    RenderWorker::new(scene, camera, config, coordinator, message_rx, result_tx);
                worker.run();
            });
            thread_handles.push(handle);
        }
        Self {
            image,
            result_rx,
            message_txs,
            thread_handles,
        }
    }

    pub fn offline_render<F: Facade>(
        facade: &F,
        scene: &Arc<Scene>,
        camera: &Camera,
        config: &RenderConfig,
    ) -> Self {
        let mut renderer = Self::start_render(facade, scene, camera, config);
        // This loops until all senders have disconnected
        // ie. all workers have finished
        for (rect, block) in renderer.result_rx.iter() {
            renderer.image.add_sample(rect, &block);
        }
        renderer
    }

    pub fn update_image(&mut self) {
        let mut n = 0;
        let n_max = 1000;
        for (rect, sample) in self.result_rx.try_iter().take(n_max) {
            self.image.add_sample(rect, &sample);
            n += 1;
        }
        if n == n_max {
            println!("Hit maximum iterations in update!");
        }
    }

    pub fn render_image<S: Surface>(&mut self, target: &mut S) {
        self.image.render(target);
    }

    pub fn save_image<F: Facade>(&self, facade: &F, path: &Path) {
        self.image.save(facade, path);
    }
}

impl Drop for PTRenderer {
    fn drop(&mut self) {
        // Send stop message to workers
        for sender in &self.message_txs {
            sender.send(()).ok();
        }
        // And make sure that the workers have all stopped
        for handle in self.thread_handles.drain(..) {
            handle.join().unwrap();
        }
        stats::stop_render();
    }
}
