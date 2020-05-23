extern crate tobj;
use crate::render;
use crate::shader::*;
use crate::texture;
use cgmath::*;
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
    vbo: GLuint,
    ibo: GLuint,
    vao: GLuint,
    m_type: MaterialType,
    program: Program,
    index_data: Vec<u32>,
}

impl Material {
    pub fn new(
        vertex_data: Vec<f32>,
        index_data: Vec<u32>,
        tobj_mat: tobj::Material,
        projection: Matrix4<f32>,
    ) -> Material {
        let (mat_type, shader_path) = Material::get_material_type(&tobj_mat);
        let program = Program::from_shader(shader_path).expect("Problem loading world shader");

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

            render::check_gl_error("material");

            program.set_used();

            match mat_type {
                MaterialType::Texture(_) => {
                    // The uniform needs to be the texture unit, not the handle
                    program.set_i32("u_texture0", 0);

                    let d = Vector3::new(-1.0, -1.0, 0.0).normalize();
                    program.set_vec3("u_light_dir", d.x, d.y, d.z);
                }
                MaterialType::Color(color) => {
                    program.set_vec3("color0", color.r, color.g, color.b);
                }
            }

            program.set_matrix("u_projection", projection);
        }

        Material {
            vbo: vbo,
            ibo: ibo,
            vao: vao,
            m_type: mat_type,
            program: program,
            index_data: index_data,
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

    pub unsafe fn draw(&self, model: Matrix4<f32>, view: Matrix4<f32>) {
        self.program.set_used();

        // TODO #PERF: This can be done only once for all world materials
        self.program.set_matrix("u_model", model);
        self.program.set_matrix("u_view", view);
        self.program.set_i32("u_texture0", 0);

        match self.m_type {
            MaterialType::Texture(texture_handle) => {
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, texture_handle);
            }
            MaterialType::Color(c) => self.program.set_vec3("color0", c.r, c.g, c.b),
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
