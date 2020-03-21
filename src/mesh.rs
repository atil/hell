extern crate tobj;
use crate::geom::Triangle;
use cgmath::*;

pub struct Mesh {
    pub triangles: Vec<Triangle>,
}

pub fn read_vertex_array(mesh: &tobj::Mesh) -> (Vec<f32>, Vec<u32>) {
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

impl Mesh {
    pub fn new(mesh: &tobj::Mesh) -> Mesh {
        let indices = mesh.indices.clone();
        let vertices = mesh
            .positions
            .chunks(3)
            .map(|slice| Point3::new(slice[0], slice[1], slice[2]))
            .collect::<Vec<Point3<f32>>>();

        let triangles = indices.chunks(3).fold(Vec::new(), |mut vec, next_three| {
            let i0: usize = next_three[0] as usize;
            let i1: usize = next_three[1] as usize;
            let i2: usize = next_three[2] as usize;

            vec.push(Triangle::new(vertices[i0], vertices[i1], vertices[i2]));

            vec
        });

        Mesh {
            triangles: triangles,
        }
    }
}
