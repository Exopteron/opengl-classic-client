use std::{rc::Rc, cell::RefCell};

// Copyright (c) Exopteron 2022
use glam::{vec4, Vec4};
use glutin::{
    dpi::PhysicalSize,
    event::{Event, VirtualKeyCode},
};
use render::{opengl::{vao::VertexArrayObject, program::ShaderProgram, buffer::VertexBuffer}, stage::RenderStage};

use crate::{game::{engine::GameEngine, CubeGame}, render::stage::RenderStageObj, util::logging};

pub mod game;
pub mod render;
pub mod util;

pub struct TestRenderer {
    vao: VertexArrayObject,
    vbo: VertexBuffer<Vec4>,
    shaders: ShaderProgram,
}
impl TestRenderer {
    fn init(_window: &mut render::window::GameWindow) -> Self {
        unsafe {
            let vertices = &[
                vec4(-0.7, -0.7, 0., 1.),
                vec4(0.7, -0.7, 0., 1.),
                vec4(0., 0.7, 0., 1.,),
            ];
            let mut vbo = VertexBuffer::new(gl::STATIC_DRAW);
            vbo.set_data(vertices);
            Self {
                vbo,
                vao: VertexArrayObject::new(),
                shaders: ShaderProgram::new(&[
                    (gl::VERTEX_SHADER, Self::VERTEX_SHADER),
                    (gl::FRAGMENT_SHADER, Self::FRAGMENT_SHADER),
                ])
            }
        }
    }
}
impl RenderStage<CubeGame> for TestRenderer {
    fn run(&mut self, engine: &mut CubeGame, _window: &mut render::window::GameWindow) -> anyhow::Result<()> {
        unsafe {
            self.shaders.bind();
            self.vao.bind();
            gl::ClearColor(1., 1., 1., 0.);
            let _binding = self.vbo.bind(0);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }
        Ok(())
    }
}
impl TestRenderer {
    const VERTEX_SHADER: &'static str = r#"
    #version 440

layout(location = 0) in vec4 vertexPosition_modelspace;

void main() {
    gl_Position = vertexPosition_modelspace;
}
    "#;
    const FRAGMENT_SHADER: &'static str = r#"
    #version 420


out vec4 color;
void main() {
    color = vec4(1, 0, 0, 0);
}"#;
}
#[tokio::main]
async fn main() {
    logging::setup_logging();
    println!("Hello, world!");
    let mut engine = CubeGame::new(PhysicalSize::new(1024, 768)).await;
    engine.run();
}
