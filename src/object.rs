use crate::light::DirectionalLight;
use crate::geom::Triangle;
use crate::material::Material;
use crate::mesh::Mesh;
use cgmath::*;

pub struct Object<'a> {
    // TODO: Static geometry won't have this as a variable
    // It'll be fed in via the ctor
    pub transform: Matrix4<f32>,
    material: &'a Material,
    pub mesh: &'a Mesh,

    // This is a list of transformed triangles
    // It's good to have this cached for static geometry
    pub triangles: Vec<Triangle>,
}

impl<'a> Object<'a> {
    pub fn new(mesh: &'a Mesh, material: &'a Material) -> Object<'a> {
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

    pub unsafe fn draw(&self, player_v: Matrix4<f32>, directional_light: &DirectionalLight) {
        self.material.draw(self.transform, player_v, directional_light);
    }
}
