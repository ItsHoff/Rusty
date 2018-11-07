use std::path::Path;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    mpsc::{self, Receiver, Sender},
    Arc,
};
use std::thread::{self, JoinHandle};

use glium;
use glium::backend::Facade;
use glium::texture::{MipmapsOption, RawImage2d, Texture2d, UncompressedFloatFormat};
use glium::{uniform, DrawParameters, IndexBuffer, Rect, Surface, VertexBuffer};

use crate::camera::Camera;
use crate::scene::Scene;
use crate::stats;
use crate::vertex::RawVertex;

mod config;
mod render_worker;
mod traced_image;

pub use self::config::*;
use self::render_worker::RenderWorker;
use self::traced_image::TracedImage;

pub struct RenderCoordinator {
    width: u32,
    height: u32,
    max_blocks: Option<usize>,
    current_block: AtomicUsize,
    block_width: u32,
    block_height: u32,
    x_blocks: usize,
    y_blocks: usize,
}

impl RenderCoordinator {
    fn new(width: u32, height: u32, config: &RenderConfig) -> RenderCoordinator {
        let block_height = 50;
        let block_width = 50;
        let x_blocks = (f64::from(width) / f64::from(block_width)).ceil() as usize;
        let y_blocks = (f64::from(height) / f64::from(block_height)).ceil() as usize;
        let blocks_per_iter = x_blocks * y_blocks;
        let max_blocks = config.max_iterations.map(|iters| iters * blocks_per_iter);
        RenderCoordinator {
            width,
            height,
            max_blocks,
            current_block: AtomicUsize::new(0),
            block_width,
            block_height,
            x_blocks,
            y_blocks,
        }
    }

    fn next_block(&self) -> Option<Rect> {
        let block_i = self.current_block.fetch_add(1, Ordering::Relaxed);
        if let Some(max) = self.max_blocks {
            if block_i >= max {
                return None;
            }
        };
        let iter_i = block_i % (self.x_blocks * self.y_blocks);
        let x_i = (iter_i % self.x_blocks) as u32;
        let y_i = (iter_i / self.x_blocks) as u32;
        let start_x = self.block_width * x_i;
        let end_x = (self.block_width * (x_i + 1)).min(self.width);
        let start_y = self.block_height * y_i;
        let end_y = (self.block_height * (y_i + 1)).min(self.height);
        Some(Rect {
            left: start_x,
            bottom: start_y,
            width: end_x - start_x,
            height: end_y - start_y,
        })
    }
}

struct PTVisualizer {
    shader: glium::Program,
    vertex_buffer: VertexBuffer<RawVertex>,
    index_buffer: IndexBuffer<u32>,
    texture: Texture2d,
    tone_map: bool,
}

fn create_texture<F: Facade>(facade: &F, texture_source: RawImage2d<f32>) -> Texture2d {
    Texture2d::with_format(
        facade,
        texture_source,
        UncompressedFloatFormat::F32F32F32,
        MipmapsOption::NoMipmap,
    )
    .unwrap()
}

impl PTVisualizer {
    fn new<F: Facade>(
        facade: &F,
        texture_source: RawImage2d<f32>,
        config: &RenderConfig,
    ) -> PTVisualizer {
        let vertices = vec![
            RawVertex {
                pos: [-1.0, -1.0, 0.0],
                normal: [0.0, 0.0, 0.0],
                tex_coords: [0.0, 0.0],
            },
            RawVertex {
                pos: [1.0, -1.0, 0.0],
                normal: [0.0, 0.0, 0.0],
                tex_coords: [1.0, 0.0],
            },
            RawVertex {
                pos: [1.0, 1.0, 0.0],
                normal: [0.0, 0.0, 0.0],
                tex_coords: [1.0, 1.0],
            },
            RawVertex {
                pos: [-1.0, 1.0, 0.0],
                normal: [0.0, 0.0, 0.0],
                tex_coords: [0.0, 1.0],
            },
        ];
        let vertex_buffer =
            VertexBuffer::new(facade, &vertices).expect("Failed to create vertex buffer!");
        let indices = vec![0, 1, 2, 0, 2, 3];
        let index_buffer =
            IndexBuffer::new(facade, glium::index::PrimitiveType::TrianglesList, &indices)
                .expect("Failed to create index buffer!");

        // Image shader
        let vertex_shader_src = include_str!("shaders/image.vert");
        let fragment_shader_src = include_str!("shaders/image.frag");
        let shader =
            glium::Program::from_source(facade, vertex_shader_src, fragment_shader_src, None)
                .expect("Failed to create program!");

        let texture = create_texture(facade, texture_source);

        PTVisualizer {
            shader,
            vertex_buffer,
            index_buffer,
            texture,
            tone_map: config.tone_map,
        }
    }

