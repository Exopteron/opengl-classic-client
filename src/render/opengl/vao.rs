pub struct VertexArrayObject {
    vao: u32,
}
impl VertexArrayObject {
    /// # Safety
    /// TODO
    pub unsafe fn new() -> Self {
        let mut vao = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
        Self { vao }
    }
    /// # Safety
    /// TODO
    pub unsafe fn bind(&self) {
        gl::BindVertexArray(self.vao);
    }
}
impl Drop for VertexArrayObject {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}
