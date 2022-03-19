use glutin::event::Event;

use crate::render::window::GameWindow;

use super::GameEngine;

type EventHandler<T> = dyn FnMut(&mut GameEngine<T>, &mut GameWindow, &mut T, &Event<()>);

/// Manages all window events that make 
/// it to the game engine.
pub struct EventManager<T: 'static> {
    handlers: Vec<Box<EventHandler<T>>>,
}
impl<T> Default for EventManager<T> {
    fn default() -> Self {
        Self { handlers: Vec::new() }
    }
}
impl<T> EventManager<T> {
    /// Adds a new event handler to the list.
    pub fn add_handler(&mut self, handler: impl FnMut(&mut GameEngine<T>, &mut GameWindow, &mut T, &Event<()>) + 'static) {
        self.handlers.push(Box::new(handler));
    }

    pub fn run(&mut self, engine: &mut GameEngine<T>, window: &mut GameWindow, v: &mut T, event: Event<()>) {
        for handler in self.handlers.iter_mut() {
            handler(engine, window, v, &event);
        }
    }
}