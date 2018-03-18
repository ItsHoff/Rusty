use std::collections::HashMap;
use std::time::Instant;

use glium::glutin::{Event, WindowEvent, KeyboardInput, ElementState, MouseButton, VirtualKeyCode};

pub struct InputState {
    /// Position of the mouse
    pub mouse_pos: (f64, f64),
    /// Movement of the mouse
    pub d_mouse: (f64, f64),
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
        InputState { mouse_pos: (0.0, 0.0),
                     d_mouse: (0.0, 0.0),
                     mouse_presses: HashMap::new(),
                     key_presses: HashMap::new(),
                     last_reset: Instant::now()
        }
    }

    /// Update the state with an event
    pub fn update(&mut self, event: &Event) {
        if let Event::WindowEvent{ref event, ..} = *event {
            match *event {
                WindowEvent::CursorMoved{position:(x, y), ..} => {
                    self.d_mouse = (x - self.mouse_pos.0, y - self.mouse_pos.1);
                    self.mouse_pos = (x, y);
                }
                WindowEvent::MouseInput{state: ElementState::Pressed, button, ..} => {
                    self.mouse_presses.entry(button).or_insert_with(Instant::now);
                }
                WindowEvent::MouseInput{state: ElementState::Released, button, ..} => {
                    self.mouse_presses.remove(&button);
                }
                WindowEvent::KeyboardInput{input, ..} => {
                    match input {
                        KeyboardInput{state: ElementState::Pressed, virtual_keycode: Some(key), ..} => {
                            self.key_presses.entry(key).or_insert_with(Instant::now);
                        }
                        KeyboardInput{state: ElementState::Released, virtual_keycode: Some(key), ..} => {
                            self.key_presses.remove(&key);
                        }
                        _ => ()
                    }
                }
                WindowEvent::Focused(false) => {
                    self.mouse_presses.clear();
                    self.key_presses.clear();
                }
                _ => ()
            }
        }
    }

    /// Reset the delta values after a loop
    pub fn reset_deltas(&mut self) {
        self.d_mouse = (0.0, 0.0);
        self.last_reset = Instant::now();
    }
}
