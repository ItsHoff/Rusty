use glium::backend::Facade;
use glium::{uniform, DrawParameters, Surface};

use crate::camera::Camera;
use crate::float::IntoArray;
use crate::scene::GpuScene;

pub struct GlRenderer {
    shader: glium::Program,
}

impl GlRenderer {
    pub fn new<F: Facade>(facade: &F) -> GlRenderer {
        let vertex_shader_src = include_str!("shaders/preview.vert");
        let fragment_shader_src = include_str!("shaders/preview.frag");
        let shader =
            glium::Program::from_source(facade, vertex_shader_src, fragment_shader_src, None)
                .expect("Failed to create program!");
        GlRenderer { shader }
    }

    pub fn render<S: Surface>(&self, target: &mut S, scene: &GpuScene, camera: &Camera) {
        let draw_parameters = DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            ..Default::default()
        };

        for mesh in &scene.meshes {
            let material = &scene.materials[mesh.material_i];
            let uniforms = uniform! {
                world_to_clip: camera.world_to_clip().into_array(),
                u_light: [-1.0, 0.4, 0.9f32],
                u_is_emissive: material.is_emissive,
                tex: &material.texture
            };
            target
                .draw(
                    &scene.vertex_buffer,
                    &mesh.index_buffer,
                    &self.shader,
                    &uniforms,
                    &draw_parameters,
                )
                .unwrap();
        }
    }
}
