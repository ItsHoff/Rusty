use std::ops::Deref;

use crate::color::Color;
use crate::light::{Light, PointLight};

use super::Camera;

/// Extended camera for path tracing
pub struct PTCamera {
    camera: Camera,
    flash: PointLight,
}

impl PTCamera {
    pub fn new(camera: Camera) -> Self {
        let intensity = 10.0 * camera.scale.powf(1.4) * Color::white();
        let flash = PointLight::new(camera.pos, intensity);
        Self { camera, flash }
    }

    pub fn flash(&self) -> &dyn Light {
        &self.flash
    }
}

impl Deref for PTCamera {
    type Target = Camera;

    fn deref(&self) -> &Self::Target {
        &self.camera
    }
}
