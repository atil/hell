use cgmath::*;

pub struct DirectionalLight {
    pub model: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub projection: Matrix4<f32>,
}

impl DirectionalLight {
    pub fn new() -> DirectionalLight {
        let model = Matrix4::from_translation(Vector3::new(100.0, 100.0, 0.0));
        let view = Matrix4::look_at(
            Point3::new(100.0, 100.0, 0.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::unit_y(),
        );

        let projection = cgmath::ortho(-10.0, 10.0, -10.0, 10.0, 0.1, 1000.0);

        DirectionalLight {
            model: model,
            view: view,
            projection: projection,
        }
    }
}
