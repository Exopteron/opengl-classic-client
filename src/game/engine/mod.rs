use std::{rc::Rc, cell::RefCell, marker::PhantomData, time::Duration};

use glutin::{dpi::Size, event::{Event, WindowEvent, DeviceEvent}};

use crate::render::{stage::{RenderStageObj, RenderStage}, RenderEngine, window::GameWindow};

use self::{event::EventManager, input::InputManager};

pub mod event;
pub mod input;

pub struct GameEngine<T: 'static> {
    /// Manages user input.
    pub input: InputManager,

    /// Taken as soon as the game
    /// begins rendering.
    _renderer: Option<RenderEngine<T>>,
    /// Taken as soon as the game
    /// begins rendering.
    _event_manager: Option<EventManager<T>>,
    
    /// Delta time.
    delta_time: Duration,
}
impl<V: 'static> GameEngine<V> {
    /// Create a new `GameEngine` with a window
    /// of size `window_size`
    pub fn new(window_size: impl Into<Size> + Copy) -> Self {
        let renderer = RenderEngine::new(window_size);
        let input = InputManager::default();
        let mut event_manager = EventManager::<V>::default();

        // Input event handler.
        event_manager.add_handler(|engine, _, _, event| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput { input, .. },
                    ..
                } => {
                    if let Some(code) = input.virtual_keycode {
                        engine.input.keyboard.process_event(input.state, code);
                    }
                }
                Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                    engine.input.mouse.last_offset_x = delta.0;
                    engine.input.mouse.last_offset_y = delta.1;
                    engine.input.mouse.update(delta.0, delta.1);
                }
                _ => ()
            }
        });

        Self {
            _renderer: Some(renderer),
            _event_manager: Some(event_manager),
            input,
            delta_time: Duration::new(0, 0)
        }
    }
    /// Add a renderer to the game engine pipeline.
    pub fn add_render_stage<T: RenderStage<V> + 'static>(&mut self, r: impl Fn(&mut GameWindow) -> T + 'static) -> Rc<RefCell<T>> {
        self._renderer.as_mut().unwrap().add_render_stage(r)
    }
    /// Add an event handler to the game engine pipeline.
    pub fn add_event_handler(&mut self, h: impl FnMut(&mut GameEngine<V>, &mut GameWindow, &mut V, &Event<()>) + 'static) {
        self._event_manager.as_mut().unwrap().add_handler(h);
    }
    /// Begin running the game engine.
    pub fn run(mut self, v: V) {
        let v = Rc::new(RefCell::new(v));
        let mut event_manager = self._event_manager.take().unwrap();
        self._renderer.take().unwrap().run(v.clone(), move |window, event| {
            self.delta_time = window.delta_time();
            event_manager.run(&mut self, window, &mut v.borrow_mut(), event);
        });
    }
    /// Get a handle to the renderer.
    /// Do not call after the game
    /// engine has started.
    pub fn renderer(&mut self) -> &mut RenderEngine<V> {
        self._renderer.as_mut().unwrap()
    } 

    /// Get the current delta time.
    pub fn delta_time(&self) -> Duration {
        self.delta_time
    }
}
