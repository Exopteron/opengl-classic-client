use glam::{vec4, Vec4, Mat4, Mat3};

use crate::{render::{stage::RenderStage, window::GameWindow, opengl::{vao::VertexArrayObject, buffer::VertexBuffer, program::ShaderProgram}}, game::CubeGame};

pub struct SkyRenderer {
    vao: VertexArrayObject,
    vbo: VertexBuffer<Vec4>,
    shaders: ShaderProgram,
}
impl SkyRenderer {
    pub fn init(w: &mut GameWindow) -> Self {
        unsafe {
            let mut vertices = Vec::with_capacity(36);
            let mut i = 0;
            while i < SKYBOX_VERTICES.len() {
                vertices.push(vec4(SKYBOX_VERTICES[i], SKYBOX_VERTICES[i + 1], SKYBOX_VERTICES[i + 2], 1.));
                i += 3;
            }
            let mut vbo = VertexBuffer::new(gl::STATIC_DRAW);
            vbo.set_data(&vertices);
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
impl RenderStage<CubeGame> for SkyRenderer {
    fn run(&mut self, engine: &mut CubeGame, window: &mut crate::render::window::GameWindow) -> anyhow::Result<()> {
        unsafe {
            window.context().disable(gl::CULL_FACE);
            window.context().depth_mask(false);
            self.shaders.bind();
            self.shaders.set_uniform("MVP", engine.camera.projection() * engine.camera.view_static());
            self.vao.bind();
            let _binding = self.vbo.bind(0);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            window.context().depth_mask(true);
            window.context().enable(gl::CULL_FACE);
        }
        Ok(())
    }
}

impl SkyRenderer {
    const VERTEX_SHADER: &'static str = r#"
    #version 440

    layout(location = 0) in vec4 vertexPosition_modelspace;
    uniform mat4 MVP;
    out vec4 pos;
    void main() {
        pos = vertexPosition_modelspace;
        gl_Position = MVP * vertexPosition_modelspace;
    }
    "#;
    const FRAGMENT_SHADER: &'static str = r#"
    #version 420

in vec4 pos;
out vec4 color;
vec4 skytop = vec4(0.67, 0.8, 0.9, 0);
vec4 skyhorizon = vec4(137.0 / 256.0, 209.0 / 256.0, 254.0 / 256.0, 0.0f);
void main() {
    vec4 pointOnSphere = normalize(pos);
    float a = pointOnSphere.y * 3.0;
    color = mix(skyhorizon, skytop, a);
    // vec4 blue = vec4(0.67, 0.8, 0.9, 0);
    // vec4 white = vec4(1, 1, 1, 0);
    // color = mix(blue, white, pos.y);
}"#;
}

const SKYBOX_VERTICES: [f32; 108] = [
    // positions          
    -1.0f32,  1.0f32, -1.0f32,
    -1.0f32, -1.0f32, -1.0f32,
     1.0f32, -1.0f32, -1.0f32,
     1.0f32, -1.0f32, -1.0f32,
     1.0f32,  1.0f32, -1.0f32,
    -1.0f32,  1.0f32, -1.0f32,

    -1.0f32, -1.0f32,  1.0f32,
    -1.0f32, -1.0f32, -1.0f32,
    -1.0f32,  1.0f32, -1.0f32,
    -1.0f32,  1.0f32, -1.0f32,
    -1.0f32,  1.0f32,  1.0f32,
    -1.0f32, -1.0f32,  1.0f32,

     1.0f32, -1.0f32, -1.0f32,
     1.0f32, -1.0f32,  1.0f32,
     1.0f32,  1.0f32,  1.0f32,
     1.0f32,  1.0f32,  1.0f32,
     1.0f32,  1.0f32, -1.0f32,
     1.0f32, -1.0f32, -1.0f32,

    -1.0f32, -1.0f32,  1.0f32,
    -1.0f32,  1.0f32,  1.0f32,
     1.0f32,  1.0f32,  1.0f32,
     1.0f32,  1.0f32,  1.0f32,
     1.0f32, -1.0f32,  1.0f32,
    -1.0f32, -1.0f32,  1.0f32,

    -1.0f32,  1.0f32, -1.0f32,
     1.0f32,  1.0f32, -1.0f32,
     1.0f32,  1.0f32,  1.0f32,
     1.0f32,  1.0f32,  1.0f32,
    -1.0f32,  1.0f32,  1.0f32,
    -1.0f32,  1.0f32, -1.0f32,

    -1.0f32, -1.0f32, -1.0f32,
    -1.0f32, -1.0f32,  1.0f32,
     1.0f32, -1.0f32, -1.0f32,
     1.0f32, -1.0f32, -1.0f32,
    -1.0f32, -1.0f32,  1.0f32,
     1.0f32, -1.0f32,  1.0f32
];