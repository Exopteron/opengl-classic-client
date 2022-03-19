use std::{rc::Rc, cell::RefCell};

use glutin::{dpi::Size, event::Event};

use self::{stage::{RenderStageObj, RenderStage}, window::GameWindow};

pub mod opengl;
pub mod stage;
pub mod window;

pub type StageInitializer<E> = Box<dyn Fn(&mut GameWindow) -> RenderStageObj<E>>;

/// The rendering engine. Contains the game
/// window and manages rendering stages.
pub struct RenderEngine<E: 'static> {
    pub window: GameWindow,
    stages: Vec<RenderStageObj<E>>,
}
impl<E> RenderEngine<E> {
    /// Creates a new `RenderEngine` with a
    /// window of size `s`.
    pub fn new(s: impl Into<Size> + Copy) -> Self {
        let window = GameWindow::new(s);
        Self {
            window,
            stages: Vec::new(),
        }
    }
    /// Adds a stage to the rendering pipeline.
    pub fn add_render_stage<T: RenderStage<E> + 'static>(&mut self, stage: impl Fn(&mut GameWindow) -> T + 'static) -> Rc<RefCell<T>> {
        let v = RenderStageObj::new(stage(&mut self.window));
        self.stages.push(v.0);
        v.1
    }

    /// Runs the rendering engine. All events
    /// handled by this will still be passed
    /// to `event_handler`, as well as
    /// any other event.
    pub fn run(mut self, engine: Rc<RefCell<E>>, mut event_handler: impl FnMut(&mut GameWindow, Event<()>) + 'static) {
        self.window.run(move |window, event| {
            if let Event::MainEventsCleared = event {
                window.context().clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                for stage in self.stages.iter() {
                    if let Err(e) = stage.run(&mut engine.borrow_mut(), window) {
                        log::error!("Error in rendering stage: {}", e); // TODO add stage name
                    }
                }
                if let Err(e) = window.context().swap_buffers() {
                    log::error!("Error swapping buffers: {:?}", e);
                }
            }
            event_handler(window, event);
        });
    }
}
