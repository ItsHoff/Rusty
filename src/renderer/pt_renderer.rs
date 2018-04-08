use cgmath::prelude::*;
use cgmath::Vector4;

use glium;
use glium::{VertexBuffer, IndexBuffer, Surface, DrawParameters};
use glium::backend::Facade;
use glium::texture::{RawImage2d, Texture2d};

use camera::Camera;
use renderer::{Vertex, Ray, Hit, Intersect};
use scene::Scene;
use scene::bvh::BVHNode;

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

    fn next_block(&mut self) -> (usize, usize, usize, usize) {
        let block_height = 20;
        let block_width = 20;
        // TODO: check max_iterations
        if self.end_y >= self.height && self.end_x >= self.width {
            self.iteration += 1;
            self.end_y = 0;
            self.end_x = 0;
        } else if self.end_x >= self.width {
            self.start_y = self.end_y;
            self.end_y = (self.start_y + block_height).min(self.height);
            self.end_x = 0;
        }
        let start_x = self.end_x;
        self.end_x = (start_x + block_width).min(self.width);
        (start_x as usize, self.end_x as usize,
         self.start_y as usize, self.end_y as usize)
    }
}

pub struct PTRenderer {
    shader: glium::Program,
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u32>,
    image: Vec<f32>,
    node_stack: Vec<(*const BVHNode, f32)>,
    coordinator: RenderCoordinator,
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

        // Image shader
        let vertex_shader_src = include_str!("../image.vert");
        let fragment_shader_src = include_str!("../image.frag");
        let shader = glium::Program::from_source(facade, vertex_shader_src, fragment_shader_src, None)
            .expect("Failed to create program!");
        PTRenderer { shader, vertex_buffer, index_buffer,
                     image: Vec::new(),
                     node_stack: Vec::new(),
                     coordinator: RenderCoordinator::new(0, 0, None),
        }
    }

    pub fn start_render(&mut self, width: u32, height: u32) {
        self.image = vec![0.0f32; (3 * width * height) as usize];
        self.coordinator = RenderCoordinator::new(width, height, None);
    }

    #[cfg_attr(feature="clippy", allow(needless_range_loop))]
    pub fn render<S: Surface, F: Facade>(&mut self, scene: &Scene, target: &mut S, facade: &F,
                                         camera: &Camera) {
        let width = self.coordinator.width;
        let height = self.coordinator.height;
        let clip_to_world = (camera.get_camera_to_clip(width, height)
                             * camera.get_world_to_camera()).invert()
            .expect("Non invertible world to clip");
        let (x_start, x_end, y_start, y_end) = self.coordinator.next_block();
        for y in y_start..y_end {
            for x in x_start..x_end {
                let clip_x = 2.0 * x as f32 / width as f32 - 1.0;
                let clip_y = 2.0 * y as f32 / height as f32 - 1.0;
                let clip_p = Vector4::new(clip_x, clip_y, 1.0, 1.0);
                let world_p = clip_to_world * clip_p;
                let dir = ((world_p / world_p.w).truncate() - camera.pos.to_vec()).normalize();
                let ray = Ray::new(camera.pos, dir, 100.0);

                let pixel_i = 3 * (y * width as usize + x);
                if let Some(hit) = self.find_hit(scene, ray) {
                    // TODO: This should account for sRBG
                    let mut c = hit.tri.diffuse(&scene.materials, hit.u, hit.v);
                    c *= dir.dot(hit.tri.normal(hit.u, hit.v)).abs();
                    self.image[pixel_i]     = c.x;
                    self.image[pixel_i + 1] = c.y;
                    self.image[pixel_i + 2] = c.z;
                } else {
                    self.image[pixel_i]     = 0.1;
                    self.image[pixel_i + 1] = 0.1;
                    self.image[pixel_i + 2] = 0.1;
                }

            }
        }
        let mut raw_image = RawImage2d::from_raw_rgb(self.image.clone(),
                                                     (width as u32, height as u32));
        raw_image.format = glium::texture::ClientFormat::F32F32F32;
        let texture = Texture2d::new(facade, raw_image).expect("Failed to upload traced image!");
        let uniforms = uniform! {
            image: &texture,
        };
        let draw_parameters = DrawParameters {
            ..Default::default()
        };
        target.draw(&self.vertex_buffer, &self.index_buffer, &self.shader,
                    &uniforms, &draw_parameters).unwrap();
    }

    fn find_hit<'a>(&mut self, scene: &'a Scene, ray: Ray) -> Option<Hit<'a>> {
        let bvh = &scene.bvh;
        self.node_stack.push((bvh.root(), 0.0f32));
        let mut closest_hit: Option<Hit> = None;
        while let Some((node_p, t)) = self.node_stack.pop() {
            let node = unsafe { &*node_p };
            // We've already found closer hit
            if closest_hit.as_ref().map_or(false, |hit| hit.t <= t) { continue }
            if node.is_leaf() {
                for tri in &scene.triangles[node.start_i..node.end_i] {
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
                    self.node_stack.push((left, t_left));
                }
                if let Some(t_right) = right.intersect(&ray) {
                    self.node_stack.push((right, t_right));
                }
            }
        }
        closest_hit
    }
}
