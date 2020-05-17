extern crate tobj;
use crate::shader::*;
use crate::texture;
use cgmath::Matrix4;
use gl::types::*;

pub struct ColorRGB {
    r: f32,
    g: f32,
    b: f32,
}

impl ColorRGB {
    pub fn from_slice(slice: [f32; 3]) -> ColorRGB {
        ColorRGB {
            r: slice[0],
            g: slice[1],
            b: slice[2],
        }
    }
}

pub struct Material {
    vbo: GLuint,
    ibo: GLuint,
    vao: GLuint,
    diffuse: ColorRGB,
    texture: GLuint,
    program: Program,
    index_data: Vec<u32>,
}

impl Material {
    pub fn new(
        vertex_data: Vec<f32>,
        index_data: Vec<u32>,
        diffuse_color: ColorRGB,
        texture_name: &str,
        projection: Matrix4<f32>,
    ) -> Material {
        let shader_program =
            Program::from_shader("src/triangle.glsl").expect("Problem loading world shader");

        let mut vbo: GLuint = 0;
        let mut ibo: GLuint = 0;
        let mut vao: GLuint = 0;
        let texture: GLuint;

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
                (8 * SIZEOF_FLOAT) as GLsizei,
                std::ptr::null(),
            );

            // Texcoord
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                (8 * SIZEOF_FLOAT) as GLsizei,
                (3 * SIZEOF_FLOAT) as *const GLvoid,
            );

            // Normals
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(
                2,
                3,
                gl::FLOAT,
                gl::FALSE,
                (8 * SIZEOF_FLOAT) as GLsizei,
                (5 * SIZEOF_FLOAT) as *const GLvoid,
            );

            if !texture_name.is_empty() {
                texture = texture::load_from_file(format!("assets/{}", texture_name).as_str());
                shader_program.set_i32("texture0", texture as i32);
            } else {
                texture = 0;
            }

            // Unbinding
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);

            shader_program.set_matrix("projection", projection);
        }

        Material {
            vbo: vbo,
            ibo: ibo,
            vao: vao,
            diffuse: diffuse_color,
            texture: texture,
            program: shader_program,
            index_data: index_data,
        }
    }

    pub unsafe fn draw(&self, model: Matrix4<f32>, view: Matrix4<f32>) {
        self.program.set_used();
        self.program.set_matrix("model", model);
        self.program.set_matrix("view", view);

        gl::BindTexture(gl::TEXTURE_2D, self.texture);

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
            gl::DeleteTextures(1, &self.texture);
        }
    }
}
