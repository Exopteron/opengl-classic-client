use std::{
    marker::PhantomData,
    mem::{self,},
    ptr,
};

use gl::types::GLenum;
use glam::{Vec2, Vec3, Vec4};
pub mod multi;

pub struct VertexBuffer<T: Bufferable> {
    buffer: u32,
    len: usize,
    t: GLenum,
    _a: PhantomData<T>,
}
impl<T: Bufferable> VertexBuffer<T> {
    /// # Safety
    /// TODO
    pub unsafe fn new(t: GLenum) -> Self {
        let mut buffer = 0;
        gl::GenBuffers(1, &mut buffer);

        gl::BindBuffer(gl::ARRAY_BUFFER, buffer);

        Self {
            t,
            buffer,
            _a: PhantomData,
            len: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    /// # Safety
    /// TODO
    pub unsafe fn set_data(&mut self, data: &[T]) {
        self.len = data.len();
        if self.len > 0 {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.buffer);
            let data = T::get_data(data);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                data.len() as isize,
                mem::transmute(&data[0]),
                self.t,
            );
        }
    }
    /// # Safety
    /// TODO
    pub unsafe fn bind(&mut self, index: u32) -> BufferBindHandle {
        gl::BindBuffer(gl::ARRAY_BUFFER, self.buffer);
        gl::EnableVertexAttribArray(index);
        gl::VertexAttribPointer(index, T::size(), T::data_type(), gl::FALSE, 0, ptr::null());
        BufferBindHandle::new(index)
    }
    /// # Safety
    /// TODO
    pub unsafe fn unbind(&mut self, index: u32) {
        gl::DisableVertexAttribArray(index);
    }
}
pub struct BufferBindHandle(u32);
impl BufferBindHandle {
    pub fn new(v: u32) -> Self {
        Self(v)
    }
}
impl Drop for BufferBindHandle {
    fn drop(&mut self) {
        unsafe {
            gl::DisableVertexAttribArray(self.0);
        }
    }
}
pub trait Bufferable: Sized {
    fn size() -> i32;
    fn data_type() -> GLenum;
    /// # Safety
    /// TODO
    unsafe fn get_data(arr: &[Self]) -> Vec<u8>;

    fn size_self(&self) -> i32 {
        Self::size()
    }
    fn data_type_self(&self) -> GLenum {
        Self::data_type()
    }
    /// # Safety
    /// TODO
    unsafe fn get_data_self(&self, arr: &[Self]) -> Vec<u8> {
        Self::get_data(arr)
    }
}
impl Bufferable for f32 {
    fn size() -> i32 {
        1
    }
    fn data_type() -> GLenum {
        gl::FLOAT
    }
    unsafe fn get_data(arr: &[f32]) -> Vec<u8> {
        let mut vec = Vec::new();
        for e in arr {
            vec.append(&mut e.to_le_bytes().to_vec());
        }
        // gl::BufferData(
        //     gl::ARRAY_BUFFER,
        //     (arr.len() * 4) as isize,
        //     mem::transmute(&arr[0]),
        //     gl::STATIC_DRAW,
        // );
        vec
    }
}
impl Bufferable for Vec2 {
    fn size() -> i32 {
        2
    }

    fn data_type() -> GLenum {
        gl::FLOAT
    }
    unsafe fn get_data(arr: &[Self]) -> Vec<u8> {
        let mut new_arr = Vec::new();
        for e in arr {
            new_arr.append(&mut e.x.to_le_bytes().to_vec());
            new_arr.append(&mut e.y.to_le_bytes().to_vec());
        }
        new_arr
    }
}
impl Bufferable for Vec3 {
    fn size() -> i32 {
        3
    }

    fn data_type() -> GLenum {
        gl::FLOAT
    }
    unsafe fn get_data(arr: &[Self]) -> Vec<u8> {
        let mut new_arr = Vec::new();
        for e in arr {
            new_arr.append(&mut e.x.to_le_bytes().to_vec());
            new_arr.append(&mut e.y.to_le_bytes().to_vec());
            new_arr.append(&mut e.z.to_le_bytes().to_vec());
        }
        new_arr
    }
}
impl Bufferable for Vec4 {
    fn size() -> i32 {
        4
    }

    fn data_type() -> GLenum {
        gl::FLOAT
    }
    unsafe fn get_data(arr: &[Self]) -> Vec<u8> {
        let mut new_arr = Vec::new();
        for e in arr {
            new_arr.append(&mut e.x.to_le_bytes().to_vec());
            new_arr.append(&mut e.y.to_le_bytes().to_vec());
            new_arr.append(&mut e.z.to_le_bytes().to_vec());
            new_arr.append(&mut e.w.to_le_bytes().to_vec());
        }
        new_arr
    }
}
impl<T: Bufferable> Drop for VertexBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.buffer);
        }
    }
}
