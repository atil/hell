use crate::object::Object;
use cgmath::*;

struct PlayerShape {
    capsule0: Point3<f32>,
    capsule1: Point3<f32>,
    radius: f32,
}

pub fn step(objects: &Vec<Object>, player_pos: Point3<f32>) -> (Vector3<f32>, bool) {
    let player_shape = PlayerShape {
        capsule0: Point3::new(0.0, 0.5, 0.0) + EuclideanSpace::to_vec(player_pos),
        capsule1: Point3::new(0.0, -0.5, 0.0) + EuclideanSpace::to_vec(player_pos),
        radius: 0.5,
    };

    let mut is_grounded = false;

    let mut total_displacement = Vector3::<f32>::zero();
    for obj in objects {
        for tri in &obj.mesh.triangles {
            total_displacement += resolve(
                player_shape.capsule0,
                player_shape.capsule1,
                player_shape.radius,
                tri.p0,
                tri.p1,
                tri.p2,
            );
        }
    }

    (total_displacement, is_grounded)
}

fn resolve(
    c0: Point3<f32>,
    c1: Point3<f32>,
    cr: f32,
    t0: Point3<f32>,
    t1: Point3<f32>,
    t2: Point3<f32>,
) -> Vector3<f32> {
    Vector3::zero()
}
