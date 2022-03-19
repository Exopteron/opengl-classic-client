// Copyright (c) Exopteron 2022
use gl::types::GLenum;
use glam::{Mat4, Vec3};

use super::{get_uniform_location, texture::Texture2D};

pub struct ShaderProgram {
    program: u32,
}
impl ShaderProgram {
    /// # Safety
    /// TODO
    pub unsafe fn new(shaders: &[(GLenum, &str)]) -> Self {
        let program = gl::CreateProgram();
        for (shader_type, source) in shaders {
            gl::AttachShader(program, super::compile_shader(source, *shader_type));
        }
        gl::LinkProgram(program);
        Self { program }
    }
    /// # Safety
    /// TODO
    pub unsafe fn bind(&self) {
        gl::UseProgram(self.program);
    }
    /// # Safety
    /// TODOs
    pub unsafe fn set_uniform(&self, name: &str, data: impl Uniformable) {
        let loc = get_uniform_location(self.program, name);
        data.bind_uniform(loc);
    }
}

pub trait Uniformable {
    /// # Safety
    /// TODO
    unsafe fn bind_uniform(&self, uniform_location: i32);
}
impl Uniformable for f32 {
    unsafe fn bind_uniform(&self, uniform_location: i32) {
        gl::Uniform1f(uniform_location, *self);
    }
}
impl Uniformable for Vec3 {
    unsafe fn bind_uniform(&self, uniform_location: i32) {
        gl::Uniform3f(uniform_location, self.x, self.y, self.z);
    }
}
impl Uniformable for i32 {
    unsafe fn bind_uniform(&self, uniform_location: i32) {
        gl::Uniform1i(uniform_location, *self);
    }
}
impl Uniformable for Mat4 {
    unsafe fn bind_uniform(&self, uniform_location: i32) {
        gl::UniformMatrix4fv(uniform_location, 1, gl::FALSE, &self.to_cols_array()[0]);
    }
}
impl Uniformable for Texture2D {
    unsafe fn bind_uniform(&self, uniform_location: i32) {
        gl::Uniform1ui(uniform_location, self.tex);
    }
}
