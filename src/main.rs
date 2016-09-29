#[macro_use]
extern crate glium;
extern crate cgmath;

use std::io::Read;
use std::fs::File;
use std::path::{Path, PathBuf};

use glium::{DisplayBuild, Surface, VertexBuffer, IndexBuffer, };
use glium::index::PrimitiveType;
use cgmath::conv::*;

mod common;

fn get_project_root() -> PathBuf {
    let exe_dir = std::env::current_exe().unwrap();
    let mut parent_dir = exe_dir.parent().unwrap();
    while !parent_dir.ends_with("rusty") {
        parent_dir = parent_dir.parent().unwrap();
    }
    parent_dir.to_path_buf()
}

fn read_shader_from_file(shader_path: &Path) -> String {
    let mut file = File::open(shader_path).unwrap();
    let mut shader_src = String::new();
    file.read_to_string(&mut shader_src).unwrap();
    shader_src
}

fn main() {
    let display = glium::glutin::WindowBuilder::new().with_depth_buffer(24).build_glium().unwrap();

    let root_path = get_project_root();
    //let scene = common::load_scene(&root_path.join("scenes/cornell/cornell.obj"));
    //let scene = common::load_scene(&root_path.join("scenes/cornell-box/CornellBox-Original.obj"));
    let scene = common::load_scene(&root_path.join("scenes/nanosuit/nanosuit.obj"));
    let vertex_buffer = VertexBuffer::new(&display, &scene.vertices).expect("Failed to unwrap vertex buffer.");
    let index_buffer = IndexBuffer::new(&display, PrimitiveType::TrianglesList,
                                        &scene.indices).expect("Failed to unwrap index buffer.");

    let src_path = root_path.join("src");
    let vertex_shader_src = read_shader_from_file(&src_path.join("vertex.glsl"));
    let fragment_shader_src = read_shader_from_file(&src_path.join("fragment.glsl"));
    let program = glium::Program::from_source(&display, &vertex_shader_src, &fragment_shader_src, None).unwrap();

    let params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            .. Default::default()
        },
        .. Default::default()
    };

    let mut camera_pos = cgmath::Point3::new(-2.0, 0.0, 0.0);

    loop {

        let mut target = display.draw();

        let (width, height) = target.get_dimensions();
        let perspective = cgmath::perspective(cgmath::Rad(std::f32::consts::PI / 3.0),
                                              width as f32 / height as f32, 0.01, 100.0f32);
        let camera = cgmath::Matrix4::look_at(camera_pos,
                                              cgmath::Point3::new(1.0, 0.0, 0.0),
                                              cgmath::vec3(0.0, 1.0, 0.0f32));
        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ],
            camera: array4x4(camera),
            perspective: array4x4(perspective),
            u_light: [-1.0, 0.4, 0.9f32]
        };

        target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
        target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &params).unwrap();
        target.finish().unwrap();

        for event in display.poll_events() {
            match event {
                glium::glutin::Event::Closed => return,
                glium::glutin::Event::KeyboardInput(_, _, _) => {
                    camera_pos.x = camera_pos.x + 0.01;
                }
                _ => ()
            }
        }
    }
}
