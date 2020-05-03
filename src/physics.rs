use crate::geom::*;
use crate::object::Object;
use cgmath::*;
use std::cmp::Ordering;

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
        PlayerShape {
            capsule0: position + Vector3::new(0.0, height / 2.0, 0.0),
            capsule1: position - Vector3::new(0.0, height / 2.0, 0.0),
            radius: radius,
            tip0: position + Vector3::new(0.0, (height / 2.0) + radius, 0.0),
            tip1: position - Vector3::new(0.0, (height / 2.0) + radius, 0.0),
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

pub fn step(
    objects: &Vec<Object>,
    player_pos: Point3<f32>,
    player_forward: Point3<f32>,
) -> (Vector3<f32>, bool, Vector3<f32>) {
    let mut player_shape = PlayerShape::new(player_pos, 1.0, 0.5);

    let mut total_displacement = Vector3::zero();
    for obj in objects {
        for tri in &obj.triangles {
            let penet = compute_penetration(player_shape, *tri);
            if penet.magnitude2() > 0.001 {
                player_shape.displace(penet);
                total_displacement += penet;
            }
        }
    }

    let (is_grounded, ground_normal) = grounded_check(objects, player_shape, player_forward);

    (total_displacement, is_grounded, ground_normal)
}

fn grounded_check(
    objects: &Vec<Object>,
    player_shape: PlayerShape,
    player_forward: Point3<f32>,
) -> (bool, Vector3<f32>) {
    let ray_origin = player_shape.capsule1;
    let ray_direction = -Vector3::unit_y();
    let ghost_ray_origin = player_shape.capsule1 - EuclideanSpace::to_vec(player_forward);

    let mut hit_triangle = false;
    let mut ground_normal = Vector3::zero();
    'all: for obj in objects {
        for tri in &obj.triangles {
            if let Some(t) = ray_triangle_check(ray_origin, ray_direction, *tri) {
                if t < 0.9 {
                    hit_triangle = true;
                    ground_normal = tri.normal;
                    break 'all;
                }
            }

            if let Some(t) = ray_triangle_check(ghost_ray_origin, ray_direction, *tri) {
                if t < 0.9 {
                    hit_triangle = true;
                    ground_normal = tri.normal;
                    break 'all;
                }
            }
        }
    }

    (hit_triangle, ground_normal)
}

fn compute_penetration(player_shape: PlayerShape, triangle: Triangle) -> Vector3<f32> {
    if player_shape.is_behind_triangle(&triangle) {
        return Vector3::zero();
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
        return Vector3::zero();
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
        (closer_tip_point - point_on_plane).magnitude() * triangle.normal
    } else {
        let (p, distance_to_triangle) = get_closest_point_on_triangle(point_on_plane, triangle);

        if distance_to_triangle > player_shape.radius {
            return Vector3::<f32>::zero();
        }

        (point_on_plane - p).normalize() * (player_shape.radius - distance_to_triangle)
    }
}

fn get_closest_point_on_triangle(point: Point3<f32>, triangle: Triangle) -> (Point3<f32>, f32) {
    let (p1, d1) = get_closest_point_on_line_segment(point, triangle.p0, triangle.p1);
    let (p2, d2) = get_closest_point_on_line_segment(point, triangle.p1, triangle.p2);
    let (p3, d3) = get_closest_point_on_line_segment(point, triangle.p2, triangle.p0);

    let min_dist = d1.min(d2.min(d3));
    match min_dist {
        x if x == d1 => (p1, d1),
        x if x == d2 => (p2, d2),
        x if x == d3 => (p3, d3),
        _ => unreachable!(),
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
            Vector3::new(0.0, 0.75, 0.0)
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
            Vector3::new(0.0, 0.97, 0.0)
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
            Vector3::new(0.0, 1.0 - 0.92 + 0.5, 0.0)
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
            Vector3::new(0.07999992, 0.0, 0.0)
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
            Vector3::new(0.14000034, 0.0, 0.0)
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
            Vector3::new(0.19999981, 0.0, 0.0)
        );
    }

    #[test]
    fn test_resolve_7() {
        let player_shape = setup_player_shape(19.67636, 1.0, -2.3507338);

        let tri = Triangle::new(
            Point3::new(20.0, 1.0, -10.0),
            Point3::new(20.0, 1.0, 0.0),
            Point3::new(30.0, 1.0, -10.0),
        );

        assert_eq!(
            compute_penetration(player_shape, tri),
            Vector3::new(-0.17635918, 0.0, 0.0)
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

        assert_eq!(
            compute_penetration(player_shape, tri),
            Vector3::<f32>::zero()
        );
    }

    #[test]
    fn test_resolve_no_collision_2() {
        let player_shape = setup_player_shape_at_zero();

        let tri = Triangle::new(
            Point3::new(1.0, 1.25, 0.0),
            Point3::new(0.0, 1.25, -1.0),
            Point3::new(-1.0, 1.25, 0.0),
        );

        assert_eq!(
            compute_penetration(player_shape, tri),
            Vector3::new(0.0, 0.0, 0.0)
        );
    }

    fn setup_player_shape_at_zero() -> PlayerShape {
        PlayerShape::new(Point3::new(0.0, 0.0, 0.0), 1.0, 0.5)
    }

    fn setup_player_shape(a: f32, b: f32, c: f32) -> PlayerShape {
        PlayerShape::new(Point3::new(a, b, c), 1.0, 0.5)
    }
}
