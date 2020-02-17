use crate::material::Material;
use cgmath::*;

pub struct Object<'a> {
    transform: Matrix4<f32>,
    material: &'a Material,
}

impl Object<'_> {
    pub fn new(material: &Material) -> Object {
        Object {
            transform: Matrix4::<f32>::identity(),
            material: material,
        }
    }

    pub fn translate(&mut self, displacement: Vector3<f32>) {
        self.transform = self.transform * Matrix4::from_translation(displacement);
    }

    pub fn rotate(&mut self, axis: Vector3<f32>, angle: f32) {
        self.transform = self.transform * Matrix4::from_axis_angle(axis, Deg(angle));
    }

    pub fn draw(&self, view_matrix: Matrix4<f32>) {
        self.material.draw(self.transform, view_matrix);
    }
}
