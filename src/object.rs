use crate::material::Material;
use crate::mesh::Mesh;
use cgmath::*;

pub struct Object<'a> {
    pub transform: Matrix4<f32>,
    material: &'a Material,
    pub mesh: &'a Mesh,
}

impl<'a> Object<'a> {
    pub fn new(material: &'a Material, mesh: &'a Mesh) -> Object<'a> {
        Object {
            transform: Matrix4::<f32>::identity(),
            material: material,
            mesh: mesh,
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
