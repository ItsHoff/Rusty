/// Module containing the camera functionality

use std::time::Duration;

use cgmath;
use cgmath::prelude::*;
use cgmath::{Point3, Vector3, Matrix4, Matrix3, Rad};

use glium::glutin::{MouseButton, VirtualKeyCode};

use input::InputState;

/// Representation of a camera
pub struct Camera {
    /// Position of the camera in world coordinates
    pos: Point3<f32>,
    /// Direction the camera is looking at in world coordinates
    dir: Vector3<f32>,
    /// Definition of camera up in world coordinates
    up: Vector3<f32>,
    /// Vertical field-of-view of the camera
    fov: Rad<f32>,
    /// Near plane of the camera
    near: f32,
    /// Far plane of the camera
    far: f32,
    /// Size of the scene
    scale: f32
}


impl Default for Camera {
    fn default() -> Camera {
        Camera {
            pos: Point3 {x: 0.0, y: 0.0, z: 0.0},
            dir: Vector3 {x: 0.0, y: 0.0, z: 1.0},
            up: Vector3 {x: 0.0, y: 1.0, z: 0.0},
            fov: Rad(::std::f32::consts::PI / 3.0),
            near: 0.001,
            far: 10.0,
            scale: 1.0
        }
    }
}

impl Camera {
    pub fn new(pos: Point3<f32>, dir: Vector3<f32>) -> Camera {
        Camera { pos: pos, dir: dir, .. Default::default() }
    }

    /// Move the camera to new position
    pub fn set_position(&mut self, pos: Point3<f32>, dir: Vector3<f32>) {
        self.pos = pos;
        self.dir = dir;
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }

    /// Get the world to camera transformation matrix
    pub fn get_world_to_camera(&self) -> Matrix4<f32> {
        Matrix4::from(self.get_rotation()) * Matrix4::from_translation(-self.pos.to_vec())
    }

    /// Get the camera to clip space transformation matrix
    pub fn get_camera_to_clip(&self, w: u32, h: u32) -> Matrix4<f32> {
        cgmath::perspective(self.fov, w as f32 / h as f32, self.near * self.scale, self.far * self.scale)
    }

    /// Get the camera rotation matrix
    fn get_rotation(&self) -> Matrix3<f32> {
        Matrix3::look_at(-self.dir, self.up)
    }

    /// Get the speed of the camera based on the duration of the input
    fn get_speed(dt: Duration) -> f32 {
        // Use tanh acceleration curve
        let x = dt.as_secs() as f32 + dt.subsec_nanos() as f32 / 1e9 - 2.0;
        x.tanh() + 1.05
    }

    /// Helper function to move the camera to the given direction
    fn translate(&mut self, local_dir: Vector3<f32>, distance: f32) {
        let inverse_rotation = self.get_rotation().invert().expect("Non invertable camera rotation!");
        let movement = distance * inverse_rotation * local_dir;
        self.pos += movement;
    }

    /// Helper function to rotate the camera around the given axis
    fn rotate(&mut self, local_axis: Vector3<f32>, angle: Rad<f32>) {
        let inverse_rotation = self.get_rotation().invert().expect("Non invertable camera rotation!");
        let axis = inverse_rotation * local_axis;
        self.dir = Matrix3::from_axis_angle(axis, angle) * self.dir;
    }

    /// Move camera based on input event
    #[cfg_attr(feature="clippy", allow(single_match))]
    pub fn process_input(&mut self, input: &InputState) {
        let dt = input.last_reset.elapsed();
        let time_scale = (dt.as_secs() as f32 * 1e9 + dt.subsec_nanos() as f32) / 1e8;
        for (key, t) in &input.key_presses {
            let t_press = t.elapsed();  // Length of the key press
            let move_speed = time_scale * self.scale.sqrt().min(self.scale) * Self::get_speed(t_press);
            let rotation_speed = 3.0 * time_scale * Self::get_speed(t_press);
            match *key {
                // Move with wasd + e, q for up and down
                VirtualKeyCode::W => self.translate(-Vector3::unit_z(), move_speed),
                VirtualKeyCode::S => self.translate(Vector3::unit_z(), move_speed),
                VirtualKeyCode::A => self.translate(-Vector3::unit_x(), move_speed),
                VirtualKeyCode::D => self.translate(Vector3::unit_x(), move_speed),
                VirtualKeyCode::Q => self.translate(-Vector3::unit_y(), move_speed),
                VirtualKeyCode::E => self.translate(Vector3::unit_y(), move_speed),

                // Rotate with arrow keys
                VirtualKeyCode::Up => self.rotate(Vector3::unit_x(), Rad(rotation_speed)),
                VirtualKeyCode::Down => self.rotate(-Vector3::unit_x(), Rad(rotation_speed)),
                VirtualKeyCode::Left => self.rotate(Vector3::unit_y(), Rad(rotation_speed)),
                VirtualKeyCode::Right => self.rotate(-Vector3::unit_y(), Rad(rotation_speed)),
                _ => ()
            }
        }
        for button in input.mouse_presses.keys() {
            match *button {
                // Rotate camera while holding left mouse button
                MouseButton::Left => {
                    let (dx, dy) = input.d_mouse;
                    self.rotate(-Vector3::unit_y(), Rad(dx as f32 / 250.0));
                    self.rotate(-Vector3::unit_x(), Rad(dy as f32 / 250.0));
                },
                _ => ()
            }
        }
    }
}
