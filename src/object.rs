use crate::geom::Triangle;
use crate::material::Material;
use crate::mesh::Mesh;
use cgmath::*;

pub struct Object<'a> {
    pub transform: Matrix4<f32>,
    material: &'a Material,
    pub mesh: &'a Mesh,
    pub triangles: Vec<Triangle>,
}

impl<'a> Object<'a> {
    pub fn new(material: &'a Material, mesh: &'a Mesh) -> Object<'a> {
        Object {
            transform: Matrix4::<f32>::identity(),
            material: material,
            mesh: mesh,
            triangles: mesh.triangles.clone(),
        }
    }

    pub fn translate(&mut self, displacement: Vector3<f32>) {
        self.transform = self.transform * Matrix4::from_translation(displacement);

        for (i, tri) in self.mesh.triangles.iter().enumerate() {
            self.triangles[i] = tri.transformed_by(self.transform);
        }
    }

    pub fn rotate(&mut self, axis: Vector3<f32>, angle: f32) {
        self.transform = self.transform * Matrix4::from_axis_angle(axis, Deg(angle));

        for (i, tri) in self.mesh.triangles.iter().enumerate() {
            self.triangles[i] = tri.transformed_by(self.transform);
        }
    }

    pub unsafe fn draw(&self, view_matrix: Matrix4<f32>) {
        self.material.draw(self.transform, view_matrix);
    }
}
