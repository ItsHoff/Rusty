use cgmath::conv::*;

use glium;
use glium::{DrawParameters, Surface};
use glium::backend::Facade;

use camera::Camera;
use scene::GPUScene;

pub struct GLRenderer {
    shader: glium::Program,
}

impl GLRenderer {
    pub fn new<F: Facade>(facade: &F) -> GLRenderer {
        let vertex_shader_src = include_str!("shaders/preview.vert");
        let fragment_shader_src = include_str!("shaders/preview.frag");
        let shader = glium::Program::from_source(facade, vertex_shader_src, fragment_shader_src, None)
            .expect("Failed to create program!");
        GLRenderer { shader }
    }

    pub fn render<S: Surface>(&self, target: &mut S, scene: &GPUScene, camera: &Camera) {
        let world_to_clip = camera.get_world_to_clip();
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
                u_has_diffuse: material.has_diffuse,
                u_has_emissive: material.has_emissive,
                tex_diffuse: &material.diffuse_texture
            };
            target.draw(&scene.vertex_buffer, &mesh.index_buffer,
                        &self.shader, &uniforms, &draw_parameters).unwrap();
        }
    }
}
