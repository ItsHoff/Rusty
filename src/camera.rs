/// Module containing the camera functionality

use cgmath;
use cgmath::prelude::*;
use cgmath::{Point3, Vector3, Matrix4, Matrix3, Rad};

use glium::glutin::{Event, ElementState, VirtualKeyCode};

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
    /// Movement speed of the camera
    move_speed: f32,
    /// Rotation speed of the camera
    rotation_speed: Rad<f32>
}

const ACCEL: f32 = 1.05;
const V_MOVE: f32 = 0.05;
const V_ROTATION: Rad<f32> = Rad(0.02);

impl Default for Camera {
    fn default() -> Camera {
        Camera {
            pos: Point3 {x: 0.0, y: 0.0, z: 0.0},
            dir: Vector3 {x: 0.0, y: 0.0, z: 1.0},
            up: Vector3 {x: 0.0, y: 1.0, z: 0.0},
            fov: Rad(::std::f32::consts::PI / 3.0),
            near: 0.01,
            far: 1000.0,
            move_speed: V_MOVE,
            rotation_speed: V_ROTATION
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

    /// Helper function to move the camera to the given direction
    fn translate(&mut self, local_dir: Vector3<f32>) {
        let inverse_rotation = self.get_rotation().invert().expect("Non invertable camera rotation!");
        let movement = self.move_speed * inverse_rotation * local_dir;
        self.pos += movement;
        self.move_speed *= ACCEL;
    }

    /// Helper function to rotate the camera around the given axis
    fn rotate(&mut self, local_axis: Vector3<f32>) {
        let inverse_rotation = self.get_rotation().invert().expect("Non invertable camera rotation!");
        let axis = inverse_rotation * local_axis;
        self.dir = Matrix3::from_axis_angle(axis, self.rotation_speed) * self.dir;
        self.rotation_speed *= ACCEL;
    }

    /// Move camera based on input event
    pub fn handle_event(&mut self, event: &Event) {
        match *event {
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::W)) => {
                self.translate(-Vector3::unit_z())
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::S)) => {
                self.translate(Vector3::unit_z())
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::A)) => {
                self.translate(-Vector3::unit_x())
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::D)) => {
                self.translate(Vector3::unit_x())
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Q)) => {
                self.translate(-Vector3::unit_y())
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::E)) => {
                self.translate(Vector3::unit_y())
            }

            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Up)) |
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::O)) => {
                self.rotate(Vector3::unit_x())
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Down)) |
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::L)) => {
                self.rotate(-Vector3::unit_x())
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Left)) |
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::K)) => {
                self.rotate(Vector3::unit_y())
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Right)) |
            Event::KeyboardInput(ElementState::Pressed, 39, _) => {  // Ö
                self.rotate(-Vector3::unit_y())
            }
            Event::KeyboardInput(ElementState::Released, _, _) => {
                self.move_speed = V_MOVE;
                self.rotation_speed = V_ROTATION;
            }
            _ => ()
        }
    }
}
