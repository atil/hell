use crate::cgmath::SquareMatrix;
use crate::shader::*;
use cgmath::Matrix4;
use gl::types::*;
use std::ffi::CString;

pub struct Ui {
    vbo: GLuint,
    ibo: GLuint,
    vao: GLuint,
    program: Program,
    index_data: Vec<u32>,
}

impl Ui {
    pub fn init() -> Self {
        let vert_shader =
            Shader::from_vert_source(&CString::new(include_str!("ui.vert")).unwrap()).unwrap();

        let frag_shader =
            Shader::from_frag_source(&CString::new(include_str!("ui.frag")).unwrap()).unwrap();

        let shader_program = Program::from_shaders(&[vert_shader, frag_shader]).unwrap();

        let vertex_data: Vec<f32> = vec![-0.5, -0.5, 0.0, 0.5, -0.5, 0.0, 0.0, 0.5, 0.0];
        let index_data = vec![0, 1, 2];

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
                (3 * SIZEOF_FLOAT) as GLsizei,
                std::ptr::null(),
            );

            // Unbinding
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);

            let projection = cgmath::ortho(-100.0, 100.0, -100.0, 100.0, 100.0, -100.0);
            // shader_program.set_matrix("projection", projection);
            // shader_program.set_matrix("model", Matrix4::identity());
        }

        Self {
            vbo: vbo,
            ibo: ibo,
            vao: vao,
            program: shader_program,
            index_data: index_data,
        }
    }

    pub fn draw(&mut self) {
        unsafe {
            self.program.set_used();

            gl::BindVertexArray(self.vao);
            gl::DrawElements(
                gl::TRIANGLES,
                self.index_data.len() as i32,
                gl::UNSIGNED_INT,
                self.index_data.as_ptr() as *const std::os::raw::c_void,
            );
        }
    }
}
