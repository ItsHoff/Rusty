extern crate num_cpus;

mod render_worker;
mod traced_image;

use std::path::Path;
use std::sync::{Arc, Mutex, mpsc::{self, Sender, Receiver}};
use std::thread::{self, JoinHandle};

use cgmath::{Vector3, Point3};

use glium;
use glium::{VertexBuffer, IndexBuffer, Surface, DrawParameters, Rect, uniform};
use glium::texture::{Texture2d, RawImage2d};
use glium::backend::Facade;

use crate::camera::Camera;
use crate::scene::Scene;
use crate::vertex::Vertex;

use self::render_worker::RenderWorker;
use self::traced_image::TracedImage;

pub trait Intersect<'a, H> {
    fn intersect(&'a self, ray: &Ray) -> Option<H>;
}

#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub orig: Point3<f32>,
    pub dir: Vector3<f32>,
    // For more efficient ray plane intersections
    pub reciprocal_dir: Vector3<f32>,
    pub length: f32,
}

impl Ray {
    fn new(orig: Point3<f32>, dir: Vector3<f32>, length: f32) -> Ray {
        let reciprocal_dir = 1.0 / dir;
        Ray { orig, dir, reciprocal_dir, length }
    }
}

#[allow(dead_code)]
pub struct RenderCoordinator {
    width: u32,
    height: u32,
    max_iterations: Option<u32>,
    end_x: u32,
    start_y: u32,
    end_y: u32,
    iteration: u32,
}

impl RenderCoordinator {
    fn new(width: u32, height: u32, max_iterations: Option<u32>) -> RenderCoordinator {
        RenderCoordinator {
            width, height, max_iterations,
            end_x: 0, start_y: 0, end_y: 0, iteration: 0
        }
    }

    fn next_block(&mut self) -> Option<Rect> {
        let block_height = 50;
        let block_width = 50;
        if let Some(max) = self.max_iterations {
            if self.iteration > max {
                return None;
            }
        }
        if self.end_y >= self.height && self.end_x >= self.width {
            self.iteration += 1;
            self.start_y = 0;
            self.end_y = block_width.min(self.height);
            self.end_x = 0;
        } else if self.end_x >= self.width {
            self.start_y = self.end_y;
            self.end_y = (self.start_y + block_height).min(self.height);
            self.end_x = 0;
        }
        let start_x = self.end_x;
        self.end_x = (start_x + block_width).min(self.width);
        Some (
            Rect {
                left: start_x,
                bottom: self.start_y,
                width: self.end_x - start_x,
                height: self.end_y - self.start_y,
            }
        )
    }
}

struct PTVisualizer {
    shader: glium::Program,
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u32>,
    texture: Texture2d,
}

impl PTVisualizer {
    fn new<F: Facade>(facade: &F, width: u32, height: u32) -> PTVisualizer {
        let vertices = vec!(
            Vertex { pos: [-1.0, -1.0, 0.0],
                     normal: [0.0, 0.0, 0.0],
                     tex_coords: [0.0, 0.0] },
            Vertex { pos: [1.0, -1.0, 0.0],
                     normal: [0.0, 0.0, 0.0],
                     tex_coords: [1.0, 0.0] },
            Vertex { pos: [1.0, 1.0, 0.0],
                     normal: [0.0, 0.0, 0.0],
                     tex_coords: [1.0, 1.0] },
            Vertex { pos: [-1.0, 1.0, 0.0],
                     normal: [0.0, 0.0, 0.0],
                     tex_coords: [0.0, 1.0] },
        );
        let vertex_buffer = VertexBuffer::new(facade, &vertices)
            .expect("Failed to create vertex buffer!");
        let indices = vec!(0, 1, 2, 0, 2, 3);
        let index_buffer = IndexBuffer::new(facade,
                                            glium::index::PrimitiveType::TrianglesList,
                                            &indices)
            .expect("Failed to create index buffer!");

        // Image shader
        let vertex_shader_src = include_str!("shaders/image.vert");
        let fragment_shader_src = include_str!("shaders/image.frag");
        let shader = glium::Program::from_source(facade, vertex_shader_src, fragment_shader_src, None)
            .expect("Failed to create program!");

        let texture = Texture2d::empty(facade, width, height).unwrap();

        PTVisualizer {
            shader, vertex_buffer, index_buffer, texture,
        }
    }

