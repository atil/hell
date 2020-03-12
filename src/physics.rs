use crate::geom::*;
use crate::object::Object;
use cgmath::*;

#[derive(Clone, Copy)]
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
            total_displacement += resolve(player_shape, *tri);
        }
    }

    (total_displacement, is_grounded)
}

fn resolve(player_shape: PlayerShape, tri: Triangle) -> Vector3<f32> {
    let plane = get_plane(tri.p0, tri.normal);
    let dist1 = point_plane_distance(player_shape.capsule0, plane);
    let dist2 = point_plane_distance(player_shape.capsule1, plane);
    let (closer_point, dist_to_plane) = match dist1 < dist2 {
        true => (player_shape.capsule0, dist1),
        false => (player_shape.capsule1, dist2),
    };

    let point_on_plane = project_point_on_plane(closer_point, plane);
    let (penet, dir) = match is_point_in_triangle(point_on_plane, tri) {
        true => (dist_to_plane, tri.normal),
        false => todo!(),
    };
    Vector3::zero()
}
