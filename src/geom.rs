use cgmath::*;

#[derive(Clone, Copy)]
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

pub fn point_triangle_plane_distance(point: Point3<f32>, triangle: Triangle) -> f32 {
    Vector3::dot(point - triangle.p0, triangle.normal).abs()
}

pub fn project_point_on_triangle_plane(point: Point3<f32>, triangle: Triangle) -> Point3<f32> {
    point + (point_triangle_plane_distance(point, triangle) * -triangle.normal)
}

pub fn is_point_in_triangle(point: Point3<f32>, tri: Triangle) -> bool {
    if Vector3::dot(point - tri.p0, tri.normal) != 0.0 {
        panic!("attempted to perform point-triangle check on non-coplanar point-triangle");
    }

    let c1 = Vector3::cross(tri.p1 - tri.p0, point - tri.p0);
    let c2 = Vector3::cross(tri.p2 - tri.p1, point - tri.p1);
    let c3 = Vector3::cross(tri.p0 - tri.p2, point - tri.p2);

    Vector3::dot(c1.normalize(), tri.normal).abs() == 1.0
        && Vector3::dot(c2.normalize(), tri.normal).abs() == 1.0
        && Vector3::dot(c3.normalize(), tri.normal).abs() == 1.0
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_resolve() {
        let tri = Triangle::new(
            Point3::new(0.5, -1.0, -1.0),
            Point3::new(0.5, -1.0, 1.0),
            Point3::new(0.5, 1.0, 0.0),
        );

        assert!(is_point_in_triangle(Point3::new(0.5, 0.0, 0.0), tri));
    }
}
