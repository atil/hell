use cgmath::*;

#[derive(Clone, Copy)]
pub struct Triangle {
    pub p0: Point3<f32>,
    pub p1: Point3<f32>,
    pub p2: Point3<f32>,
    pub normal: Vector3<f32>,
}

#[derive(Clone, Copy)]
pub struct Plane {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
}

pub fn get_plane(point: Point3<f32>, normal: Vector3<f32>) -> Plane {
    Plane {
        a: normal.x,
        b: normal.y,
        c: normal.z,
        d: Vector3::dot(EuclideanSpace::to_vec(point), normal),
    }
}

pub fn point_plane_distance(point: Point3<f32>, plane: Plane) -> f32 {
    (plane.a * point.x + plane.b * point.y + plane.c * point.z - plane.d).abs()
        / (plane.a * plane.a + plane.b * plane.b + plane.c * plane.c).sqrt()
}

pub fn point_triangle_distance(point: Point3<f32>, triangle: Triangle) -> f32 {
    let plane = get_plane(triangle.p0, triangle.normal);
    point_plane_distance(point, plane)
}

pub fn project_point_on_plane(point: Point3<f32>, plane: Plane) -> Point3<f32> {
    todo!()
}

pub fn is_point_in_triangle(point: Point3<f32>, tri: Triangle) -> bool {
    todo!()
}
