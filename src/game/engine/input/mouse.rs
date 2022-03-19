/// Holds last offset of the mouse
/// cursor.
#[derive(Default)]
pub struct MouseInput {
    pub last_offset_x: f64,
    pub last_offset_y: f64,
    pub last_x: f64,
    pub last_y: f64,
    pub x: f64,
    pub y: f64,
    pub updated: bool
}
impl MouseInput {
    pub fn reset(&mut self) {
        self.last_offset_x = 0.;
        self.last_offset_y = 0.;
    }
    pub fn update(&mut self, x: f64, y: f64) {
        self.last_x = self.x;
        self.last_y = self.y;
        self.x += x;
        self.y += y;
        self.updated = true;
    }
}