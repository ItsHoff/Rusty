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
}


impl Default for Camera {
    fn default() -> Camera {
        Camera {
            pos: Point3 {x: 0.0, y: 0.0, z: 0.0},
            dir: Vector3 {x: 0.0, y: 0.0, z: 1.0},
            up: Vector3 {x: 0.0, y: 1.0, z: 0.0},
            fov: Rad(::std::f32::consts::PI / 3.0),
            near: 0.01,
            far: 1000.0,
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

    /// Get the world to camera transformation matrix
    pub fn get_world_to_camera(&self) -> Matrix4<f32> {
        Matrix4::from(self.get_rotation()) * Matrix4::from_translation(-self.pos.to_vec())
    }

    /// Get the camera to clip space transformation matrix
    pub fn get_camera_to_clip(&self, w: u32, h: u32) -> Matrix4<f32> {
        cgmath::perspective(self.fov, w as f32 / h as f32, self.near, self.far)
    }

    /// Get the camera rotation matrix
    fn get_rotation(&self) -> Matrix3<f32> {
        Matrix3::look_at(-self.dir, self.up)
    }

    /// Get the speed of the camera
    fn get_speed(dt: Duration) -> f32 {
        // Use tanh acceleration curve
        let x = dt.as_secs() as f32 + dt.subsec_nanos() as f32 / 1e9 - 3.0;
        let tanh = x.tanh() + 1.0;
        tanh * 0.5
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
    pub fn process_input(&mut self, input: &InputState) {
        for (key, t) in &input.key_presses {
            let dt = t.elapsed();  // Length of the key press
            match *key {
                VirtualKeyCode::W => {
                    self.translate(-Vector3::unit_z(), Self::get_speed(dt))
                }
                VirtualKeyCode::S => {
                    self.translate(Vector3::unit_z(), Self::get_speed(dt))
                }
                VirtualKeyCode::A => {
                    self.translate(-Vector3::unit_x(), Self::get_speed(dt))
                }
                VirtualKeyCode::D => {
                    self.translate(Vector3::unit_x(), Self::get_speed(dt))
                }
                VirtualKeyCode::Q => {
                    self.translate(-Vector3::unit_y(), Self::get_speed(dt))
                }
                VirtualKeyCode::E => {
                    self.translate(Vector3::unit_y(), Self::get_speed(dt))
                }

                VirtualKeyCode::Up => {
                    self.rotate(Vector3::unit_x(), Rad(0.5 * Self::get_speed(dt)))
                }
                VirtualKeyCode::Down => {
                    self.rotate(-Vector3::unit_x(), Rad(0.5 * Self::get_speed(dt)))
                }
                VirtualKeyCode::Left => {
                    self.rotate(Vector3::unit_y(), Rad(0.5 * Self::get_speed(dt)))
                }
                VirtualKeyCode::Right => {
                    self.rotate(-Vector3::unit_y(), Rad(0.5 * Self::get_speed(dt)))
                }
                _ => ()
            }
        }
        for (button, _) in &input.mouse_presses {
            match *button {
                MouseButton::Left => {
                    let (dx, dy) = input.d_mouse;
                    let dx = dx as f32 / 10.0;
                    let dy = dy as f32 / 10.0;
                    self.rotate(-Vector3::unit_y(), Rad(0.025 * dx.tanh()));
                    self.rotate(-Vector3::unit_x(), Rad(0.025 * dy.tanh()));
                }
                _ => ()
            }
        }
    }
}
