use crate::mesh;
use crate::mesh::Mesh;
use crate::render::material;
use crate::render::material::Material;
use crate::static_object::StaticObject;
use cgmath::*;
use serde::*;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct PrefabEntry {
    name: String,
    asset_name: String,
}

#[derive(Serialize, Deserialize)]
struct Repository {
    prefab_entries: Vec<PrefabEntry>,
}

pub struct Prefab {
    name: String,
    meshes: Vec<Mesh>,
    materials: Vec<Material>,
}

#[derive(Serialize, Deserialize)]
struct StaticObjectEntry {
    name: String,
    prefab_name: String,
    position: [f32; 3],
}

#[derive(Serialize, Deserialize)]
struct Scene {
    static_object_entries: Vec<StaticObjectEntry>,
}

pub fn load_prefabs(path: &str) -> Vec<Prefab> {
    let json_string = fs::read_to_string(path).expect("Unable to read the prefabs file");
    let json_str = json_string.as_str();
    let repository: Repository = serde_json::from_str(json_str).expect("Repository error");
    let mut prefabs = Vec::new();
    for prefab_entry in repository.prefab_entries {
        let (meshes, materials) = load_obj(&prefab_entry.asset_name);
        let prefab = Prefab {
            name: prefab_entry.name,
            meshes: meshes,
            materials: materials,
        };
        prefabs.push(prefab);
    }

    prefabs
}

pub fn create_static_objects<'a>(path: &str, prefabs: &'a Vec<Prefab>) -> Vec<StaticObject<'a>> {
    let json_string = fs::read_to_string(path).expect("Unable to read the scene file");
    let json_str = json_string.as_str();

    let scene: Scene = serde_json::from_str(json_str).unwrap();

    let mut static_objects = Vec::new();
    for static_object_entry in scene.static_object_entries {
        let prefab = prefabs
            .iter()
            .find(|&p| p.name == static_object_entry.prefab_name)
            .expect("prefab couldn't be found");

        let pos = static_object_entry.position;

        for (mesh, material) in prefab.meshes.iter().zip(prefab.materials.iter()) {
            static_objects.push(StaticObject::new(
                &mesh,
                &material,
                Matrix4::from_translation(Vector3::new(pos[0], pos[1], pos[2])),
            ));
        }
    }

    static_objects
}

fn load_obj(path: &str) -> (Vec<Mesh>, Vec<Material>) {
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
        materials.push(material::Material::new(vertex_data, index_data, tobj_mat));
    }

    (meshes, materials)
}
