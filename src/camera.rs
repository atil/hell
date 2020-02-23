extern crate cgmath;
use cgmath::*;
use sdl2::keyboard::*;

pub struct Camera {
    position: Point3<f32>,
    forward: Point3<f32>,
}

const MOVE_SPEED: f32 = 0.004;
const SENSITIVITY: f32 = 0.004;

impl Camera {
    pub fn new() -> Camera {
        Camera {
            position: Point3::new(0.0, 0.0, 0.0),
            forward: Point3::new(0.0, 0.0, -1.0),
        }
    }

    pub fn tick(&mut self, dt: f32, keys: Vec<Keycode>, mouse: (f32, f32)) {
        let (mouse_x, mouse_y) = mouse;

        let horz_rot = Quaternion::from_axis_angle(Vector3::unit_y(), Rad(-mouse_x) * SENSITIVITY);
        self.forward = horz_rot.rotate_point(self.forward);

        let left = EuclideanSpace::to_vec(self.forward).cross(Vector3::unit_y());
        self.forward = Quaternion::from_axis_angle(left, Rad(-mouse_y) * SENSITIVITY)
            .rotate_point(self.forward);

        if keys.contains(&Keycode::W) {
            self.position += cgmath::EuclideanSpace::to_vec(self.forward) * MOVE_SPEED * dt;
        } else if keys.contains(&Keycode::S) {
            self.position -= cgmath::EuclideanSpace::to_vec(self.forward) * MOVE_SPEED * dt;
        }
        if keys.contains(&Keycode::A) {
            let local_left = cgmath::EuclideanSpace::to_vec(self.forward).cross(Vector3::unit_y());
            self.position -= local_left * MOVE_SPEED * dt;
        } else if keys.contains(&Keycode::D) {
            let local_left = cgmath::EuclideanSpace::to_vec(self.forward).cross(Vector3::unit_y());
            self.position += local_left * MOVE_SPEED * dt;
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
