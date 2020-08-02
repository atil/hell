use crate::geom::*;
use crate::static_object::StaticObject;
use cgmath::*;
use std::cmp::Ordering;

const PLAYER_HEIGHT: f32 = 1.0;
const PLAYER_CAPSULE_RADIUS: f32 = 0.5;

#[derive(Clone, Copy, Debug)]
struct PlayerShape {
    capsule0: Point3<f32>,
    capsule1: Point3<f32>,
    radius: f32,
    tip0: Point3<f32>,
    tip1: Point3<f32>,
}

impl PlayerShape {
    pub fn new(position: Point3<f32>, height: f32, radius: f32) -> PlayerShape {
        let half_height = height / 2.0;
        PlayerShape {
            capsule0: position + Vector3::new(0.0, half_height, 0.0),
            capsule1: position - Vector3::new(0.0, half_height, 0.0),
            radius: radius,
            tip0: position + Vector3::new(0.0, half_height + radius, 0.0),
            tip1: position - Vector3::new(0.0, half_height + radius, 0.0),
        }
    }

    pub fn displace(&mut self, displacement: Vector3<f32>) {
        self.capsule0 += displacement;
        self.capsule1 += displacement;
        self.tip0 += displacement;
        self.tip1 += displacement;
    }

    pub fn is_behind_triangle(&self, tri: &Triangle) -> bool {
        Vector3::dot(tri.normal, tri.p0 - self.capsule0) > 0.0
            && Vector3::dot(tri.normal, tri.p0 - self.capsule1) > 0.0
    }

    #[cfg(test)]
    pub fn with_capsule_points(c0: Point3<f32>, c1: Point3<f32>, radius: f32) -> PlayerShape {
        PlayerShape {
            capsule0: c0,
            capsule1: c1,
            radius: radius,
            tip0: c0 + Vector3::new(0.0, radius, 0.0),
            tip1: c1 - Vector3::new(0.0, radius, 0.0),
        }
    }
}

impl std::fmt::Display for PlayerShape {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "C0: [{:?}] C1: [{:?}] R:{:?}",
            self.capsule0, self.capsule1, self.radius
        )
    }
}

pub fn resolve_penetration(
    static_static_objects: &Vec<StaticObject>,
    player_pos: Point3<f32>,
) -> Vector3<f32> {
    let mut player_shape = PlayerShape::new(player_pos, PLAYER_HEIGHT, PLAYER_CAPSULE_RADIUS);
    let mut total_displacement = Vector3::zero();
    for obj in static_static_objects {
        // TODO #PERF: We can do this multithreaded
        // Technically, there _is_ an order which _might_ change the outcome of the calculation
        // But we don't rely on that. We might as well send each triangle to a different thread
        for tri in &obj.triangles {
            if let Some(mut penet) = compute_penetration(player_shape, *tri) {
                // Give an extra tiny push to the vertical displacement
                // If the capsule's bottom tip is perfectly aligned with the ground,
                // (for example the ground is y == 0.0 hand the player is as y == 1.0)
                // then it collides with ground triangles
                if penet.y.abs() > 0.0 {
                    penet.y += 0.0001 * penet.y.signum();
                }

                // If more than one triangles is penetrating the capsule
                // then don't compute penetrations from the same capsule position
                // Triangles poke at the capsule one by one
                // NOTE: This causes a sudden jump when walking over an edge
                player_shape.displace(penet);

                total_displacement += penet;
            }
        }
    }

    total_displacement
}

pub fn grounded_check(
    static_objects: &Vec<StaticObject>,
    player_pos: Point3<f32>,
    player_move_dir_horz: Option<Vector3<f32>>,
) -> (bool, Vector3<f32>) {
    let player_shape = PlayerShape::new(player_pos, PLAYER_HEIGHT, PLAYER_CAPSULE_RADIUS);

    const GROUNDED_HEIGHT: f32 = 0.51;
    const GHOST_RAY_OFFSET: f32 = PLAYER_CAPSULE_RADIUS - 0.01;

    let (velocity_dir, side_dir) = match player_move_dir_horz {
        Some(v) => (
            v,
            Quaternion::from_axis_angle(Vector3::unit_y(), Deg(90.0)).rotate_vector(v),
        ),
        None => (Vector3::unit_x(), Vector3::unit_z()),
    };

    let center = player_shape.capsule1;
    let ray_origins = vec![
        center + velocity_dir * GHOST_RAY_OFFSET,
        center - velocity_dir * GHOST_RAY_OFFSET,
        center + side_dir * GHOST_RAY_OFFSET,
        center - side_dir * GHOST_RAY_OFFSET,
    ];

    let ray_direction = -Vector3::unit_y();

    let mut hit_triangle = false;
    let mut ground_normal = Vector3::zero();
    'all: for obj in static_objects {
        for tri in &obj.triangles {
            // TODO #PERF: No need to run this loop if the velocity is zero
            for ray_slot in &ray_origins {
                if let Some(t) = ray_triangle_check(*ray_slot, ray_direction, *tri) {
                    if t < GROUNDED_HEIGHT {
                        hit_triangle = true;
                        ground_normal = tri.normal;
                        break 'all;
                    }
                }
            }
        }
    }

    (hit_triangle, ground_normal)
}

