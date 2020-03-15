use cgmath::*;

#[derive(Clone, Copy)]
pub struct Triangle {
    pub p0: Point3<f32>,
    pub p1: Point3<f32>,
    pub p2: Point3<f32>,
    pub normal: Vector3<f32>,
}

pub fn point_triangle_plane_distance(point: Point3<f32>, triangle: Triangle) -> f32 {
    Vector3::dot(point - triangle.p0, triangle.normal)
}

pub fn project_point_on_triangle_plane(point: Point3<f32>, triangle: Triangle) -> Point3<f32> {
    point + (point_triangle_plane_distance(point, triangle) * -triangle.normal)
}

pub fn is_point_in_triangle(point: Point3<f32>, tri: Triangle) -> bool {
    let c1 = Vector3::cross(point - tri.p0, tri.p1 - tri.p0);
    let c2 = Vector3::cross(point - tri.p1, tri.p2 - tri.p1);
    let c3 = Vector3::cross(point - tri.p2, tri.p0 - tri.p2);

    c1.y > 0.0 && c2.y > 0.0 && c3.y > 0.0
}

pub fn point_line_segment_distance(point: Point3<f32>, p0: Point3<f32>, p1: Point3<f32>) -> f32 {
    // TODO: Lots of sqrt here
    let dot = Vector3::dot(point - p0, p1 - p0);
    let line_segment_length = (p1 - p0).magnitude();
    if 0.0 < dot && dot < line_segment_length {
        // on the line segment
        let line_segment_dir = (p1 - p0).normalize();
        let point_on_line = p0 + dot * line_segment_dir;
        return (point - point_on_line).magnitude();
    } else {
        // not on the segment, take the shorter one
        let dist1 = (point - p0).magnitude();
        let dist2 = (point - p1).magnitude();
        if dist1 < dist2 {
            return dist1;
        } else {
            return dist2;
        }
    }
}
