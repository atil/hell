use cgmath::*;

pub struct DirectionalLight {
    pub position: Point3<f32>,
    pub view: Matrix4<f32>,
    pub projection: Matrix4<f32>,
    pub color: Vector4<f32>,
}

impl DirectionalLight {
    pub fn new() -> DirectionalLight {
        let s = 100.0;
        let pos = Point3::new(-100.0, 100.0, 20.0);
        DirectionalLight {
            position: pos,
            view: Matrix4::look_at(pos, Point3::new(0.0, 0.0, 0.0), Vector3::unit_y()),
            projection: cgmath::ortho(-s, s, -s, s, 0.1, 1000.0),
            color: Vector4::new(0.2, 0.1, 0.0, 1.0),
        }
    }
}
