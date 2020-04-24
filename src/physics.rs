use crate::geom::*;
use crate::object::Object;
use cgmath::*;
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug)]
struct PlayerShape {
    capsule0: Point3<f32>,
    capsule1: Point3<f32>,
    radius: f32,
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

pub fn step(objects: &Vec<Object>, player_pos: Point3<f32>) -> (Vector3<f32>, bool) {
    let player_shape = PlayerShape {
        capsule0: Point3::new(0.0, 0.5, 0.0) + EuclideanSpace::to_vec(player_pos),
        capsule1: Point3::new(0.0, -0.5, 0.0) + EuclideanSpace::to_vec(player_pos),
        radius: 0.5,
    };

    let mut total_displacement = Vector3::<f32>::zero();
    for obj in objects {
        for tri in &obj.triangles {
            let penet = compute_penetration(player_shape, *tri);
            total_displacement += penet;
        }
    }

    let is_grounded = spherecast(objects, player_shape);
    (total_displacement, is_grounded)
}

fn spherecast(objects: &Vec<Object>, player_shape: PlayerShape) -> bool {
    let p0 = player_shape.capsule0;
    let p1 = p0 - Vector3::unit_y() * 2.0;

    let mut hit_triangle = false;
    'all: for obj in objects {
        for tri in &obj.triangles {
            let dist = line_segment_triangle_distance(p0, p1, *tri);
            if dist < 0.1 {
                hit_triangle = true;
                break 'all;
            }
        }
    }

    hit_triangle
}

fn compute_penetration(player_shape: PlayerShape, triangle: Triangle) -> Vector3<f32> {
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
        return Vector3::<f32>::zero();
    }

    let point_on_plane = project_point_on_triangle_plane(closer_point, triangle);

    if is_point_in_triangle(point_on_plane, triangle) {
        // Projected point is in triangle
        // Since we're close to the plane, this case is a definite penetration
        (closer_tip_point - point_on_plane).magnitude() * triangle.normal
    } else {
        // Projected point isn't in triangle. Return the vector to the closest point on triangle
        let (p1, d1) = get_closest_point_on_line_segment(point_on_plane, triangle.p0, triangle.p1);
        let (p2, d2) = get_closest_point_on_line_segment(point_on_plane, triangle.p1, triangle.p2);
        let (p3, d3) = get_closest_point_on_line_segment(point_on_plane, triangle.p2, triangle.p0);

        let min_dist = d1.min(d2.min(d3));
        if min_dist > player_shape.radius {
            return Vector3::<f32>::zero();
        }

        // TODO: This is wrong? Displacement should be towards a point on the capsule, not to
        // the line segment. We should:
        // closerpoint + (closerpoint - px).normalize() * radius

        match min_dist {
            x if x == d1 => closer_point - p1,
            x if x == d2 => closer_point - p2,
            x if x == d3 => closer_point - p3,
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve() {
        let player_shape = setup_player_shape_at_zero();

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
        let player_shape = PlayerShape {
            capsule0: Point3::new(0.0, 1.47, 0.0),
            capsule1: Point3::new(0.0, 0.47, 0.0),
            radius: 0.5,
        };

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
        let player_shape = PlayerShape {
            capsule0: Point3::new(0.0, 1.92, 0.0),
            capsule1: Point3::new(0.0, 0.92, 0.0),
            radius: 0.5,
        };

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

    fn point(a: i32, b: i32, c: i32) -> Point3<f32> {
        Point3::new(a as f32, b as f32, c as f32)
    }

    fn setup_player_shape_at_zero() -> PlayerShape {
        setup_player_shape(0.0, 0.0, 0.0)
    }

    fn setup_player_shape(p0: f32, p1: f32, p2: f32) -> PlayerShape {
        let player_pos = Point3::new(p0, p1, p2);

        PlayerShape {
            capsule0: Point3::new(0.0, 0.5, 0.0) + EuclideanSpace::to_vec(player_pos),
            capsule1: Point3::new(0.0, -0.5, 0.0) + EuclideanSpace::to_vec(player_pos),
            radius: 0.5,
        }
    }
}