fn compute_penetration(player_shape: PlayerShape, triangle: Triangle) -> Option<Vector3<f32>> {
    if player_shape.is_behind_triangle(&triangle) {
        return None;
    }

    let dist1 = point_triangle_plane_distance(player_shape.capsule0, triangle);
    let dist2 = point_triangle_plane_distance(player_shape.capsule1, triangle);

    let (closer_point, closer_tip_point, closer_dist_to_plane) = match dist1.partial_cmp(&dist2) {
        Some(Ordering::Less) => (
            player_shape.capsule0,
            player_shape.capsule0 + Vector3::unit_y() * player_shape.radius,
            dist1,
        ),
        Some(Ordering::Greater) => (
            player_shape.capsule1,
            player_shape.capsule1 - Vector3::unit_y() * player_shape.radius,
            dist2,
        ),
        Some(Ordering::Equal) => {
            // Vertical triangle
            let mid_point = midpoint(player_shape.capsule0, player_shape.capsule1);

            (
                mid_point,
                mid_point - triangle.normal * player_shape.radius,
                dist1,
            )
        }
        None => panic!(format!(
            "Invalid capsule point comparison {} and {}",
            dist1, dist2
        )),
    };

    if closer_dist_to_plane > player_shape.radius {
        // Away from the triangle plane
        return None;
    }

    let point_on_plane = project_point_on_triangle_plane(closer_point, triangle);
    let point_on_plane_c0 = project_point_on_triangle_plane(player_shape.tip0, triangle);
    let point_on_plane_c1 = project_point_on_triangle_plane(player_shape.tip1, triangle);

    if is_point_in_triangle(point_on_plane, triangle)
        || is_point_in_triangle(point_on_plane_c0, triangle)
        || is_point_in_triangle(point_on_plane_c1, triangle)
    {
        // Projected point is in triangle
        // Since we're close to the plane, this case is a definite penetration
        let v = (closer_tip_point - point_on_plane).magnitude() * triangle.normal;
        Some(v)
    } else {
        let (closest_point_on_triangle, distance_to_triangle_on_plane) =
            get_closest_point_on_triangle(point_on_plane, triangle);

        let (_, distance_to_triangle_line_segment, is_on_line_segment) =
            get_closest_point_on_line_segment(
                closest_point_on_triangle,
                player_shape.capsule0,
                player_shape.capsule1,
            );

        let capsule_segment_distance_to_triangle = {
            if is_on_line_segment {
                // Direct distance
                distance_to_triangle_line_segment * distance_to_triangle_line_segment
            } else {
                let a = closer_dist_to_plane;
                let b = distance_to_triangle_on_plane;
                a * a + b * b // Pisagor
            }
        };

        if capsule_segment_distance_to_triangle >= player_shape.radius * player_shape.radius {
            return None;
        }

        let v = (point_on_plane - closest_point_on_triangle).normalize()
            * (player_shape.radius - distance_to_triangle_on_plane);
        Some(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve() {
        let player_shape = PlayerShape::with_capsule_points(
            Point3::new(0.0, 0.5, 0.0),
            Point3::new(0.0, -0.5, 0.0),
            0.5,
        );

        let tri = Triangle::new(
            Point3::new(1.0, -0.25, 0.0),
            Point3::new(0.0, -0.25, -1.0),
            Point3::new(-1.0, -0.25, 0.0),
        );

        assert_eq!(
            compute_penetration(player_shape, tri),
            Some(Vector3::new(0.0, 0.75, 0.0))
        );
    }

    #[test]
    fn test_resolve_2() {
        let player_shape = PlayerShape::with_capsule_points(
            Point3::new(0.0, 1.47, 0.0),
            Point3::new(0.0, 0.47, 0.0),
            0.5,
        );

        let tri = Triangle::new(
            Point3::new(29.0, 1.0, -29.0),
            Point3::new(-29.0, 1.0, -29.0),
            Point3::new(-29.0, 1.0, 29.0),
        );

        assert_eq!(
            compute_penetration(player_shape, tri),
            Some(Vector3::new(0.0, 0.97, 0.0))
        );
    }

    #[test]
    fn test_resolve_3() {
        let player_shape = PlayerShape::with_capsule_points(
            Point3::new(0.0, 1.92, 0.0),
            Point3::new(0.0, 0.92, 0.0),
            0.5,
        );

        let tri = Triangle::new(
            Point3::new(29.0, 1.0, -29.0),
            Point3::new(-29.0, 1.0, -29.0),
            Point3::new(-29.0, 1.0, 29.0),
        );

        assert_eq!(
            compute_penetration(player_shape, tri),
            Some(Vector3::new(0.0, 1.0 - 0.92 + 0.5, 0.0))
        );
    }

    #[test]
    fn test_resolve_4() {
        let player_shape = setup_player_shape(-9.58, 1.43, -20.92);

        let tri = Triangle::new(
            Point3::new(-10.0, 3.0, -30.0),
            Point3::new(-10.0, 3.0, -20.0),
            Point3::new(-10.0, 0.0, -20.0),
        );

        assert_eq!(
            compute_penetration(player_shape, tri),
            Some(Vector3::new(0.07999992, 0.0, 0.0))
        );
    }

    #[test]
    fn test_resolve_5() {
        let player_shape = setup_player_shape(-9.64, 1.47, -23.93);

        let tri = Triangle::new(
            Point3::new(-10.0, 3.0, -30.0),
            Point3::new(-10.0, 0.0, -20.0),
            Point3::new(-10.0, 0.0, -30.0),
        );

        assert_eq!(
            compute_penetration(player_shape, tri),
            Some(Vector3::new(0.14000034, 0.0, 0.0))
        );
    }

    #[test]
    fn test_resolve_6() {
        let player_shape = setup_player_shape(-9.7, 1.56, -23.49);

        let tri = Triangle::new(
            Point3::new(-10.0, 3.0, -30.0),
            Point3::new(-10.0, 0.0, -20.0),
            Point3::new(-10.0, 0.0, -30.0),
        );

        assert_eq!(
            compute_penetration(player_shape, tri),
            Some(Vector3::new(0.19999981, 0.0, 0.0))
        );
    }

    #[test]
    fn test_resolve_7() {
        let player_shape = setup_player_shape(19.51, 1.0, -2.0);

        let tri = Triangle::new(
            Point3::new(20.0, 1.0, -10.0),
            Point3::new(20.0, 1.0, 0.0),
            Point3::new(30.0, 1.0, -10.0),
        );

        assert_eq!(
            compute_penetration(player_shape, tri),
            Some(Vector3::new(-0.010000229, 0.0, 0.0))
        );
    }

    #[test]
    fn test_resolve_8() {
        let player_shape = setup_player_shape(9.5, 1.4914774, 5.7156773);

        let tri = Triangle::new(
            Point3::new(10.0, 1.0, 0.0),
            Point3::new(10.0, 0.0, 0.0),
            Point3::new(10.0, 1.0, 10.0),
        );

        assert_eq!(
            compute_penetration(player_shape, tri),
            None // Some(Vector3::new(0.0, 0.008522629, 0.0))
        );
    }

    #[test]
    fn test_resolve_no_collision() {
        let player_shape = setup_player_shape_at_zero();

        let tri = Triangle::new(
            Point3::new(10.0, 0.0, 0.0),
            Point3::new(9.0, 0.0, -1.0),
            Point3::new(8.0, 0.0, 0.0),
        );

        assert_eq!(compute_penetration(player_shape, tri), None);
    }

    #[test]
    fn test_resolve_no_collision_2() {
        let player_shape = setup_player_shape_at_zero();

        let tri = Triangle::new(
            Point3::new(1.0, 1.25, 0.0),
            Point3::new(0.0, 1.25, -1.0),
            Point3::new(-1.0, 1.25, 0.0),
        );

        assert_eq!(compute_penetration(player_shape, tri), None);
    }

    #[test]
    fn test_resolve_no_collision_3() {
        let player_shape = setup_player_shape(-3.848164, 6.019497, -30.203638);

        let tri = Triangle::new(
            Point3::new(0.0, 5.0, -30.0),
            Point3::new(-10.0, 5.0, -30.0),
            Point3::new(0.0, 0.0, -20.0),
        );

        assert_eq!(compute_penetration(player_shape, tri), None);
    }

    #[test]
    fn test_resolve_no_collision_with_floor() {
        let player_shape = setup_player_shape(-0.5, 1.01, -9.67);

        let tri = Triangle::new(
            Point3::new(-10.0, 0.0, -10.0),
            Point3::new(0.0, 0.0, -10.0),
            Point3::new(-10.0, 0.0, -20.0),
        );

        assert_eq!(compute_penetration(player_shape, tri), None);
    }

    fn setup_player_shape_at_zero() -> PlayerShape {
        PlayerShape::new(
            Point3::new(0.0, 0.0, 0.0),
            PLAYER_HEIGHT,
            PLAYER_CAPSULE_RADIUS,
        )
    }

    fn setup_player_shape(a: f32, b: f32, c: f32) -> PlayerShape {
        PlayerShape::new(Point3::new(a, b, c), PLAYER_HEIGHT, PLAYER_CAPSULE_RADIUS)
    }
}
