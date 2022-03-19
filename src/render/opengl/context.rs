use std::{ptr, ffi::c_void, mem, pin::Pin};

use anyhow::bail;
use gl::types::{GLenum, GLuint, GLsizei, GLchar};
use glutin::{ContextWrapper, PossiblyCurrent, window::Window};



/// The OpenGL context.
pub struct GlContext {
    context: Option<ContextWrapper<PossiblyCurrent, Window>>,
    error_handler: Option<Pin<Box<FnPointer>>>,
}
struct FnPointer(Box<dyn Fn(GLenum, GLenum, GLuint, GLenum, String)>); 
impl GlContext {
    pub fn new(context: ContextWrapper<PossiblyCurrent, Window>) -> Self {
        Self { context: Some(context), error_handler: None }
    }
    pub fn set_error_handler(&mut self, handler: impl Fn(GLenum, GLenum, GLuint, GLenum, String) + 'static) {
        let v = Box::new(handler);
        let v = Box::pin(FnPointer(v));
        unsafe {
            gl::DebugMessageCallback(Some(error_callback), (&*v as *const FnPointer) as *mut c_void);
        }
        self.error_handler = Some(v);
    }
    pub fn get_context(&mut self) -> &ContextWrapper<PossiblyCurrent, Window> {
        self.context.as_ref().unwrap()
    }
    /// Make this context the current context.
    pub fn make_current(&mut self) -> anyhow::Result<()> {
        if !self.get_context().is_current() {
            unsafe {
                let v = self.context.take().unwrap();
                let v = match v.make_current() {
                    Ok(c) => c,
                    Err((_, e)) => bail!("{}", e)
                };
                self.context = Some(v)
            }
        }
        Ok(())
    }
    pub fn swap_buffers(&mut self) -> anyhow::Result<()> {
        self.get_context().swap_buffers()?;
        Ok(())
    }
    pub fn clear(&mut self, mask: u32) {
        unsafe {
            gl::Clear(mask);
        }
    }
    pub fn enable(&mut self, cap: u32) {
        unsafe {
            gl::Enable(cap);
        }
    }
    pub fn disable(&mut self, cap: u32) {
        unsafe {
            gl::Disable(cap);
        }
    }
    pub fn depth_mask(&mut self, state: bool) {
        let s = match state {
            true => gl::TRUE,
            false => gl::FALSE,
        };
        unsafe {
            gl::DepthMask(s);
        }
    }
}

unsafe fn read_gl_string(length: GLsizei, message: *const GLchar) -> String {
    let mut string = String::new();
    for i in 0..length {
        string.push((*message.offset(i as isize) as u8) as char);
    }
    string
}

extern "system" fn error_callback(
    _source: GLenum,
    _err_type: GLenum,
    _id: GLuint,
    severity: GLenum,
    length: GLsizei,
    message: *const GLchar,
    _user_param: *mut c_void,
) {
    unsafe {
        let v: *const FnPointer = _user_param as *const FnPointer;
        let string = read_gl_string(length, message);
        (*(*v).0)(_source, _err_type, _id, severity, string);
    }
}
