use std::collections::HashMap;
use std::time::Instant;

use glium::glutin::{Event, ElementState, MouseButton, VirtualKeyCode};

pub struct InputState {
    /// Position of the mouse
    pub mouse_pos: (i32, i32),
    /// Movement of the mouse
    pub d_mouse: (i32, i32),
    /// Currently pressed mouse buttons with the time of the press
    pub mouse_presses: HashMap<MouseButton, Instant>,
    /// Currently pressed keys with the time of the press
    pub key_presses: HashMap<VirtualKeyCode, Instant>,
    /// Time of the last reset
    pub last_reset: Instant
}

impl InputState {
    /// Get a new empty input state
    pub fn new() -> InputState {
        InputState { mouse_pos: (0, 0),
                     d_mouse: (0, 0),
                     mouse_presses: HashMap::new(),
                     key_presses: HashMap::new(),
                     last_reset: Instant::now()
        }
    }

    /// Update the state with an event
    pub fn update(&mut self, event: &Event) {
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

    /// Reset the delta values after a loop
    pub fn reset_deltas(&mut self) {
        self.d_mouse = (0, 0);
        self.last_reset = Instant::now();
    }
}