    fn render<S: Surface>(&self, target: &mut S) {
        let uniforms = uniform! {
            image: &self.texture,
            tone_map: self.tone_map,
        };
        let draw_parameters = DrawParameters {
            ..Default::default()
        };
        target
            .draw(
                &self.vertex_buffer,
                &self.index_buffer,
                &self.shader,
                &uniforms,
                &draw_parameters,
            )
            .unwrap();
    }

    fn new_image<F: Facade>(
        &mut self,
        facade: &F,
        texture_source: RawImage2d<f32>,
        config: &RenderConfig,
    ) {
        self.texture = create_texture(facade, texture_source);
        self.tone_map = config.tone_map;
    }

    fn update_texture(&mut self, rect: Rect, texture_block: RawImage2d<f32>) {
        self.texture.write(rect, texture_block);
    }
}

pub struct PTRenderer {
    visualizer: Option<PTVisualizer>,
    image: TracedImage,
    coordinator: Arc<RenderCoordinator>,
    result_rx: Option<Receiver<(Rect, Vec<f32>)>>,
    message_txs: Vec<Sender<()>>,
    thread_handles: Vec<JoinHandle<()>>,
}

impl PTRenderer {
    pub fn new() -> PTRenderer {
        let image = TracedImage::empty(0, 0);
        PTRenderer {
            visualizer: None,
            image,
            coordinator: Arc::new(RenderCoordinator::new(0, 0, &RenderConfig::default())),
            result_rx: None,
            message_txs: Vec::new(),
            thread_handles: Vec::new(),
        }
    }

    fn start_render(&mut self, scene: &Arc<Scene>, camera: &Camera, config: &RenderConfig) {
        stats::start_render();
        let width = camera.width;
        let height = camera.height;
        self.image = TracedImage::empty(width, height);
        self.coordinator = Arc::new(RenderCoordinator::new(width, height, config));

        let (result_tx, result_rx) = mpsc::channel();
        self.result_rx = Some(result_rx);
        for _ in 0..num_cpus::get_physical().max(1) {
            let result_tx = result_tx.clone();
            let (message_tx, message_rx) = mpsc::channel();
            self.message_txs.push(message_tx);
            let coordinator = self.coordinator.clone();
            let camera = camera.clone();
            let config = config.clone();
            let scene = scene.clone();
            let handle = thread::spawn(move || {
                let worker =
                    RenderWorker::new(scene, camera, config, coordinator, message_rx, result_tx);
                worker.run();
            });
            self.thread_handles.push(handle);
        }
    }

    pub fn online_render<F: Facade>(
        &mut self,
        facade: &F,
        scene: &Arc<Scene>,
        camera: &Camera,
        config: &RenderConfig,
    ) {
        // TODO: Add support for max_iterations
        self.start_render(scene, camera, config);
        if let Some(visualizer) = &mut self.visualizer {
            visualizer.new_image(facade, self.image.get_texture_source(), config);
        } else {
            self.visualizer = Some(PTVisualizer::new(
                facade,
                self.image.get_texture_source(),
                config,
            ));
        }
    }

    pub fn offline_render(&mut self, scene: &Arc<Scene>, camera: &Camera, config: &RenderConfig) {
        self.start_render(scene, camera, config);

        // Wait for all the threads to finish
        for handle in self.thread_handles.drain(..) {
            handle.join().unwrap();
        }

        // TODO: update image during render
        for (rect, block) in self.result_rx.as_ref().unwrap().try_iter() {
            self.image.update_block(rect, &block);
        }
        stats::stop_render();
    }

    pub fn render<S: Surface>(&mut self, target: &mut S) {
        if let Some(ref rx) = self.result_rx {
            let visualizer = self.visualizer.as_mut().unwrap();
            for (rect, block) in rx.try_iter().take(10) {
                let (rect, texture_block) = self.image.update_block(rect, &block);
                visualizer.update_texture(rect, texture_block);
            }
            visualizer.render(target);
        }
    }

    pub fn stop_threads(&mut self) {
        for sender in &self.message_txs {
            sender.send(()).unwrap();
        }
        for handle in self.thread_handles.drain(..) {
            handle.join().unwrap();
        }
        // Drop channels only after join to make sure
        // that stop messages are properly received
        self.message_txs.clear();
        stats::stop_render();
    }

    pub fn save_image(&self, path: &Path) {
        self.image.save_image(path);
    }
}
