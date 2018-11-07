use glium::glutin::VirtualKeyCode;

use crate::Float;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum ColorMode {
    /// Standard radiance
    Radiance,
    /// Normals
    DebugNormals,
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
    /// Use all light sources
    All,
}

#[derive(Clone, Debug)]
pub struct RenderConfig {
    /// Maximum number of threads to use for rendering
    pub max_threads: usize,
    /// Should normal mapping be used
    pub normal_mapping: bool,
    /// Source of the image color
    pub color_mode: ColorMode,
    /// Which lights should be used
    pub light_mode: LightMode,
    /// Maximum number of iterations. None corresponds to manual stop.
    pub max_iterations: Option<usize>,
    /// The russian roulette termination probability. None skips russian roulette.
    pub russian_roulette: Option<Float>,
    /// Number of bounces before starting russian roulette or terminating the path.
    pub bounces: usize,
    /// Samples per pixel per direction. Squared to get the total samples per pixel.
    pub samples_per_dir: usize,
    /// Should tone mapping be used
    pub tone_map: bool,
}

impl Default for RenderConfig {
    fn default() -> Self {
        // Desired expectation value of russian roulette bounces
        let eb = 2.0;
        // The matching survival probability from negative binomial distribution
        let surv_prob = eb / (eb + 1.0);

        RenderConfig {
            max_threads: 8,
            normal_mapping: true,
            color_mode: ColorMode::Radiance,
            light_mode: LightMode::Scene,
            max_iterations: None,
            russian_roulette: Some(1.0 - surv_prob),
            bounces: 5,
            samples_per_dir: 2,
            tone_map: true,
        }
    }
}

#[allow(dead_code)]
impl RenderConfig {
    pub fn benchmark() -> Self {
        RenderConfig {
            max_iterations: Some(2),
            ..Default::default()
        }
    }

    pub fn debug_normals() -> Self {
        RenderConfig {
            normal_mapping: true,
            color_mode: ColorMode::DebugNormals,
            russian_roulette: None,
            bounces: 0,
            samples_per_dir: 1,
            tone_map: false,
            ..Default::default()
        }
    }

    pub fn forward_normals() -> Self {
        let mut c = Self::debug_normals();
        c.color_mode = ColorMode::ForwardNormals;
        c
    }

    pub fn handle_key(&mut self, key: VirtualKeyCode) {
        match key {
            VirtualKeyCode::N => self.normal_mapping = !self.normal_mapping,
            VirtualKeyCode::L => {
                self.light_mode = match self.light_mode {
                    LightMode::Scene => LightMode::Camera,
                    LightMode::Camera => LightMode::Scene,
                    LightMode::All => unimplemented!(),
                }
            }
            VirtualKeyCode::F1 => *self = Self::default(),
            VirtualKeyCode::F2 => *self = Self::debug_normals(),
            VirtualKeyCode::F3 => *self = Self::forward_normals(),
            _ => (),
        }
    }
}
