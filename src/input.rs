use std::collections::HashMap;
use std::time::Instant;

use glium::glutin::{
    dpi::PhysicalPosition,
    event::{ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent},
};

pub struct InputState {
    /// Position of the mouse
    pub mouse_pos: (f64, f64),
    /// Previous position of the mouse
    prev_mouse_pos: (f64, f64),
    /// Currently pressed mouse buttons with the time of the press
    pub mouse_presses: HashMap<MouseButton, Instant>,
    /// Currently pressed keys with the time of the press
    pub key_presses: HashMap<VirtualKeyCode, Instant>,
    /// Time of the last reset
    pub last_reset: Instant,
}

impl InputState {
    /// Get a new empty input state
    pub fn new() -> InputState {
        InputState {
            mouse_pos: (0.0, 0.0),
            prev_mouse_pos: (0.0, 0.0),
            mouse_presses: HashMap::new(),
            key_presses: HashMap::new(),
            last_reset: Instant::now(),
        }
    }

    pub fn d_mouse(&self) -> (f64, f64) {
        let (px, py) = self.prev_mouse_pos;
        let (x, y) = self.mouse_pos;
        (x - px, y - py)
    }

    /// Update the state with an event
    pub fn update(&mut self, event: &Event<()>) {
        if let Event::WindowEvent { ref event, .. } = *event {
            match *event {
                WindowEvent::CursorMoved {
                    position: PhysicalPosition { x, y },
                    ..
                } => {
                    self.mouse_pos = (x, y);
                }
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button,
                    ..
                } => {
                    self.mouse_presses
                        .entry(button)
                        .or_insert_with(Instant::now);
                }
                WindowEvent::MouseInput {
                    state: ElementState::Released,
                    button,
                    ..
                } => {
                    self.mouse_presses.remove(&button);
                }
                WindowEvent::KeyboardInput { input, .. } => match input {
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(key),
                        ..
                    } => {
                        self.key_presses.entry(key).or_insert_with(Instant::now);
                    }
                    KeyboardInput {
                        state: ElementState::Released,
                        virtual_keycode: Some(key),
                        ..
                    } => {
                        self.key_presses.remove(&key);
                    }
                    _ => (),
                },
                WindowEvent::Focused(false) => {
                    self.mouse_presses.clear();
                    self.key_presses.clear();
                }
                _ => (),
            }
        }
    }

    /// Reset the delta values after a loop
    pub fn reset_deltas(&mut self) {
        self.prev_mouse_pos = self.mouse_pos;
        self.last_reset = Instant::now();
    }
}
