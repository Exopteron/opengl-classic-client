use std::{mem, ffi::c_void};

use fnv::FnvHashMap;
use freetype::{Library, face::LoadFlag};
use glam::{vec4, Vec4, Mat4, Mat3, Vec2, IVec2, ivec2, Vec3, vec2, vec3};

use crate::{render::{stage::RenderStage, window::GameWindow, opengl::{vao::VertexArrayObject, buffer::VertexBuffer, program::ShaderProgram, texture::Texture2D}}, game::CubeGame};

struct Character {
    texture: Texture2D,
    size: IVec2,
    bearing: IVec2,
    advance: u32,
}
impl Character {
    pub fn new(texture: Texture2D, size: IVec2, bearing: IVec2, advance: u32) -> Self {
        Self { texture, size, bearing, advance }
    }
}

struct TextRenderRequest {
    pub text: String,
    pub position: Vec2,
    pub scale: f32,
    pub color: Vec3,
}

pub struct TextRenderer {
    requests: Vec<TextRenderRequest>,
    vao: VertexArrayObject,
    shaders: ShaderProgram,
    chars: FnvHashMap<char, Character>,
    vertices: VertexBuffer<Vec2>,
    texcoords: VertexBuffer<Vec2>,
}
impl TextRenderer {
    pub fn render(&mut self, string: String, position: Vec2, scale: f32, color: Vec3) {
        self.requests.push(TextRenderRequest { text: string, position, scale, color });
    }
    pub fn init(w: &mut GameWindow) -> Self {
        unsafe {
            let vao = VertexArrayObject::new();
            vao.bind();
            let shaders = ShaderProgram::new(&[
                (gl::VERTEX_SHADER, Self::VERTEX_SHADER),
                (gl::FRAGMENT_SHADER, Self::FRAGMENT_SHADER),
            ]);
            let lib = Library::init().unwrap();
            let face = lib.new_face("minecraft_font.ttf", 0).unwrap();
            face.set_pixel_sizes(0, 48).unwrap();
            let mut f = LoadFlag::empty();
            f.set(LoadFlag::RENDER, true);
            let mut characters = FnvHashMap::default();
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            for c in 0u8..128 {
                if let Err(e) = face.load_char(c as usize, f) {
                    log::error!("ERROR loading glyph: {}", e);
                    continue;
                }
                let mut texture = 0;
                gl::GenTextures(1, &mut texture);
                gl::BindTexture(gl::TEXTURE_2D, texture);
                let bmp = face.glyph().bitmap();
                let buf = bmp.buffer();
                gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RED as i32, bmp.width(), bmp.rows(), 0, gl::RED, gl::UNSIGNED_BYTE, buf.as_ptr() as *const c_void);
            
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            
                let character = Character::new(Texture2D::new(texture), ivec2(bmp.width(), bmp.rows()), ivec2(face.glyph().bitmap_left(), face.glyph().bitmap_top()), face.glyph().advance().x as u32);
                characters.insert(c as char, character);
            }
            let mut vertices = VertexBuffer::new(gl::DYNAMIC_DRAW);
            let mut texcoords = VertexBuffer::new(gl::DYNAMIC_DRAW);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 4);
            Self {
                requests: Vec::new(),
                vertices,
                texcoords,
                chars: characters,
                shaders,
                vao,
            }
        }
    }
}
impl RenderStage<CubeGame> for TextRenderer {
    fn run(&mut self, engine: &mut CubeGame, window: &mut crate::render::window::GameWindow) -> anyhow::Result<()> {
        unsafe {
            self.vao.bind();
            self.shaders.bind();
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Disable(gl::DEPTH_TEST);
            let projection = Mat4::orthographic_lh(0.0, window.size().width as f32, 0.0, window.size().height as f32, 0.5, 1.5);
            self.shaders.set_uniform("projection", projection);
            for request in self.requests.drain(..).collect::<Vec<TextRenderRequest>>() {
                self.render_text(engine, request.text, request.position.x, request.position.y, request.scale, request.color);
            }
            gl::Enable(gl::DEPTH_TEST);
            gl::Disable(gl::BLEND);
        }
        Ok(())
    }
}
impl TextRenderer {
    unsafe fn render_text(&mut self, engine: &mut CubeGame, text: String, mut x: f32, mut y: f32, scale: f32, color: Vec3) {
        self.shaders.bind();
        self.shaders.set_uniform("textColor", color);
        for c in text.chars() {
            if let Some(ch) = self.chars.get(&c) {
                let xpos = x + ch.bearing.x as f32 * scale;
                let ypos = y - (ch.size.y - ch.bearing.y) as f32 * scale;

                let w = ch.size.x as f32 * scale;
                let h = ch.size.y as f32 * scale;

                let vertices = &[
                    vec2(xpos, ypos + h),
                    vec2(xpos, ypos),
                    vec2(xpos + w, ypos),

                    vec2(xpos, ypos + h),
                    vec2(xpos + w, ypos),
                    vec2(xpos + w, ypos + h),
                ];

                let texcoords = &[
                    vec2(0.0, 0.0),
                    vec2(0.0, 1.0),
                    vec2(1.0, 1.0,),

                    vec2(0.0, 0.0),
                    vec2(1.0, 1.0),
                    vec2(1.0, 0.0),
                ];

                self.vertices.set_data(vertices);
                self.texcoords.set_data(texcoords);

                ch.texture.bind(gl::TEXTURE0);

                let _b1 = self.vertices.bind(0);
                let _b2 = self.texcoords.bind(1);

                gl::DrawArrays(gl::TRIANGLES, 0, 6);
                x += (ch.advance >> 6) as f32 * scale;
            }
        }
    }
}
impl TextRenderer {
    const VERTEX_SHADER: &'static str = r#"
    #version 440

    layout(location = 0) in vec2 vertex;
    layout(location = 1) in vec2 texcoords;

    out vec2 TexCoords;

    uniform mat4 projection;

    void main() {
        gl_Position = projection * vec4(vertex, 0.0, 1.0);
        TexCoords = texcoords;
    }
    "#;
    const FRAGMENT_SHADER: &'static str = r#"
    #version 420
    out vec4 color;
    
    in vec2 TexCoords;
    
    uniform sampler2D text;
    uniform vec3 textColor;
    
    void main() {
        vec4 sampled = vec4(1.0, 1.0, 1.0, texture(text, TexCoords).r);
        color = vec4(textColor, 1.0) * sampled;
    }"#;
}