extern crate num_cpus;

mod render_worker;

use std::sync::{Arc, Mutex, mpsc::{self, Sender, Receiver}};
use std::thread::{self, JoinHandle};

use cgmath::{Vector3, Point3};

use glium;
use glium::{VertexBuffer, IndexBuffer, Surface, DrawParameters, Rect};
use glium::backend::Facade;
use glium::texture::{RawImage2d, Texture2d};

use camera::Camera;
use scene::Scene;
use vertex::Vertex;

use self::render_worker::RenderWorker;

pub trait Intersect<'a, H> {
    fn intersect(&'a self, ray: &Ray) -> Option<H>;
}

#[derive(Clone, Copy)]
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

pub struct PTRenderer {
    shader: glium::Program,
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u32>,
    texture: Texture2d,
    coordinator: Arc<Mutex<RenderCoordinator>>,
    result_rx: Option<Receiver<(Rect, Vec<f32>)>>,
    message_txs: Vec<Sender<()>>,
    thread_handles: Vec<JoinHandle<()>>,
}

impl PTRenderer {
    pub fn new<F: Facade>(facade: &F) -> PTRenderer {
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
        let texture = Texture2d::empty(facade, 0, 0).expect("Failed to create trace texture!");

        // Image shader
        let vertex_shader_src = include_str!("../shaders/image.vert");
        let fragment_shader_src = include_str!("../shaders/image.frag");
        let shader = glium::Program::from_source(facade, vertex_shader_src, fragment_shader_src, None)
            .expect("Failed to create program!");
        PTRenderer { shader, vertex_buffer, index_buffer, texture,
                     coordinator: Arc::new(Mutex::new(RenderCoordinator::new(0, 0, None))),
                     result_rx: None,
                     message_txs: Vec::new(),
                     thread_handles: Vec::new(),
        }
    }

    pub fn start_render<F: Facade>(&mut self, facade: &F, scene: &Arc<Scene>, camera: &Camera) {
        let width = camera.width;
        let height = camera.height;
        let empty_image = RawImage2d::from_raw_rgb(vec![0.0; (3 * width * height) as usize], (width, height));
        self.texture = Texture2d::new(facade, empty_image).expect("Failed to upload traced image!");
        self.coordinator = Arc::new(Mutex::new(RenderCoordinator::new(width, height, None)));
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

    pub fn render<S: Surface>(&mut self, target: &mut S) {
        if let Some(ref rx) = self.result_rx {
            for (rect, block) in rx.try_iter().take(10) {
                let raw_block = RawImage2d::from_raw_rgb(block, (rect.width, rect.height));
                self.texture.write(rect, raw_block);
            }
            let uniforms = uniform! {
                image: &self.texture,
            };
            let draw_parameters = DrawParameters {
                ..Default::default()
            };
            target.draw(&self.vertex_buffer, &self.index_buffer, &self.shader,
                        &uniforms, &draw_parameters).unwrap();
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
}
