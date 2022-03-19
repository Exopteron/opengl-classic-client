use std::time::Instant;

use glam::{vec4, Vec4, Mat4, Vec2};

use crate::{render::{stage::RenderStage, window::GameWindow, opengl::{vao::VertexArrayObject, buffer::VertexBuffer, program::ShaderProgram}}, game::{CubeGame, world::{World, FlatWorldGenerator, ChunkPosition, Block, BlockPosition}, mesh::MeshBuilder, texture::TerrainAtlas}};

pub struct WorldRenderer {
    vao: VertexArrayObject,
    vbo: VertexBuffer<Vec4>,
    texcoords: VertexBuffer<Vec2>,
    shaders: ShaderProgram,
    textures: TerrainAtlas,
    mesh: MeshBuilder,
    last_poll: Instant,
}
impl WorldRenderer {
    pub fn build_chunk(&mut self, world: World, pos: ChunkPosition) {
        let mut start = Instant::now();
        self.mesh.build_chunk(world, pos);
        //self.mesh.set_data(&mut self.vbo, &mut self.texcoords);
        let end = Instant::now();
        //log::info!("Took {}ms", (end - start).as_millis());
    }
    pub fn poll(&mut self) {
        if self.mesh.poll() {
            self.mesh.set_data(&mut self.vbo, &mut self.texcoords);
        }
    }
    pub fn init(w: &mut GameWindow) -> Self {
        unsafe {
            const SIZE: f32 = 2.0;
            // let mut world = World::new(FlatWorldGenerator::new(6, 1, 1, 1), 32, 16, 32);
            // let mut world = World::from_file("w.cw").unwrap();
            //world.set_block(Block::new(0, BlockPosition::new(0, 1, 0)));
            let mut mesh = MeshBuilder::default();
            // for x in 0..(world.length() >> 4) + 1 {
            //     for y in 0..(world.height() >> 4) + 1 {
            //         for z in 0..(world.width() >> 4) + 1 {
            //             mesh.build_chunk(&mut world, ChunkPosition::new(x, y, z));
            //         }
            //     }
            // }
            // mesh.build_chunk(&mut world, ChunkPosition::new(0, 0, 0));
            // mesh.build_chunk(&mut world, ChunkPosition::new(0, 0, 1));
            // mesh.build_chunk(&mut world, ChunkPosition::new(1, 0, 0));
            let mut vbo = VertexBuffer::new(gl::DYNAMIC_DRAW);
            let mut texcoords = VertexBuffer::new(gl::DYNAMIC_DRAW);
            // mesh.set_data(&mut vbo, &mut texcoords);
            Self {
                last_poll: Instant::now(),
                mesh,
                vbo,
                texcoords,
                vao: VertexArrayObject::new(),
                shaders: ShaderProgram::new(&[
                    (gl::VERTEX_SHADER, Self::VERTEX_SHADER),
                    (gl::FRAGMENT_SHADER, Self::FRAGMENT_SHADER),
                ]),
                textures: TerrainAtlas::load_from_file("terrain.png").unwrap()
            }
        }
    }
}
impl RenderStage<CubeGame> for WorldRenderer {
    fn run(&mut self, engine: &mut CubeGame, window: &mut crate::render::window::GameWindow) -> anyhow::Result<()> {
        unsafe {
            let now = Instant::now();
            if now.duration_since(self.last_poll).as_millis() > 25 {
                self.poll();
                self.last_poll = now;
            }
            self.shaders.bind();
            self.shaders.set_uniform("MVP", engine.camera.matrix());
            //self.shaders.set_uniform("terrainTexture", self.textures.texture.tex as i32);
            self.vao.bind();
            gl::ClearColor(1., 1., 1., 0.);
            let _binding = self.vbo.bind(0);
            let _binding2 = self.texcoords.bind(1);
            self.textures.texture.bind(gl::TEXTURE0);
            gl::DrawArrays(gl::TRIANGLES, 0, self.vbo.len() as i32);
        }
        Ok(())
    }
}

impl WorldRenderer {
    const VERTEX_SHADER: &'static str = r#"
    #version 440

    layout(location = 0) in vec4 vertexPosition_modelspace;
    layout(location = 1) in vec2 texcoords;
    uniform mat4 MVP;
    out vec2 fPos;
    out vec2 fTexCoords;
    void main() {
        fTexCoords = texcoords;
        fPos = vec2(vertexPosition_modelspace);
        gl_Position = MVP * vertexPosition_modelspace;
    }
    "#;
    const FRAGMENT_SHADER: &'static str = r#"
    #version 420

in vec2 fPos;
in vec2 fTexCoords;
out vec4 color;
uniform sampler2D terrainTexture;

void main() {
    color = texture(terrainTexture, fTexCoords);
}"#;
}