use crate::material;
use crate::material::Material;
use crate::mesh;
use crate::mesh::Mesh;
use crate::render;
use std::path::Path;

pub fn load_obj(path: &str) -> (Vec<Mesh>, Vec<Material>) {
    let (tobj_models, tobj_mats) = match tobj::load_obj(&Path::new(path)) {
        Ok(cube_obj) => cube_obj,
        Err(e) => panic!("Error during loading models: {}", e),
    };

    assert_eq!(tobj_models.len(), tobj_mats.len());

    let mut materials = Vec::new();
    let mut meshes = Vec::new();
    for (tobj_model, tobj_mat) in tobj_models.iter().zip(tobj_mats) {
        meshes.push(mesh::Mesh::new(&tobj_model.mesh));

        let (vertex_data, index_data) = mesh::read_vertex_array(&tobj_model.mesh);
        materials.push(material::Material::new(
            vertex_data,
            index_data,
            tobj_mat,
            render::get_projection_matrix(),
        ));
    }

    (meshes, materials)
}
