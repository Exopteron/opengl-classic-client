use std::{cell::RefCell, rc::Rc};

use self::{world::WorldRenderer, sky::SkyRenderer, text::TextRenderer};

use super::{engine::GameEngine, CubeGame};
mod world;
mod sky;
mod text;
pub struct RenderManager {
    pub sky: Rc<RefCell<SkyRenderer>>,
    pub world: Rc<RefCell<WorldRenderer>>,
    pub text: Rc<RefCell<TextRenderer>>,
}
impl RenderManager {
    pub fn new(engine: &mut GameEngine<CubeGame>) -> Self {
        Self {
            sky: engine.add_render_stage(SkyRenderer::init),
            world: engine.add_render_stage(WorldRenderer::init),
            text: engine.add_render_stage(TextRenderer::init),
        }
    }
}