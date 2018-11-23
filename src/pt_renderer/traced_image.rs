use std::path::Path;

use glium::backend::Facade;
use glium::framebuffer::SimpleFrameBuffer;
use glium::texture::{MipmapsOption, RawImage2d, SrgbTexture2d, Texture2d, UncompressedFloatFormat};
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
        let raw_image = RawImage2d::from_raw_rgb(pixels.clone(), (width, height));
        let visualizer = Visualizer::new(facade, raw_image, config);
        Self {
            pixels,
            n_samples,
            width,
            height,
            visualizer,
        }
    }

    pub fn add_sample(&mut self, rect: Rect, sample: &[f32]) {
        let mut updated_pixels = vec![0.0f32; (3 * rect.width * rect.height) as usize];
        for h in 0..rect.height {
            for w in 0..rect.width {
                let i_image = ((h + rect.bottom) * self.width + w + rect.left) as usize;
                let i_block = (h * rect.width + w) as usize;
                let n = self.n_samples[i_image] + 1;
                self.n_samples[i_image] = n;
                for c in 0..3 {
                    let old_val = self.pixels[3 * i_image + c];
                    let new_val = old_val + 1.0 / (n as f32) * (sample[3 * i_block + c] - old_val);
                    updated_pixels[3 * i_block + c] = new_val;
                    self.pixels[3 * i_image + c] = new_val;
                }
            }
        }
        // Update the visualizer texture as well
        let data = RawImage2d::from_raw_rgb(updated_pixels, (rect.width, rect.height));
        self.visualizer.update_texture(rect, data);
    }

    pub fn render<S: Surface>(&self, target: &mut S) {
        self.visualizer.render(target);
    }

    pub fn save<F: Facade>(&self, facade: &F, path: &Path) {
        let texture = SrgbTexture2d::empty(
            facade,
            self.width,
            self.height,
        ).unwrap();
        let mut target = SimpleFrameBuffer::new(facade, &texture).unwrap();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
        self.visualizer.render(&mut target);
        let pb = texture.read_to_pixel_buffer();
        let raw_image: RawImage2d<u8> = pb.read_as_texture_2d().unwrap();
        let image = image::RgbaImage::from_vec(self.width, self.height,
                                        raw_image.data.to_vec()).unwrap();
        let image = image::imageops::flip_vertical(&image);
        image.save(path).unwrap();
    }
}

struct Visualizer {
    shader: glium::Program,
    vertex_buffer: VertexBuffer<RawVertex>,
    index_buffer: IndexBuffer<u32>,
    texture: Texture2d,
    tone_map: bool,
}

impl Visualizer {
    fn new<F: Facade>(facade: &F, raw_image: RawImage2d<f32>, config: &RenderConfig) -> Self {
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

        let texture = Texture2d::with_format(
            facade,
            raw_image,
            UncompressedFloatFormat::F32F32F32,
            MipmapsOption::NoMipmap,
        )
        .unwrap();

        Self {
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

    fn update_texture(&mut self, rect: Rect, data: RawImage2d<f32>) {
        self.texture.write(rect, data);
    }
}
