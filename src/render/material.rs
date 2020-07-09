extern crate tobj;
use crate::render::texture;
use crate::render::*;
use gl::types::*;

#[derive(Copy, Clone, Debug)]
struct Color {
    r: f32,
    g: f32,
    b: f32,
}

enum MaterialType {
    Texture(GLuint),
    Color(Color),
}

pub struct Material {
    vbo: BufferHandle,
    ibo: BufferHandle,
    vao: BufferHandle,
    m_type: MaterialType,
    index_data: Vec<u32>,
}

impl Material {
    pub fn new(vertex_data: Vec<f32>, index_data: Vec<u32>, tobj_mat: tobj::Material) -> Material {
        let (mat_type, _shader_path) = get_material_type(&tobj_mat);

        let mut vbo: GLuint = 0;
        let mut ibo: GLuint = 0;
        let mut vao: GLuint = 0;

        unsafe {
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ibo);
            gl::GenVertexArrays(1, &mut vao);

            const SIZEOF_FLOAT: usize = std::mem::size_of::<f32>();
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertex_data.len() * SIZEOF_FLOAT) as GLsizeiptr,
                vertex_data.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (index_data.len() * SIZEOF_FLOAT) as GLsizeiptr,
                index_data.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            // Position
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                (8 * SIZEOF_FLOAT) as i32,
                std::ptr::null(),
            );

            // Texcoord
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                (8 * SIZEOF_FLOAT) as i32,
                (3 * SIZEOF_FLOAT) as *const GLvoid,
            );

            // Normals
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(
                2,
                3,
                gl::FLOAT,
                gl::FALSE,
                (8 * SIZEOF_FLOAT) as i32,
                (5 * SIZEOF_FLOAT) as *const GLvoid,
            );

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);

            check_gl_error("material");
        }

        Material {
            vbo: vbo,
            ibo: ibo,
            vao: vao,
            m_type: mat_type,
            index_data: index_data,
        }
    }

    pub unsafe fn draw(&self) {
        match self.m_type {
            MaterialType::Texture(texture_handle) => {
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, texture_handle);
            }
            _ => unreachable!(),
        }

        gl::BindVertexArray(self.vao);
        gl::DrawElements(
            gl::TRIANGLES,
            self.index_data.len() as i32,
            gl::UNSIGNED_INT,
            self.index_data.as_ptr() as *const std::os::raw::c_void,
        );
    }
}

impl Drop for Material {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteBuffers(1, &self.ibo);
            if let MaterialType::Texture(texture) = self.m_type {
                gl::DeleteTextures(1, &texture);
            }
        }
    }
}

fn get_material_type(tobj_mat: &tobj::Material) -> (MaterialType, &str) {
    if !tobj_mat.diffuse_texture.is_empty() {
        let texture =
            texture::load_from_file(format!("assets/{}", tobj_mat.diffuse_texture).as_str());

        (MaterialType::Texture(texture), "src/shaders/triangle.glsl")
    } else {
        (
            MaterialType::Color(Color {
                r: tobj_mat.diffuse[0],
                g: tobj_mat.diffuse[1],
                b: tobj_mat.diffuse[2],
            }),
            "src/shaders/color.glsl",
        )
    }
}
