//! Module containing the camera functionality
use std::ops::Deref;
use std::time::Duration;

use cgmath;
use cgmath::prelude::*;
use cgmath::{Matrix4, Point2, Point3, Quaternion, Rad, Vector3};

use glium::glutin::{dpi::LogicalSize, MouseButton, VirtualKeyCode};

use crate::color::Color;
use crate::consts;
use crate::float::*;
use crate::input::InputState;
use crate::light::{Light, PointLight};

/// Representation of a camera
#[derive(Clone, Debug)]
pub struct Camera {
    /// Position of the camera in world coordinates
    pub pos: Point3<Float>,
    /// Rotation of the camera
    rot: Quaternion<Float>,
    /// Aspect ratio of the viewport
    ratio: Float,
    /// Vertical field-of-view of the camera
    fov: Rad<Float>,
    /// Near plane of the camera
    near: Float,
    /// Far plane of the camera
    far: Float,
    /// Size of the scene
    pub scale: Float,
}

/// Extended camera for path tracing
#[derive(Debug)]
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

    /// Evaluate the geometric cosine with dir
    pub fn cos_g(&self, dir: Vector3<Float>) -> Float {
        dir.dot(self.rot * -Vector3::unit_z())
    }

    pub fn we(&self, dir: Vector3<Float>) -> Color {
        let cos_t = self.cos_g(dir);
        let clip_dir = self.world_to_clip() * dir.extend(0.0);
        if cos_t < consts::EPSILON {
            Color::black()
        } else {
            // Find the intersection with the image plane
            let clip_p = clip_dir.truncate() / clip_dir.z;
            if clip_p.x < -1.0 || clip_p.x > 1.0 || clip_p.y < -1.0 || clip_p.y > 1.0 {
                Color::black()
            } else {
                let area = 2.0;
                Color::white() / (area * cos_t.powi(4))
            }
        }
    }

    /// Evaluate pdf of sampling dir
    pub fn pdf_dir(&self, dir: Vector3<Float>) -> Float {
        let cos_t = self.cos_g(dir);
        let clip_dir = self.world_to_clip() * dir.extend(0.0);
        if cos_t < consts::EPSILON {
            0.0
        } else {
            // Find the intersection with the image plane
            let clip_p = clip_dir.truncate() / clip_dir.z;
            if clip_p.x < -1.0 || clip_p.x > 1.0 || clip_p.y < -1.0 || clip_p.y > 1.0 {
                0.0
            } else {
                let area = 2.0;
                // Directional pdf
                1.0 / (area * cos_t.powi(3))
            }
        }
    }

    /// Try to convert dir to clip plane position
    pub fn clip_pos(&self, dir: Vector3<Float>) -> Option<Point2<Float>> {
        let clip_dir = self.world_to_clip() * dir.extend(0.0);
        // Only accept direction coming from the front
        if clip_dir.z < consts::EPSILON {
            None
        } else {
            let clip_p = clip_dir.truncate() / clip_dir.z;
            if clip_p.x < -1.0 || clip_p.x > 1.0 || clip_p.y < -1.0 || clip_p.y > 1.0 {
                None
            } else {
                Some(Point2::new(clip_p.x, clip_p.y))
            }
        }
    }
}

impl Deref for PTCamera {
    type Target = Camera;

    fn deref(&self) -> &Self::Target {
        &self.camera
    }
}

impl Default for Camera {
    fn default() -> Camera {
        Camera {
            pos: Point3::origin(),
            rot: Quaternion::one(),
            ratio: 1.0,
            fov: Rad(consts::PI / 3.0),
            near: 0.001,
            far: 10.0,
            scale: 1.0,
        }
    }
}

impl Camera {
    pub fn new(pos: Point3<Float>, rot: Quaternion<Float>) -> Camera {
        Camera {
            pos,
            rot,
            ..Default::default()
        }
    }

    pub fn update_viewport(&mut self, size: LogicalSize) {
        self.ratio = (size.width / size.height).to_float();
    }

