use cgmath::*;
use sdl2::keyboard::*;

const MOVE_SPEED: f32 = 0.004;
const SENSITIVITY: f32 = 0.004;
const ACCELERATION: f32 = 0.005;
const MAX_SPEED_ON_ONE_DIMENSION: f32 = 3.0;
const FRICTION: f32 = 0.05;

pub struct Player {
    velocity: Vector3<f32>,
    position: Point3<f32>,
    forward: Point3<f32>,
}

impl Player {
    pub fn new() -> Player {
        Player {
            velocity: Vector3::<f32>::zero(),
            position: Point3::new(0.0, 0.0, 0.0),
            forward: Point3::new(0.0, 0.0, -1.0),
        }
    }

    pub fn tick(&mut self, dt: f32, keys: Vec<Keycode>, mouse: (f32, f32)) {
        Player::set_forward(&mut self.forward, mouse);

        let wish_dir = Player::get_wish_dir(&keys, self.forward);

        let is_grounded = true;
        if is_grounded {
            Player::accelerate(&mut self.velocity, wish_dir, dt);
            Player::apply_friction(&mut self.velocity, dt);
            self.velocity.y = 0.0;
        } else {
        }

        self.position += self.velocity * dt;
    }

    fn set_forward(forward: &mut Point3<f32>, mouse: (f32, f32)) {
        let (mouse_x, mouse_y) = mouse;

        let horz_rot = Quaternion::from_axis_angle(Vector3::unit_y(), Rad(-mouse_x) * SENSITIVITY);
        *forward = horz_rot.rotate_point(*forward);

        let left = EuclideanSpace::to_vec(*forward).cross(Vector3::unit_y());
        *forward =
            Quaternion::from_axis_angle(left, Rad(-mouse_y) * SENSITIVITY).rotate_point(*forward);
    }

    fn get_wish_dir(keys: &Vec<Keycode>, forward: Point3<f32>) -> Vector3<f32> {
        let mut vec = Vector3::<f32>::zero();
        if keys.contains(&Keycode::W) {
            vec += cgmath::EuclideanSpace::to_vec(forward)
        } else if keys.contains(&Keycode::S) {
            vec -= cgmath::EuclideanSpace::to_vec(forward)
        }
        if keys.contains(&Keycode::A) {
            vec -= cgmath::EuclideanSpace::to_vec(forward).cross(Vector3::unit_y())
        } else if keys.contains(&Keycode::D) {
            vec += cgmath::EuclideanSpace::to_vec(forward).cross(Vector3::unit_y())
        }

        vec
    }

    fn accelerate(velocity: &mut Vector3<f32>, wish_dir: Vector3<f32>, dt: f32) {
        let proj_speed = Vector3::dot(*velocity, wish_dir);
        let add_speed = MAX_SPEED_ON_ONE_DIMENSION - proj_speed;
        if add_speed < 0.0 {
            return;
        }

        let mut accel_amount = ACCELERATION * MAX_SPEED_ON_ONE_DIMENSION * dt;
        if accel_amount > add_speed {
            accel_amount = add_speed;
        }

        *velocity += wish_dir * accel_amount;
    }

    fn apply_friction(velocity: &mut Vector3<f32>, dt: f32) {
        let speed = velocity.magnitude();
        if speed > 0.0001 {
            let mut drop_amount = speed - (speed * FRICTION * dt);
            if drop_amount < 0.0 {
                drop_amount = 0.0;
            }
            *velocity *= drop_amount / speed;
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
