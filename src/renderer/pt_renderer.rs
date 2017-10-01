use cgmath::prelude::*;
use cgmath::Vector4;

use glium;
use glium::{VertexBuffer, IndexBuffer, Surface, DrawParameters};
use glium::backend::Facade;
use glium::texture::{RawImage2d, Texture2d};

use camera::Camera;
use renderer::{Vertex, Ray, Hit};
use scene::Scene;

pub struct PTRenderer {
    shader: glium::Program,
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u32>,
}

impl PTRenderer {
    pub fn new<F: Facade>(facade: &F) -> PTRenderer {
        let vertices = vec!(
            Vertex { position: [-1.0, -1.0, 0.0],
                     normal: [0.0, 0.0, 0.0],
                     tex_coords: [0.0, 0.0] },
            Vertex { position: [1.0, -1.0, 0.0],
                     normal: [0.0, 0.0, 0.0],
                     tex_coords: [1.0, 0.0] },
            Vertex { position: [1.0, 1.0, 0.0],
                     normal: [0.0, 0.0, 0.0],
                     tex_coords: [1.0, 1.0] },
            Vertex { position: [-1.0, 1.0, 0.0],
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
        let program = glium::Program::from_source(facade, vertex_shader_src, fragment_shader_src, None)
            .expect("Failed to create program!");
        PTRenderer { shader: program, vertex_buffer: vertex_buffer, index_buffer: index_buffer }
    }

    #[cfg_attr(feature="clippy", allow(needless_range_loop))]
    pub fn render<S: Surface, F: Facade>(&self, scene: &Scene, target: &mut S, facade: &F,
                                         width: usize, height: usize, camera: &Camera) {
        let clip_to_world = (camera.get_camera_to_clip(width as u32, height as u32)
            * camera.get_world_to_camera()).invert().expect("Non invertible world to clip");
        let mut image = vec![0.0; 3 * width * height];
        for y in 0..height {
            for x in 0..width {
                let clip_x = 2.0 * x as f32 / width as f32 - 1.0;
                let clip_y = 2.0 * y as f32 / height as f32 - 1.0;
                let clip_p = Vector4::new(clip_x, clip_y, 1.0, 1.0);
                let world_p = clip_to_world * clip_p;
                let dir = ((world_p / world_p.w).truncate() - camera.pos.to_vec()).normalize();
                let ray = Ray::new(camera.pos, dir, 100.0);
                let mut current_hit: Option<Hit> = None;

                for tri in &scene.triangles {
                    if let Some(hit) = tri.intersect(&ray) {
                        if let Some(best) = current_hit.take() {
                            if hit.t < best.t {
                                current_hit = Some(hit);
                            } else {
                                current_hit = Some(best);
                            }
                        } else {
                            current_hit = Some(hit);
                        }
                    }
                }

                if let Some(hit) = current_hit {
                    // This should account for sRBG
                    let mut c = hit.tri.get_diffuse(&scene.materials, hit.u, hit.v);
                    c *= dir.dot(hit.tri.get_normal(hit.u, hit.v)).abs();
                    image[3 * (y * width + x)]     = c.x;
                    image[3 * (y * width + x) + 1] = c.y;
                    image[3 * (y * width + x) + 2] = c.z;
                } else {
                    image[3 * (y * width + x)]     = 0.0;
                    image[3 * (y * width + x) + 1] = 0.0;
                    image[3 * (y * width + x) + 2] = 0.0;
                }

            }
        }
        let mut raw_image = RawImage2d::from_raw_rgb(image, (width as u32, height as u32));
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
}
