use std::path::Path;

use cgmath::Point2;

use glium::backend::Facade;
use glium::framebuffer::SimpleFrameBuffer;
use glium::texture::{
    ClientFormat, MipmapsOption, RawImage2d, SrgbTexture2d, Texture2d, UncompressedFloatFormat,
    UncompressedUintFormat, UnsignedTexture2d,
};
use glium::{uniform, DrawParameters, IndexBuffer, Rect, Surface, VertexBuffer};

use crate::pt_renderer::RenderConfig;
use crate::vertex::RawVertex;

pub struct TracedImage {
    pixels: Vec<f32>,
    n_samples: Vec<u32>,
    width: u32,
    height: u32,
    visualizer: Visualizer,
}

impl TracedImage {
    pub fn new<F: Facade>(facade: &F, config: &RenderConfig) -> Self {
        let width = config.width;
        let height = config.height;
        let pixels = vec![0.0; (3 * width * height) as usize];
        let n_samples = vec![0; (width * height) as usize];
        let visualizer = Visualizer::new(facade, config);
        Self {
            pixels,
            n_samples,
            width,
            height,
            visualizer,
        }
    }

    pub fn add_sample(&mut self, rect: Rect, sample: &[f32]) {
        for h in 0..rect.height {
            for w in 0..rect.width {
                let i_image = ((h + rect.bottom) * self.width + w + rect.left) as usize;
                let i_block = (h * rect.width + w) as usize;
                self.n_samples[i_image] += 1;
                for c in 0..3 {
                    self.pixels[3 * i_image + c] += sample[3 * i_block + c];
                }
            }
        }
    }

    #[allow(clippy::needless_range_loop)]
    pub fn add_splat(&mut self, pixel: Point2<u32>, sample: [f32; 3]) {
        let i_image = (pixel.y * self.width + pixel.x) as usize;
        for c in 0..3 {
            self.pixels[3 * i_image + c] += sample[c];
        }
    }

    pub fn render<F: Facade, S: Surface>(&self, facade: &F, target: &mut S) {
        self.visualizer.render(
            facade,
            target,
            &self.pixels,
            &self.n_samples,
            self.width,
            self.height,
        );
    }

    pub fn save<F: Facade>(&self, facade: &F, path: &Path) {
        let texture = SrgbTexture2d::empty(facade, self.width, self.height).unwrap();
        let mut target = SimpleFrameBuffer::new(facade, &texture).unwrap();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
        self.render(facade, &mut target);
        let pb = texture.read_to_pixel_buffer();
        let raw_image: RawImage2d<u8> = pb.read_as_texture_2d().unwrap();
        let image =
            image::RgbaImage::from_vec(self.width, self.height, raw_image.data.to_vec()).unwrap();
        let image = image::imageops::flip_vertical(&image);
        image.save(path).unwrap();
    }
}

struct Visualizer {
    shader: glium::Program,
    vertex_buffer: VertexBuffer<RawVertex>,
    index_buffer: IndexBuffer<u32>,
    tone_map: bool,
}

impl Visualizer {
    fn new<F: Facade>(facade: &F, config: &RenderConfig) -> Self {
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
        let vertex_shader_src = include_str!("../shaders/image.vert");
        let fragment_shader_src = include_str!("../shaders/image.frag");
        let shader =
            glium::Program::from_source(facade, vertex_shader_src, fragment_shader_src, None)
                .expect("Failed to create program!");

        Self {
            shader,
            vertex_buffer,
            index_buffer,
            tone_map: config.tone_map,
        }
    }

    fn render<F: Facade, S: Surface>(
        &self,
        facade: &F,
        target: &mut S,
        data: &[f32],
        n_samples: &[u32],
        width: u32,
        height: u32,
    ) {
        let data_raw = RawImage2d {
            data: std::borrow::Cow::from(data),
            width,
            height,
            format: ClientFormat::F32F32F32,
        };
        let data_texture = Texture2d::with_format(
            facade,
            data_raw,
            UncompressedFloatFormat::F32F32F32,
            MipmapsOption::NoMipmap,
        )
        .unwrap();

        let n_raw = RawImage2d {
            data: std::borrow::Cow::from(n_samples),
            width,
            height,
            format: ClientFormat::U32,
        };
        let n_texture = UnsignedTexture2d::with_format(
            facade,
            n_raw,
            UncompressedUintFormat::U32,
            MipmapsOption::NoMipmap,
        )
        .unwrap();

        let uniforms = uniform! {
            image: &data_texture,
            n: &n_texture,
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
}
