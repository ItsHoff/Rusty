use glium::glutin::{dpi::LogicalSize, event::VirtualKeyCode};

use crate::bvh::SplitMode;
use crate::float::*;

#[derive(Clone, Debug)]
pub enum RenderMode {
    /// Standard path tracing
    PathTracing,
    /// Bidirectional path tracing
    Bdpt,
    /// Debug
    Debug(DebugMode),
}

#[derive(Clone, Debug)]
pub enum DebugMode {
    /// Normals
    Normals,
    /// Normals that point away from the camera
    ForwardNormals,
}

#[derive(Clone, Debug)]
pub enum LightMode {
    /// Use scene lights only (will still fall back to camera if there are none)
    Scene,
    /// Use camera flash as the light source
    Camera,
}

#[derive(Clone, Debug)]
pub enum RussianRoulette {
    /// Select survival probability based on path throughput
    Dynamic,
    /// Constant survival probability
    Static(Float),
    /// No russian roulette
    Off,
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
    /// Type of russian roulette
    pub russian_roulette: RussianRoulette,
    /// Multiple importance sampling on or off
    pub mis: bool,
    /// Number of bounces before starting russian roulette.
    /// Won't have effect is russian roulette is off.
    pub pre_rr_bounces: usize,
    /// Maximum number of bounces allowed before path is terminated.
    // std::usize::MAX should suffice for "unlimited" bounces
    pub max_bounces: usize,
    /// Samples per pixel per direction. Squared to get the total samples per pixel.
    pub samples_per_dir: usize,
    /// Should tone mapping be used
    pub tone_map: bool,
    /// Splitting method for bvh
    pub bvh_split: SplitMode,
}

impl RenderConfig {
    fn path_trace() -> Self {
        Self {
            width: 1000,
            height: 800,
            max_threads: num_cpus::get_physical(),
            normal_mapping: true,
            render_mode: RenderMode::PathTracing,
            light_mode: LightMode::Scene,
            max_iterations: None,
            russian_roulette: RussianRoulette::Dynamic,
            mis: true,
            pre_rr_bounces: 5,
            max_bounces: std::usize::MAX,
            samples_per_dir: 2,
            tone_map: true,
            bvh_split: SplitMode::Sah,
        }
    }

    pub fn bdpt() -> Self {
        Self {
            render_mode: RenderMode::Bdpt,
            pre_rr_bounces: 3,
            max_bounces: std::usize::MAX,
            russian_roulette: RussianRoulette::Static(0.5),
            ..Self::path_trace()
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
            russian_roulette: RussianRoulette::Off,
            mis: true,
            pre_rr_bounces: 5,
            max_bounces: 5,
            samples_per_dir: 3,
            tone_map: true,
            bvh_split: SplitMode::Sah,
        }
    }

    pub fn bdpt_benchmark() -> Self {
        Self {
            render_mode: RenderMode::Bdpt,
            samples_per_dir: 2,
            ..Self::benchmark()
        }
    }

    pub fn high_quality() -> Self {
        Self {
            width: 300,
            height: 200,
            samples_per_dir: 50,
            max_iterations: Some(1),
            ..Self::bdpt()
        }
    }

    pub fn high_quality_pt() -> Self {
        Self {
            width: 300,
            height: 200,
            samples_per_dir: 160, //148,
            max_iterations: Some(1),
            ..Self::path_trace()
        }
    }

    pub fn debug_normals() -> Self {
        Self {
            normal_mapping: true,
            render_mode: RenderMode::Debug(DebugMode::Normals),
            russian_roulette: RussianRoulette::Off,
            pre_rr_bounces: 0,
            max_bounces: 0,
            samples_per_dir: 1,
            tone_map: false,
            ..Self::path_trace()
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
        println!("Running single threaded!");
        Self {
            max_threads: 1,
            ..self
        }
    }

    pub fn dimensions(&self) -> LogicalSize<Float> {
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
                println!("Config: Path trace");
                *self = Self::path_trace();
            }
            VirtualKeyCode::F2 => {
                println!("Config: Bdpt");
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
