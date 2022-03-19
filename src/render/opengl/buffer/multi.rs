use std::{ops::Mul, mem, ptr};

use gl::types::GLenum;

use super::Bufferable;
struct MultibufferObject {
    offset: usize,
    len: usize,
    gl_type: GLenum,
    size_per_v: i32,
}
impl MultibufferObject {
    pub fn new(offset: usize, len: usize, gl_type: GLenum, size_per_v: i32) -> Self {
        Self { offset, len, gl_type, size_per_v, }
    }
}
pub struct MultiVertexBuffer {
    data: Vec<MultibufferObject>,
    buffer: u32
}
impl MultiVertexBuffer {
    pub unsafe fn new(in_data: &[Vec<impl Bufferable>]) -> Self {
        let mut buffer = 0;
        gl::GenBuffers(1, &mut buffer);

        gl::BindBuffer(gl::ARRAY_BUFFER, buffer);
        let mut data = Vec::new();
        let mut buffer_data = Vec::new();
        let mut offset = 0;
        for object in in_data {
            let mut bytes = object[0].get_data_self(object);
            let len = bytes.len();
            data.push(MultibufferObject::new(offset, len, object[0].data_type_self(), object[0].size_self()));
            buffer_data.append(&mut bytes);
            offset += len;
        }
        gl::BufferData(
            gl::ARRAY_BUFFER,
            buffer_data.len() as isize,
            mem::transmute(&buffer_data[0]),
            gl::STATIC_DRAW,
        );
        Self { data, buffer }
    }
    pub unsafe fn bind(&self) {
        gl::BindBuffer(gl::ARRAY_BUFFER, self.buffer);
        for (index, val) in self.data.iter().enumerate() {
            gl::EnableVertexAttribArray(index as u32);
            gl::VertexAttribPointer(index as u32, val.size_per_v, val.gl_type, gl::FALSE, 0, mem::transmute(val.offset));
        }
    }
}