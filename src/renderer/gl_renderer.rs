use cgmath::Matrix4;
use cgmath::conv::*;

use glium;
use glium::{DrawParameters, Surface};
use glium::backend::Facade;

use scene::Scene;

pub struct GLRenderer {
    shader: glium::Program,
}

impl GLRenderer {
    pub fn new<F: Facade>(facade: &F) -> GLRenderer {
        let vertex_shader_src = include_str!("../preview.vert");
        let fragment_shader_src = include_str!("../preview.frag");
        let program = glium::Program::from_source(facade, vertex_shader_src, fragment_shader_src, None)
            .expect("Failed to create program!");
        GLRenderer { shader: program }
    }

    pub fn render<S: Surface>(&self, scene: &Scene, target: &mut S, world_to_clip: Matrix4<f32>) {
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
                local_to_world: array4x4(mesh.local_to_world),
                world_to_clip: array4x4(world_to_clip),
                u_light: [-1.0, 0.4, 0.9f32],
                u_color: material.diffuse,
                u_has_diffuse: material.diffuse_image.is_some(),
                tex_diffuse: material.diffuse_texture.as_ref().expect("Use of unloaded texture!")
            };
            target.draw(scene.vertex_buffer.as_ref().expect("No vertex buffer"),
                        mesh.index_buffer.as_ref().expect("No index buffer!"),
                        &self.shader, &uniforms, &draw_parameters).unwrap();
        }
    }
}
