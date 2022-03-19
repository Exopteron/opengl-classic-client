use glutin::event::KeyboardInput;

use self::{mouse::MouseInput, key::KeyboardState};

pub mod key;
pub mod mouse;

/// Handles user input.
/// (mouse, keyboard etc.)
#[derive(Default)]
pub struct InputManager {
    /// Current state of the keyboard.
    pub keyboard: KeyboardState,
    /// Current state of the mouse.
    pub mouse: MouseInput,
}