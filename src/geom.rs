use cgmath::*;
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug)]
pub struct Triangle {
    pub p0: Point3<f32>,
    pub p1: Point3<f32>,
    pub p2: Point3<f32>,
    pub normal: Vector3<f32>,
}

impl Triangle {
    pub fn new(p0: Point3<f32>, p1: Point3<f32>, p2: Point3<f32>) -> Triangle {
        Triangle {
            p0: p0,
            p1: p1,
            p2: p2,
            normal: Vector3::cross(p1 - p0, p2 - p0).normalize(),
        }
    }
}

impl std::fmt::Display for Triangle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "P0: [{:?}] P1: [{:?}] P2:{:?}",
            self.p0, self.p1, self.p2
        )
    }
}

pub fn point_triangle_plane_distance(point: Point3<f32>, triangle: Triangle) -> f32 {
    Vector3::dot(point - triangle.p0, triangle.normal).abs()
}

pub fn project_point_on_triangle_plane(point: Point3<f32>, triangle: Triangle) -> Point3<f32> {
    let side = {
        if Vector3::dot(point - triangle.p0, triangle.normal) > 0.0 {
            -1.0 // The same side with the normal
        } else {
            1.0
        }
    };

    point + (point_triangle_plane_distance(point, triangle) * side * triangle.normal)
}

pub fn is_point_in_triangle(point: Point3<f32>, tri: Triangle) -> bool {
    if Vector3::dot(point - tri.p0, tri.normal).abs() > 0.0001 {
        panic!(format!(
            "attempted to perform point-triangle check on non-coplanar point-triangle\n {:?}\n {:?}\n {:?}",
            point, tri.p0, tri.normal
        ));
    }

    let d1 = Vector3::dot(tri.p1 - tri.p0, point - tri.p0);
    let d2 = Vector3::dot(tri.p2 - tri.p1, point - tri.p1);
    let d3 = Vector3::dot(tri.p0 - tri.p2, point - tri.p2);

    d1 > 0.0 && d2 > 0.0 && d3 > 0.0
}

pub fn get_closest_point_on_line_segment(
    point: Point3<f32>,
    p0: Point3<f32>,
    p1: Point3<f32>,
) -> (Point3<f32>, f32) {
    // TODO: Lots of sqrt here
    let line_segment_dir = (p1 - p0).normalize();
    let line_segment_length = (p1 - p0).magnitude();

    let dot = Vector3::dot(point - p0, line_segment_dir);
    if 0.0 < dot && dot < line_segment_length {
        // projection is on the line segment
        let point_on_line = p0 + dot * line_segment_dir;
        return (point_on_line, (point - point_on_line).magnitude());
    } else {
        // not on the segment, take the shorter one
        let dist1 = (point - p0).magnitude();
        let dist2 = (point - p1).magnitude();
        if dist1 < dist2 {
            return (p0, dist1);
        } else {
            return (p1, dist2);
        }
    }
}

pub fn line_segment_triangle_distance(p0: Point3<f32>, p1: Point3<f32>, triangle: Triangle) -> f32 {
    let dist1 = point_triangle_plane_distance(p0, triangle);
    let dist2 = point_triangle_plane_distance(p1, triangle);

    let closer_point = match dist1.partial_cmp(&dist2) {
        Some(Ordering::Less) => p0,
        Some(Ordering::Greater) => p1,
        Some(Ordering::Equal) => p0 + (p1 - p0) * 0.5,
        None => panic!(format!(
            "Invalid line-segment / point comparison {} and {}",
            dist1, dist2
        )),
    };

    let point_on_plane = project_point_on_triangle_plane(closer_point, triangle);
    match is_point_in_triangle(point_on_plane, triangle) {
        true => (closer_point - point_on_plane).magnitude(),
        false => {
            let (_, d1) =
                get_closest_point_on_line_segment(point_on_plane, triangle.p0, triangle.p1);
            let (_, d2) =
                get_closest_point_on_line_segment(point_on_plane, triangle.p1, triangle.p2);
            let (_, d3) =
                get_closest_point_on_line_segment(point_on_plane, triangle.p2, triangle.p0);

            d1.min(d2.min(d3))
        }
    }
}

pub fn midpoint(p0: Point3<f32>, p1: Point3<f32>) -> Point3<f32> {
    p0 + (p1 - p0) * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_point_in_triangle() {
        let tri = Triangle::new(
            Point3::new(0.5, -1.0, -1.0),
            Point3::new(0.5, -1.0, 1.0),
            Point3::new(0.5, 1.0, 0.0),
        );

        assert!(is_point_in_triangle(Point3::new(0.5, 0.0, 0.0), tri));
    }

    #[test]
    fn test_project_point_on_triangle_plane() {
        let p = Point3::new(0.0, -0.5, 0.0);
        let tri = Triangle::new(
            Point3::new(1.0, -0.25, 0.0),
            Point3::new(0.0, -0.25, -1.0),
            Point3::new(-1.0, -0.25, 0.0),
        );

        assert_eq!(
            project_point_on_triangle_plane(p, tri),
            Point3::new(0.0, -0.25, 0.0)
        );
    }

    #[test]
    fn test_project_point_on_triangle_plane_2() {
        let p = Point3::new(0.0, -1.49, 0.0);
        let tri = Triangle::new(
            Point3::new(-29.0, -1.0, 29.0),
            Point3::new(29.0, -1.0, -29.0),
            Point3::new(29.0, -1.0, 29.0),
        );
        assert_eq!(
            project_point_on_triangle_plane(p, tri),
            Point3::new(0.0, -1.0, 0.0)
        );
    }

    #[test]
    fn test_line_segment_triangle_distance_1() {
        let tri = Triangle::new(
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 0.0, -1.0),
            Point3::new(-1.0, 0.0, 0.0),
        );

        let p0 = Point3::new(0.0, 0.5, 0.0);
        let p1 = Point3::new(0.0, -0.5, 0.0);

        assert_eq!(line_segment_triangle_distance(p0, p1, tri), 0.0);
    }

    #[test]
    fn test_line_segment_triangle_distance_2() {
        let tri = Triangle::new(
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 0.0, -1.0),
            Point3::new(-1.0, 0.0, 0.0),
        );

        let p0 = Point3::new(0.0, 1.5, 0.0);
        let p1 = Point3::new(0.0, 0.5, 0.0);

        assert_eq!(line_segment_triangle_distance(p0, p1, tri), 0.5);
    }

    #[test]
    fn test_line_segment_triangle_distance_3() {
        let tri = Triangle::new(
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 0.0, -1.0),
            Point3::new(-1.0, 0.0, 0.0),
        );

        let p0 = Point3::new(3.0, 0.5, 0.0);
        let p1 = Point3::new(3.0, -0.5, 0.0);

        assert_eq!(line_segment_triangle_distance(p0, p1, tri), 2.0);
    }
}
