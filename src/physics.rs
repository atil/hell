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

fn resolve(player_shape: PlayerShape, triangle: Triangle) -> Vector3<f32> {
    let dist1 = point_triangle_plane_distance(player_shape.capsule0, triangle);
    let dist2 = point_triangle_plane_distance(player_shape.capsule1, triangle);
    let (closer_point, closer_dist_to_plane) = match dist1 < dist2 {
        true => (player_shape.capsule0, dist1),
        false => (player_shape.capsule1, dist2),
    };

    if closer_dist_to_plane > player_shape.radius {
        return Vector3::<f32>::zero();
    }

    let point_on_plane = project_point_on_triangle_plane(closer_point, triangle);

    match is_point_in_triangle(point_on_plane, triangle) {
        true => closer_dist_to_plane * triangle.normal,
        false => {
            let (p1, d1) =
                get_closest_point_on_line_segment(point_on_plane, triangle.p0, triangle.p1);
            let (p2, d2) =
                get_closest_point_on_line_segment(point_on_plane, triangle.p1, triangle.p2);
            let (p3, d3) =
                get_closest_point_on_line_segment(point_on_plane, triangle.p2, triangle.p0);

            match d1.min(d2.min(d3)) {
                d1 => closer_point - p1,
                d2 => closer_point - p2,
                d3 => closer_point - p3,
            }
        }
    }
}
