use crate::math::*;
use cgmath::*;

#[derive(Clone, Copy, Debug)]
pub struct Triangle {
    pub p0: Point3<f32>,
    pub p1: Point3<f32>,
    pub p2: Point3<f32>,
    pub normal: Vector3<f32>,
    area: f32,
}

impl Triangle {
    pub fn new(p0: Point3<f32>, p1: Point3<f32>, p2: Point3<f32>) -> Triangle {
        let c = Vector3::cross(p1 - p0, p2 - p0);
        Triangle {
            p0: p0,
            p1: p1,
            p2: p2,
            normal: c.normalize(),
            area: c.magnitude() / 2.0,
        }
    }

    pub fn transformed_by(&self, m: Matrix4<f32>) -> Triangle {
        let p0 = m.transform_point(self.p0);
        let p1 = m.transform_point(self.p1);
        let p2 = m.transform_point(self.p2);
        let c = Vector3::cross(self.p1 - self.p0, self.p2 - self.p0);

        Triangle {
            p0: p0,
            p1: p1,
            p2: p2,
            normal: c.normalize(),
            area: self.area,
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

fn point_triangle_plane_side(point: Point3<f32>, triangle: Triangle) -> bool {
    // True if they're on the same side with the normal
    Vector3::dot(point - triangle.p0, triangle.normal) > 0.0
}

pub fn project_point_on_triangle_plane(point: Point3<f32>, triangle: Triangle) -> Point3<f32> {
    let side = {
        if point_triangle_plane_side(point, triangle) {
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

    // Check barycentric coordinates of the point inside the triangle
    // https://math.stackexchange.com/a/28552
    let a = Vector3::cross(tri.p1 - point, tri.p2 - point).magnitude() / (2.0 * tri.area);
    let b = Vector3::cross(tri.p2 - point, tri.p0 - point).magnitude() / (2.0 * tri.area);
    let c = Vector3::cross(tri.p0 - point, tri.p1 - point).magnitude() / (2.0 * tri.area);

    in_between(a, 0.0, 1.0)
        && in_between(b, 0.0, 1.0)
        && in_between(c, 0.0, 1.0)
        && approx(a + b + c, 1.0)
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
        (point_on_line, (point - point_on_line).magnitude())
    } else {
        // not on the segment, take the shorter one
        let dist1 = (point - p0).magnitude();
        let dist2 = (point - p1).magnitude();
        if dist1 < dist2 {
            (p0, dist1)
        } else {
            (p1, dist2)
        }
    }
}

pub fn ray_triangle_check(
    ray_orig: Point3<f32>,
    ray_dir: Vector3<f32>,
    triangle: Triangle,
) -> Option<f32> {
    if approx(Vector3::dot(ray_dir, triangle.normal), 0.0) {
        return None; // Parallel
    }

    let t = Vector3::dot(triangle.p0 - ray_orig, triangle.normal)
        / Vector3::dot(ray_dir, triangle.normal);

    if t < 0.0 {
        return None; // Behind the ray
    }

    if is_point_in_triangle(ray_orig + t * ray_dir, triangle) {
        Some(t)
    } else {
        None
    }
}

pub fn get_closest_point_on_triangle(point: Point3<f32>, triangle: Triangle) -> (Point3<f32>, f32) {
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

pub fn midpoint(p0: Point3<f32>, p1: Point3<f32>) -> Point3<f32> {
    p0 + (p1 - p0) * 0.5
}

pub fn project_vector_on_plane(v: Vector3<f32>, n: Vector3<f32>) -> Vector3<f32> {
    let sqr_mag = Vector3::dot(n, n);
    if sqr_mag.is_nan() || sqr_mag < 0.0001 {
        return v;
    }
    let dot = Vector3::dot(v, n);

    let dot_over_sqr_mag = dot / sqr_mag;

    Vector3::new(
        v.x - n.x * dot_over_sqr_mag,
        v.y - n.y * dot_over_sqr_mag,
        v.z - n.z * dot_over_sqr_mag,
    )
}

pub fn horz_norm(v: &Vector3<f32>) -> Option<Vector3<f32>> {
    if v.magnitude2() < 0.00001 {
        None
    } else {
        Some(Vector3::new(v.x, 0.0, v.z).normalize())
    }
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
    fn test_is_point_in_triangle_2() {
        let tri = Triangle::new(
            Point3::new(-29.0, -1.0, -29.0),
            Point3::new(29.0, -1.0, 29.0),
            Point3::new(-29.0, -1.0, 29.0),
        );

        assert!(!is_point_in_triangle(Point3::new(-53.0, -1.0, 0.0), tri));
    }

    #[test]
    fn test_is_point_in_triangle_3() {
        let tri = Triangle::new(
            Point3::new(-29.0, 1.0, 29.0),
            Point3::new(29.0, 1.0, -29.0),
            Point3::new(29.0, 1.0, 29.0),
        );

        assert!(is_point_in_triangle(
            Point3::new(19.815203, 1.0, 0.4436245),
            tri
        ));
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
}
