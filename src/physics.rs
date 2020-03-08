use crate::object::Object;
use cgmath::*;

struct PlayerShape {
    capsule0: Vector3<f32>,
    capsule1: Vector3<f32>,
    radius: f32,
}

fn step(objects: &Vec<Object>, player: &PlayerShape) {
    let mut total_displacement = Vector3::<f32>::zero();
    for obj in objects {
        for tri in &obj.mesh.triangles {
            total_displacement += resolve(
                player.capsule0,
                player.capsule1,
                player.radius,
                tri.p0,
                tri.p1,
                tri.p2,
            );
        }
    }
}

fn resolve(
    c0: Vector3<f32>,
    c1: Vector3<f32>,
    cr: f32,
    t0: Vector3<f32>,
    t1: Vector3<f32>,
    t2: Vector3<f32>,
) -> Vector3<f32> {
    Vector3::zero()
}
