extern crate cgmath;
use cgmath::*;

pub struct Camera {
    pub position: Point3<f32>,
    pub forward: Point3<f32>,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            position: Point3::new(0.0, 0.0, 0.0),
            forward: Point3::new(0.0, 0.0, -1.0),
        }
    }

    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at(
            self.position,
            self.position + EuclideanSpace::to_vec(self.forward),
            vec3(0.0, 1.0, 0.0), // Always up
        )
    }
}
