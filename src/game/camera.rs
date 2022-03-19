use glam::{Mat4, Vec3, vec3, const_vec3};
const UP: Vec3 = const_vec3!([0., 1., 0.,]);

pub struct Camera {
    fov: f32,
    projection_matrix: Mat4,
    pub yaw: f32,
    pub pitch: f32,
    pub position: Vec3,
}
impl Camera {
    pub fn new(fov: f32, window_width: i32, window_height: i32) -> Self {
        Self {
            fov,
            projection_matrix: Mat4::perspective_lh(fov, window_width as f32 / window_height as f32, 0.1, 1000.),
            yaw: -90.,
            pitch: 0.,
            position: vec3(0., 0., 0.,)
        }
    }
    pub fn update_wh(&mut self, w: i32, h: i32) {
        self.projection_matrix = Mat4::perspective_lh(self.fov, w as f32 / h as f32, 0.1, 1000.);
    }
    pub fn direction(&self) -> Vec3 {
        vec3(
            self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            self.pitch.to_radians().sin(),
            self.yaw.to_radians().sin() * self.pitch.to_radians().cos()
        )
    }
    pub fn right(&self) -> Vec3 {
        let camera_front = self.direction().normalize();
        camera_front.cross(UP).normalize()
    }
    pub fn left(&self) -> Vec3 {
        -self.right()
    }
    pub fn matrix(&self) -> Mat4 {
        self.projection_matrix * self.view()
    }
    pub fn projection(&self) -> Mat4 {
        self.projection_matrix
    }
    pub fn view(&self) -> Mat4 {
        let camera_front = self.direction().normalize();
        Mat4::look_at_rh(self.position, self.position + camera_front, UP)
    }
    pub fn view_static(&self) -> Mat4 {
        let camera_front = self.direction().normalize();
        Mat4::look_at_rh(vec3(0., 0., 0.,), camera_front, UP)
    }
}