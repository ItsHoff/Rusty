/// Module containing the camera functionality

use cgmath::prelude::*;
use cgmath::{Point3, Vector3, Matrix4, Matrix3, Rad};

use glium::glutin::{Event, ElementState, VirtualKeyCode};

/// Representation of a camera
pub struct Camera {
    /// Position of the camera
    pos: Point3<f32>,
    /// Direction the camera is looking at
    dir: Vector3<f32>,
    /// Definition of camera up
    up: Vector3<f32>
}

impl Camera {
    pub fn new(pos: Point3<f32>, dir: Vector3<f32>) -> Camera {
        Camera { pos: pos, dir: dir, up: Vector3::unit_y() }
    }

    /// Get the world to camera transformation matrix
    pub fn get_world_to_camera(&self) -> Matrix4<f32> {
        Matrix4::from(Matrix3::look_at(self.dir, self.up)) * Matrix4::from_translation(self.pos.to_vec())
    }

    /// Get the camera rotation matrix
    fn get_rotation(&self) -> Matrix3<f32> {
        Matrix3::look_at(self.dir, self.up)
    }

    /// Move camera based on input event
    pub fn handle_event(&mut self, event: &Event) {
        let move_speed = 0.1;
        let rotation_speed = Rad(0.05);
        let inverse_rotation = self.get_rotation().invert().expect("Non invertable camera rotation!");

        // Closure to move the camera
        let translate = |pos: Point3<f32>, local_dir: Vector3<f32>| -> Point3<f32> {
            let movement = move_speed * inverse_rotation * local_dir;
            pos + movement
        };

        // Closure to rotate the camera
        let rotate = |vec: Vector3<f32>, local_axis: Vector3<f32>| -> Vector3<f32> {
            let axis = inverse_rotation * local_axis;
            Matrix3::from_axis_angle(axis, rotation_speed) * vec
        };

        match *event {
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::W)) => {
                self.pos = translate(self.pos, Vector3::unit_z())
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::S)) => {
                self.pos = translate(self.pos, -Vector3::unit_z())
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::A)) => {
                self.pos = translate(self.pos, Vector3::unit_x())
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::D)) => {
                self.pos = translate(self.pos, -Vector3::unit_x())
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::E)) => {
                self.pos = translate(self.pos, -Vector3::unit_y())
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Q)) => {
                self.pos = translate(self.pos, Vector3::unit_y())
            }

            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Up)) |
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::O)) => {
                self.dir = rotate(self.dir, Vector3::unit_x()).normalize()
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Down)) |
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::L)) => {
                self.dir = rotate(self.dir, -Vector3::unit_x()).normalize()
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Left)) |
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::K)) => {
                self.dir = rotate(self.dir, Vector3::unit_y()).normalize()
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Right)) |
            Event::KeyboardInput(ElementState::Pressed, 39, _) => {  // Ö
                self.dir = rotate(self.dir, -Vector3::unit_y()).normalize()
            }
            Event::KeyboardInput(ElementState::Pressed, code, _) => {
                println!("{:?}", code)
            }
            _ => ()
        }
    }
}
