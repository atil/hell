use crate::geom::*;
use crate::keys::Keys;
use crate::object::Object;
use crate::physics;
use cgmath::*;
use sdl2::keyboard::Keycode;

const SENSITIVITY: f32 = 0.004;
const GROUND_ACCELERATION: f32 = 0.005;
const GROUND_FRICTION: f32 = 0.05;
const AIR_ACCELERATION: f32 = 0.00001;
const AIR_DECELERATION: f32 = 0.00005;
const MAX_SPEED_ON_ONE_DIMENSION: f32 = 0.75;
const GRAVITY: f32 = 0.00003;
const JUMP_FORCE: f32 = 0.01;

pub struct Player {
    velocity: Vector3<f32>,
    position: Point3<f32>,
    forward: Vector3<f32>,
    is_grounded: bool,
    ground_normal: Vector3<f32>,
    gonna_jump: bool,
}

impl Player {
    pub fn new() -> Player {
        Player {
            velocity: Vector3::zero(),
            position: Point3::new(0.0, 100.0, -2.0),
            forward: Vector3::new(0.0, 0.0, -1.0),
            is_grounded: false,
            ground_normal: Vector3::zero(),
            gonna_jump: false,
        }
    }

    pub fn tick(
        &mut self,
        keys: &Keys,
        mouse: (f32, f32),
        collision_objects: &Vec<Object>,
        dt: f32,
    ) {
        mouse_look(&mut self.forward, mouse);
        if keys.get_key_down(Keycode::Space) {
            self.gonna_jump = true;
        } else if keys.get_key_up(Keycode::Space) {
            self.gonna_jump = false;
        }

        let wish_dir = get_wish_dir(
            &keys,
            horz_norm(&self.forward).unwrap_or(Vector3::<f32>::zero()),
        );

        if self.is_grounded {
            // Ground move
            accelerate(&mut self.velocity, wish_dir, GROUND_ACCELERATION, dt);
            apply_friction(&mut self.velocity, dt);

            self.velocity = project_vector_on_plane(self.velocity, self.ground_normal);

            if self.gonna_jump {
                // TODO: Add a fraction of horizontal velocity to the jump direction
                self.velocity += Vector3::unit_y() * JUMP_FORCE;
            }
        } else {
            // Air move
            let air_coeff = {
                if Vector3::dot(wish_dir, self.velocity) > 0.0 {
                    AIR_ACCELERATION
                } else {
                    AIR_DECELERATION
                }
            };

            accelerate(&mut self.velocity, wish_dir, air_coeff, dt);
            self.velocity -= Vector3::unit_y() * GRAVITY * dt;
        }

        self.position += self.velocity * dt;

        let (displacement, is_grounded, ground_normal) =
            physics::step(&collision_objects, self.position, &mut self.velocity);

        self.position += displacement;
        self.is_grounded = is_grounded;
        self.ground_normal = ground_normal;
    }

    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at(
            self.position,
            self.position + self.forward,
            vec3(0.0, 1.0, 0.0),
        )
    }
}

fn mouse_look(forward: &mut Vector3<f32>, mouse: (f32, f32)) {
    let (mouse_x, mouse_y) = mouse;

    let horz_rot = Quaternion::from_axis_angle(Vector3::unit_y(), Rad(-mouse_x) * SENSITIVITY);
    *forward = horz_rot.rotate_vector(*forward);

    let left = forward.cross(Vector3::unit_y());
    *forward =
        Quaternion::from_axis_angle(left, Rad(-mouse_y) * SENSITIVITY).rotate_vector(*forward);
}

fn get_wish_dir(keys: &Keys, forward: Vector3<f32>) -> Vector3<f32> {
    let mut vec = Vector3::<f32>::zero();
    if keys.get_key(Keycode::W) {
        vec += forward
    } else if keys.get_key(Keycode::S) {
        vec -= forward
    }
    if keys.get_key(Keycode::A) {
        vec -= forward.cross(Vector3::unit_y())
    } else if keys.get_key(Keycode::D) {
        vec += forward.cross(Vector3::unit_y())
    }

    if vec.magnitude2() > 0.00001 {
        vec.normalize()
    } else {
        vec // No input
    }
}

fn accelerate(velocity: &mut Vector3<f32>, wish_dir: Vector3<f32>, accel_coeff: f32, dt: f32) {
    let proj_speed = Vector3::dot(*velocity, wish_dir);
    let add_speed = MAX_SPEED_ON_ONE_DIMENSION - proj_speed;
    if add_speed < 0.0 {
        return;
    }

    let mut accel_amount = accel_coeff * MAX_SPEED_ON_ONE_DIMENSION * dt;
    if accel_amount > add_speed {
        accel_amount = add_speed;
    }

    *velocity += wish_dir * accel_amount;
}

fn apply_friction(velocity: &mut Vector3<f32>, dt: f32) {
    let speed = velocity.magnitude();

    if speed < 0.001 {
        *velocity = Vector3::zero();
        return;
    }

    let mut drop_amount = speed - (speed * GROUND_FRICTION * dt);
    if drop_amount < 0.0 {
        drop_amount = 0.0;
    }
    *velocity *= drop_amount / speed;
}
