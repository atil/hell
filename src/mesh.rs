extern crate tobj;

pub struct Mesh;

impl Mesh {
    pub fn read_vertex_data(mesh: &tobj::Mesh) -> (Vec<f32>, Vec<u32>) {
        let vertices = mesh.positions.clone();
        let texcoords = mesh.texcoords.clone();
        let normals = mesh.normals.clone();
        let iter_zip = vertices
            .chunks(3)
            .zip(texcoords.chunks(2))
            .zip(normals.chunks(3));
        let vertex_data = iter_zip
            // (([v, v, v], [tx, tx]), [n, n])
            .map(|vec_tuple| {
                vec![
                    (vec_tuple.0).0[0], // Position
                    (vec_tuple.0).0[1], // Position
                    (vec_tuple.0).0[2], // Position
                    (vec_tuple.0).1[0], // Texcoord
                    (vec_tuple.0).1[1], // Texcoord
                    (vec_tuple.1)[0],   // Normal
                    (vec_tuple.1)[1],   // Normal
                    (vec_tuple.1)[2],   // Normal
                ]
            })
            .flatten()
            .collect::<Vec<f32>>();
        let index_data = mesh.indices.clone();

        (vertex_data, index_data)
    }
}
