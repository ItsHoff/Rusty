use glium::glutin::{dpi::LogicalSize, VirtualKeyCode};

use crate::bvh::SplitMode;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum RenderMode {
    /// Standard path tracing
    PathTracing,
    /// Bidirectional path tracing
    BDPT,
    /// Debug
    Debug(DebugMode),
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum DebugMode {
    /// Normals
    Normals,
    /// Normals that point away from the camera
    ForwardNormals,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum LightMode {
    /// Use scene lights only (will still fall back to camera if there are none)
    Scene,
    /// Use camera flash as the light source
    Camera,
}

#[derive(Clone, Debug)]
pub struct RenderConfig {
    /// Width of the render target in pixels
    pub width: u32,
    /// Height of the render target in pixels
    pub height: u32,
    /// Maximum number of threads to use for rendering
    pub max_threads: usize,
    /// Should normal mapping be used
    pub normal_mapping: bool,
    /// Source of the image color
    pub render_mode: RenderMode,
    /// Which lights should be used
    pub light_mode: LightMode,
    /// Maximum number of iterations. None corresponds to manual stop.
    pub max_iterations: Option<usize>,
    /// Russian roulette on or off
    pub russian_roulette: bool,
    /// Multiple importance sampling on or off
    pub mis: bool,
    /// Number of bounces before starting russian roulette or terminating the path.
    pub bounces: usize,
    /// Samples per pixel per direction. Squared to get the total samples per pixel.
    pub samples_per_dir: usize,
    /// Should tone mapping be used
    pub tone_map: bool,
    /// Splitting method for bvh
    pub bvh_split: SplitMode,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            width: 1000,
            height: 800,
            max_threads: num_cpus::get_physical(),
            normal_mapping: true,
            render_mode: RenderMode::PathTracing,
            light_mode: LightMode::Scene,
            max_iterations: None,
            russian_roulette: true,
            mis: true,
            bounces: 5,
            samples_per_dir: 2,
            tone_map: true,
            bvh_split: SplitMode::SAH,
        }
    }
}

impl RenderConfig {
    pub fn bdpt() -> Self {
        Self {
            render_mode: RenderMode::BDPT,
            russian_roulette: false,
            ..Default::default()
        }
    }

    pub fn benchmark() -> Self {
        Self {
            width: 600,
            height: 400,
            max_threads: 8,
            normal_mapping: true,
            render_mode: RenderMode::PathTracing,
            light_mode: LightMode::Scene,
            max_iterations: Some(1),
            russian_roulette: false,
            mis: true,
            bounces: 5,
            samples_per_dir: 3,
            tone_map: true,
            bvh_split: SplitMode::SAH,
        }
    }

    pub fn bdpt_benchmark() -> Self {
        Self {
            render_mode: RenderMode::BDPT,
            ..Self::benchmark()
        }
    }

    pub fn high_quality() -> Self {
        Self {
            width: 800,
            height: 600,
            samples_per_dir: 50,
            max_iterations: Some(1),
            ..Default::default()
        }
    }

    pub fn debug_normals() -> Self {
        Self {
            normal_mapping: true,
            render_mode: RenderMode::Debug(DebugMode::Normals),
            russian_roulette: false,
            bounces: 0,
            samples_per_dir: 1,
            tone_map: false,
            ..Default::default()
        }
    }

    pub fn forward_normals() -> Self {
        Self {
            render_mode: RenderMode::Debug(DebugMode::ForwardNormals),
            ..Self::debug_normals()
        }
    }

    #[allow(dead_code)]
    pub fn single_threaded(self) -> Self {
        Self {
            max_threads: 1,
            ..self
        }
    }

    pub fn dimensions(&self) -> LogicalSize {
        LogicalSize::from((self.width, self.height))
    }

    pub fn handle_key(&mut self, key: VirtualKeyCode) {
        match key {
            VirtualKeyCode::N => {
                self.normal_mapping = !self.normal_mapping;
                println!("Normal mapping: {}", self.normal_mapping);
            }
            VirtualKeyCode::M => {
                self.mis = !self.mis;
                println!("MIS: {}", self.mis);
            }
            VirtualKeyCode::L => {
                self.light_mode = match self.light_mode {
                    LightMode::Scene => {
                        println!("Lightmode: Camera");
                        LightMode::Camera
                    }
                    LightMode::Camera => {
                        println!("Lightmode: Scene");
                        LightMode::Scene
                    }
                }
            }
            VirtualKeyCode::F1 => {
                println!("Config: Default");
                *self = Self::default();
            }
            VirtualKeyCode::F2 => {
                println!("Config: BDPT");
                *self = Self::bdpt();
            }
            VirtualKeyCode::F3 => {
                println!("Config: Debug normals");
                *self = Self::debug_normals();
            }
            VirtualKeyCode::F4 => {
                println!("Config: Forward normals");
                *self = Self::forward_normals();
            }
            _ => (),
        }
    }
}
