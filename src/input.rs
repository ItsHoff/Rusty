use std::collections::HashMap;
use std::time::Instant;

use glium::glutin::{Event, ElementState, MouseButton, VirtualKeyCode};

#[derive(Default)]
pub struct InputState {
    /// Position of the mouse
    pub mouse_pos: (i32, i32),
    /// Movement of the mouse
    pub d_mouse: (i32, i32),
    /// Currently pressed mouse buttons with the time of the press
    pub mouse_presses: HashMap<MouseButton, Instant>,
    /// Currently pressed keys with the time of the press
    pub key_presses: HashMap<VirtualKeyCode, Instant>
}

impl InputState {
    /// Get a new empty input state
    pub fn new() -> InputState {
        InputState { .. Default::default() }
    }

    /// Update the state with an event
    pub fn update(&mut self, event: &Event) {
        self.d_mouse = (0, 0);
        match *event {
            Event::MouseMoved(x, y) => {
                self.d_mouse = (x - self.mouse_pos.0, y - self.mouse_pos.1);
                self.mouse_pos = (x, y);
            }
            Event::MouseInput(ElementState::Pressed, button) => {
                if !self.mouse_presses.contains_key(&button) {
                    self.mouse_presses.insert(button, Instant::now());
                }
            }
            Event::MouseInput(ElementState::Released, button) => {
               self.mouse_presses.remove(&button);
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(key)) => {
                if !self.key_presses.contains_key(&key) {
                    self.key_presses.insert(key, Instant::now());
                }
            }
            Event::KeyboardInput(ElementState::Released, _, Some(key)) => {
                self.key_presses.remove(&key);
            }
            Event::Focused(false) => {
                self.mouse_presses.clear();
                self.key_presses.clear();
            }
            _ => ()
        }
    }
}