    pub fn set_scale(&mut self, scale: Float) {
        self.scale = scale;
    }

    /// Get the world to camera transformation matrix
    fn world_to_camera(&self) -> Matrix4<Float> {
        Matrix4::from(self.rot.invert()) * Matrix4::from_translation(-self.pos.to_vec())
    }

    /// Get the camera to clip space transformation matrix
    fn camera_to_clip(&self) -> Matrix4<Float> {
        cgmath::perspective(
            self.fov,
            self.ratio,
            self.near * self.scale,
            self.far * self.scale,
        )
    }

    /// Get the combined world to clip transformation
    pub fn world_to_clip(&self) -> Matrix4<Float> {
        self.camera_to_clip() * self.world_to_camera()
    }

    /// Get the forward axis of the camera in the world frame
    #[allow(dead_code)]
    pub fn forward(&self) -> Vector3<Float> {
        self.rot.rotate_vector(-Vector3::unit_z())
    }

    /// Get the speed of the camera based on the duration of the input
    fn get_speed(dt: Duration) -> Float {
        // Use tanh acceleration curve
        let x = dt.as_float_secs().to_float() - 2.0;
        x.tanh() + 1.05
    }

    /// Helper function to move the camera to the given direction
    fn translate(&mut self, local_dir: Vector3<Float>, distance: Float) {
        let movement = distance * self.rot.rotate_vector(local_dir);
        self.pos += movement;
    }

    /// Helper function to rotate the camera around local x-axis
    fn rotate_x(&mut self, angle: Rad<Float>) {
        let d_rot = Quaternion::from_angle_x(angle);
        let new_rot = self.rot * d_rot;
        // Make sure that the rotation doesn't flip y-axis
        if new_rot
            .rotate_vector(Vector3::unit_y())
            .dot(Vector3::unit_y())
            > 0.0
        {
            self.rot = new_rot;
        }
    }

    /// Helper function to rotate the camera around global y-axis
    fn rotate_y(&mut self, angle: Rad<Float>) {
        let d_rot = Quaternion::from_angle_y(angle);
        self.rot = d_rot * self.rot;
    }

    /// Move camera based on input event
    pub fn process_input(&mut self, input: &InputState) {
        let dt = input.last_reset.elapsed();
        let time_scale = 10.0 * dt.as_float_secs().to_float();
        for (key, t) in &input.key_presses {
            let t_press = t.elapsed(); // Length of the key press
            let d_pos = time_scale * self.scale.sqrt().min(self.scale) * Self::get_speed(t_press);
            let angle = Rad(3.0 * time_scale * Self::get_speed(t_press));
            match *key {
                // Move with wasd + e, q for up and down
                VirtualKeyCode::W => self.translate(-Vector3::unit_z(), d_pos),
                VirtualKeyCode::S => self.translate(Vector3::unit_z(), d_pos),
                VirtualKeyCode::A => self.translate(-Vector3::unit_x(), d_pos),
                VirtualKeyCode::D => self.translate(Vector3::unit_x(), d_pos),
                VirtualKeyCode::Q => self.translate(-Vector3::unit_y(), d_pos),
                VirtualKeyCode::E => self.translate(Vector3::unit_y(), d_pos),

                // Rotate with arrow keys
                VirtualKeyCode::Up => self.rotate_x(angle),
                VirtualKeyCode::Down => self.rotate_x(-angle),
                VirtualKeyCode::Left => self.rotate_y(angle),
                VirtualKeyCode::Right => self.rotate_y(-angle),
                _ => (),
            }
        }
        for button in input.mouse_presses.keys() {
            // Rotate camera while holding left mouse button
            if let MouseButton::Left = *button {
                let (dx, dy) = input.d_mouse();
                let scale = 1.0 / 250.0;
                self.rotate_y(-Rad(scale * dx.to_float()));
                self.rotate_x(-Rad(scale * dy.to_float()));
            }
        }
    }
}
