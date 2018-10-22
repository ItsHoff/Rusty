/// Module containing the camera functionality
use std::time::Duration;

use cgmath;
use cgmath::prelude::*;
use cgmath::{Matrix4, Point3, Quaternion, Rad, Vector3};

use glium::glutin::{MouseButton, VirtualKeyCode};

use crate::consts::PI;
use crate::input::InputState;
use crate::Float;

/// Representation of a camera
#[derive(Clone)]
pub struct Camera {
    /// Position of the camera in world coordinates
    pub pos: Point3<Float>,
    /// Rotation of the camera
    pub rot: Quaternion<Float>,
    /// Width of the viewport in pixels
    pub width: u32,
    /// Height of the viewport in pixels
    pub height: u32,
    /// Vertical field-of-view of the camera
    fov: Rad<Float>,
    /// Near plane of the camera
    near: Float,
    /// Far plane of the camera
    far: Float,
    /// Size of the scene
    pub scale: Float,
}

impl Default for Camera {
    fn default() -> Camera {
        Camera {
            pos: Point3::origin(),
            rot: Quaternion::one(),
            width: 0,
            height: 0,
            fov: Rad(PI / 3.0),
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

    pub fn update_viewport(&mut self, size: (u32, u32)) {
        self.width = size.0;
        self.height = size.1;
    }

    pub fn set_scale(&mut self, scale: Float) {
        self.scale = scale;
    }

    /// Get the world to camera transformation matrix
    fn world_to_camera(&self) -> Matrix4<Float> {
        Matrix4::from(self.rot.invert()) * Matrix4::from_translation(-self.pos.to_vec())
    }

    /// Get the camera to clip space transformation matrix
    #[allow(clippy::cast_lossless)]
    fn camera_to_clip(&self) -> Matrix4<Float> {
        cgmath::perspective(
            self.fov,
            self.width as Float / self.height as Float,
            self.near * self.scale,
            self.far * self.scale,
        )
    }

    /// Get the combined world to clip transformation
    pub fn world_to_clip(&self) -> Matrix4<Float> {
        self.camera_to_clip() * self.world_to_camera()
    }

    pub fn world_to_clip_f32(&self) -> Matrix4<f32> {
        let rot = Quaternion::new(
            self.rot.s as f32,
            self.rot.v.x as f32,
            self.rot.v.y as f32,
            self.rot.v.z as f32,
        )
        .invert();
        let rot_m = Matrix4::from(rot);
        let t = -Vector3::new(self.pos.x as f32, self.pos.y as f32, self.pos.z as f32);
        let world_to_camera = rot_m * Matrix4::from_translation(t);
        let camera_to_clip = cgmath::perspective(
            Rad(self.fov.0 as f32),
            self.width as f32 / self.height as f32,
            (self.near * self.scale) as f32,
            (self.far * self.scale) as f32,
        );
        camera_to_clip * world_to_camera
    }

    /// Get the forward axis of the camera in the world frame
    pub fn forward(&self) -> Vector3<Float> {
        self.rot.rotate_vector(-Vector3::unit_z())
    }

    /// Get the speed of the camera based on the duration of the input
    fn get_speed(dt: Duration) -> Float {
        // Use tanh acceleration curve
        let x = dt.as_float_secs() as Float - 2.0;
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
        let time_scale = 10.0 * dt.as_float_secs() as Float;
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
                let (dx, dy) = input.d_mouse;
                self.rotate_y(-Rad(dx as Float / 250.0));
                self.rotate_x(-Rad(dy as Float / 250.0));
            }
        }
    }
}
