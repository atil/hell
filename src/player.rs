use crate::geom::*;
use crate::keys::Keys;
use crate::object::Object;
use crate::physics::*;
use crate::ui::Ui;
use cgmath::*;
use sdl2::keyboard::Keycode;

const SENSITIVITY: f32 = 0.004;
const GROUND_ACCELERATION: f32 = 0.003;
const GROUND_FRICTION: f32 = 0.02;
const GROUND_FRICTION_LOWER_LIMIT: f32 = 0.001; // Stop if the speed is lower than this
const AIR_ACCELERATION: f32 = 0.00001;
const AIR_DECELERATION: f32 = 0.00005;
const MAX_SPEED_ON_ONE_DIMENSION: f32 = 0.075;
const GRAVITY: f32 = 0.00003;
const JUMP_FORCE: f32 = 0.01;
const START_POSITION: Point3<f32> = Point3::new(0.0, 20.0, -2.0);

pub struct Player {
    velocity: Vector3<f32>,
    position: Point3<f32>,
    forward: Vector3<f32>,
    prev_is_grounded: bool,
    gonna_jump: bool,
}

impl Player {
    pub fn new() -> Player {
        Player {
            velocity: Vector3::zero(),
            position: START_POSITION,
            forward: Vector3::new(0.0, 0.0, -1.0),
            prev_is_grounded: false,
            gonna_jump: false,
        }
    }

    pub fn tick(
        &mut self,
        keys: &Keys,
        mouse: (f32, f32),
        collision_objects: &Vec<Object>,
        ui: &mut Ui,
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

        let (is_grounded, ground_normal) =
            grounded_check(&collision_objects, self.position, horz_norm(&self.velocity));

        if is_grounded {
            // Ground move
            if self.prev_is_grounded && !self.gonna_jump {
                apply_friction(&mut self.velocity, dt);
            }

            accelerate(&mut self.velocity, wish_dir, GROUND_ACCELERATION, dt);

            // No vetical velocity while grounded
            self.velocity = project_vector_on_plane(self.velocity, ground_normal);

            if self.gonna_jump {
                self.gonna_jump = false;

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

        let displacement = resolve_penetration(&collision_objects, self.position);
        self.position += displacement;

        self.velocity = project_vector_on_plane(self.velocity, displacement.normalize());

        self.prev_is_grounded = is_grounded;

        if self.position.y < -30.0 {
            // Fell down, reset
            self.velocity = Vector3::zero();
            self.position = START_POSITION;
        }

        let velocity_string = format!("{:.3}", self.velocity.magnitude());
        // println!("{:?} {:?} {}", self.position, displacement, is_grounded);
        ui.draw_text(velocity_string.as_str());
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

    let down_limit = speed.max(GROUND_FRICTION_LOWER_LIMIT);
    let mut drop_amount = speed - (down_limit * GROUND_FRICTION * dt);
    if drop_amount < 0.0 {
        drop_amount = 0.0;
    }
    *velocity *= drop_amount / speed;
}
