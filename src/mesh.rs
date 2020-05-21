extern crate tobj;
use crate::geom::Triangle;
use crate::math::*;
use cgmath::*;

pub struct Mesh {
    pub triangles: Vec<Triangle>,
}

pub fn read_vertex_array(mesh: &tobj::Mesh) -> (Vec<f32>, Vec<u32>) {
    let vertices = mesh.positions.clone();
    let texcoords = mesh.texcoords.clone();
    let iter_zip = vertices.chunks(3).zip(texcoords.chunks(2));

    // TODO: Shouldn't take the texcoord into consideration if there isn't a texture
    let vertex_data = iter_zip
        // ([v, v, v], [tx, tx])
        .map(|vec_tuple| {
            vec![
                (vec_tuple.0)[0],       // Position
                (vec_tuple.0)[1],       // Position
                (vec_tuple.0)[2],       // Position
                (vec_tuple.1)[0] / 5.0, // Texcoord
                (vec_tuple.1)[1] / 5.0, // Texcoord
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
            .map(|slice| {
                Point3::new(
                    slice_excess(slice[0]),
                    slice_excess(slice[1]),
                    slice_excess(slice[2]),
                )
            })
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
