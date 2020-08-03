use crate::geom::Triangle;
use crate::mesh::Mesh;
use crate::render::material::Material;
use cgmath::*;

pub struct StaticObject<'a> {
    pub transform: Matrix4<f32>,
    pub material: &'a Material,
    pub mesh: &'a Mesh,
    pub triangles: Vec<Triangle>,
}

impl<'a> StaticObject<'a> {
    pub fn new(
        mesh: &'a Mesh,
        material: &'a Material,
        transform: Matrix4<f32>,
    ) -> StaticObject<'a> {
        let triangles = mesh
            .triangles
            .iter()
            .map(|tri| tri.transformed_by(transform))
            .collect::<Vec<Triangle>>();

        StaticObject {
            transform: transform,
            material: material,
            mesh: mesh,
            triangles: triangles,
        }
    }

    // pub fn translate(&mut self, displacement: Vector3<f32>) {
    //     self.transform = self.transform * Matrix4::from_translation(displacement);
    //     for (i, tri) in self.mesh.triangles.iter().enumerate() {
    //         self.triangles[i] = tri.transformed_by(self.transform);
    //     }
    // }

    // pub fn rotate(&mut self, axis: Vector3<f32>, angle: f32) {
    //     self.transform = self.transform * Matrix4::from_axis_angle(axis, Deg(angle));
    //     for (i, tri) in self.mesh.triangles.iter().enumerate() {
    //         self.triangles[i] = tri.transformed_by(self.transform);
    //     }
    // }
}