    fn render<S: Surface>(&self, target: &mut S) {
        let uniforms = uniform! {
            image: &self.texture,
        };
        let draw_parameters = DrawParameters {
            ..Default::default()
        };
        target.draw(&self.vertex_buffer, &self.index_buffer, &self.shader,
                    &uniforms, &draw_parameters).unwrap();
    }

    fn new_texture<F: Facade>(&mut self, facade: &F, texture_source: RawImage2d<f32>) {
        self.texture = Texture2d::new(facade, texture_source).unwrap();
    }

    fn update_texture(&mut self, rect: Rect, texture_block: RawImage2d<f32>) {
        self.texture.write(rect, texture_block);
    }
}

pub struct PTRenderer {
    visualizer: Option<PTVisualizer>,
    image: TracedImage,
    coordinator: Arc<Mutex<RenderCoordinator>>,
    result_rx: Option<Receiver<(Rect, Vec<f32>)>>,
    message_txs: Vec<Sender<()>>,
    thread_handles: Vec<JoinHandle<()>>,
}

impl PTRenderer {
    pub fn new() -> PTRenderer {
        let image = TracedImage::empty(0, 0);
        PTRenderer { visualizer: None,
                     image,
                     coordinator: Arc::new(Mutex::new(RenderCoordinator::new(0, 0, None))),
                     result_rx: None,
                     message_txs: Vec::new(),
                     thread_handles: Vec::new(),
        }
    }

    pub fn start_render<F: Facade>(&mut self, facade: &F, scene: &Arc<Scene>, camera: &Camera) {
        let width = camera.width;
        let height = camera.height;
        self.image = TracedImage::empty(width, height);
        self.coordinator = Arc::new(Mutex::new(RenderCoordinator::new(width, height, None)));
        if let Some(visualizer) = &mut self.visualizer {
            visualizer.new_texture(facade, self.image.get_texture_source());
        } else {
            self.visualizer = Some(PTVisualizer::new(facade, width, height));
        }

        let (result_tx, result_rx) = mpsc::channel();
        self.result_rx = Some(result_rx);
        for _ in 0..num_cpus::get_physical() {
            let result_tx = result_tx.clone();
            let (message_tx, message_rx) = mpsc::channel();
            self.message_txs.push(message_tx);
            let coordinator = self.coordinator.clone();
            let camera = camera.clone();
            let scene = scene.clone();
            let handle = thread::spawn(move|| {
                let worker = RenderWorker::new(scene.clone(), camera.clone(), coordinator.clone(),
                                               message_rx, result_tx);
                worker.run();
            });
            self.thread_handles.push(handle);
        }
    }

    // TODO: Refactor the common offline and online paths
    pub fn offline_render(&mut self, scene: &Arc<Scene>, camera: &Camera, iterations: u32) {
        let width = camera.width;
        let height = camera.height;
        self.image = TracedImage::empty(width, height);
        self.coordinator = Arc::new(Mutex::new(
            RenderCoordinator::new(width, height, Some(iterations))));

        let (result_tx, result_rx) = mpsc::channel();
        self.result_rx = Some(result_rx);
        for _ in 0..num_cpus::get_physical() {
            let result_tx = result_tx.clone();
            let (message_tx, message_rx) = mpsc::channel();
            self.message_txs.push(message_tx);
            let coordinator = self.coordinator.clone();
            let camera = camera.clone();
            let scene = scene.clone();
            let handle = thread::spawn(move|| {
                let worker = RenderWorker::new(scene.clone(), camera.clone(), coordinator.clone(),
                                               message_rx, result_tx);
                worker.run();
            });
            self.thread_handles.push(handle);
        }

        // Wait for all the threads to finish
        for handle in self.thread_handles.drain(..) {
            handle.join().unwrap();
        }

        for (rect, block) in self.result_rx.as_ref().unwrap().try_iter() {
            self.image.update_block(rect, &block);
        }

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
        // Drop channels after join so stop messages are properly reveived
        self.message_txs.clear();
    }

    pub fn save_image(&self, path: &Path) {
        self.image.save_image(path);
    }
}
