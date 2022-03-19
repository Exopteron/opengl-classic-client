use std::{cell::{RefCell, RefMut}, rc::Rc, any::Any, sync::Arc};

use super::window::GameWindow;

/// A trait to store different stages of rendering.
/// (world, animals, GUI, etc.)
pub trait RenderStage<E> {
    fn run(&mut self, engine: &mut E, window: &mut GameWindow) -> anyhow::Result<()>;
}

/// A `dyn RenderStage` wrapped in an `Rc<RefCell<>>` to
/// allow mutable access from outside the rendering
/// engine.
pub struct RenderStageObj<E>(Rc<RefCell<dyn RenderStage<E>>>);
impl<E> RenderStageObj<E> {
    /// Creates a new `RenderStageObj`.
    pub fn new<T: RenderStage<E> + 'static>(s: T) -> (Self, Rc<RefCell<T>>) {
        let x = Rc::new(RefCell::new(s));
        (Self(x.clone()), x)
    }
    /// Creates a new handle to this rendering stage.
    pub fn new_handle(&self) -> Self {
        Self(self.0.clone())
    }
    /// Executes this stage.
    pub fn run(&self, engine: &mut E, window: &mut GameWindow) -> anyhow::Result<()> {
        self.0.borrow_mut().run(engine, window)
    }
}
