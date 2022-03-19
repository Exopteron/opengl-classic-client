use std::mem;

use gl::types::GLenum;
use image::{DynamicImage, GenericImageView};

pub struct Texture2D {
    pub tex: u32,
}
impl Texture2D {
    pub unsafe fn new(handle: u32) -> Self {
        Self { tex: handle }
    }
    /// # Safety
    /// TODO
    pub unsafe fn from_image(img: DynamicImage) -> Self {
        let imgdata = img.as_bytes();
        let mut texture_id = 0;
        gl::GenTextures(1, &mut texture_id);
        gl::BindTexture(gl::TEXTURE_2D, texture_id);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            img.width() as i32,
            img.height() as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            mem::transmute(&imgdata[0]),
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        Self { tex: texture_id }
    }
    /// # Safety
    /// TODO
    pub unsafe fn bind(&self, loc: GLenum) {
        gl::ActiveTexture(loc);
        gl::BindTexture(gl::TEXTURE_2D, self.tex);
    }
}
