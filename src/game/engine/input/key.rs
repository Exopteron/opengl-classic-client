use ahash::AHashMap;
use fxhash::FxHashMap;
use glutin::event::{ElementState, VirtualKeyCode};

/// Keeps track of which keys have been pressed.
pub struct KeyboardState {
    state: FxHashMap<VirtualKeyCode, (ElementState, bool)>,
    callbacks: AHashMap<VirtualKeyCode, Box<dyn FnMut(ElementState)>>,
}
impl Default for KeyboardState {
    /// Constructs a new KeyboardState with all the keys released.
    fn default() -> KeyboardState {
        KeyboardState {
            state: FxHashMap::default(),
            callbacks: AHashMap::new(),
        }
    }
}
impl KeyboardState {
    pub fn set_callback(
        &mut self,
        code: VirtualKeyCode,
        callback: impl FnMut(ElementState) + 'static,
    ) {
        self.callbacks.insert(code, Box::new(callback));
    }
    /// Returns true if `key` was pressed.
    pub fn was_pressed(&mut self, key: VirtualKeyCode) -> bool {
        if let Some(v) = self.state.get_mut(&key) {
            if v.1 {
                v.1 = false;
                return true;
            }
        }
        false
    }
    /// Returns true if `key` is pressed.
    pub fn is_pressed(&self, key: VirtualKeyCode) -> bool {
        self.state
            .get(&key)
            .map(|&s| s.0 == ElementState::Pressed)
            .unwrap_or(false)
    }
    /// Returns true if `key` is released.
    pub fn is_released(&self, key: VirtualKeyCode) -> bool {
        !self.is_pressed(key)
    }

    /// Processes a keyboard event and updated the internal state.
    pub fn process_event(&mut self, key_state: ElementState, code: VirtualKeyCode) {
        if let Some(cb) = self.callbacks.get_mut(&code) {
            cb(key_state);
        }
        match key_state {
            ElementState::Pressed => {
                self.state.insert(code, (key_state, true));
            }
            ElementState::Released => {
                self.state.remove(&code);
            }
        }
    }
}
