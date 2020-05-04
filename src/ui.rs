use crate::shader::*;
use crate::texture;
use cgmath::Vector2;
use gl::types::*;
use std::ffi::CString;

pub struct Ui {
    vao: GLuint,
    program: Program,
    index_data: Vec<u32>,
}

struct UiRect {
    vertex_data: Vec<f32>,
    index_data: Vec<u32>,
}

impl UiRect {
    pub fn new(top: f32, left: f32, width: f32, height: f32) -> Self {
        let p0 = Vector2::new(left, top - height);
        let p1 = Vector2::new(left + width, top - height);
        let p2 = Vector2::new(left + width, top);
        let p3 = Vector2::new(left, top);

        let vertex_data = vec![
            p0.x, p0.y, 0.0, 0.0, p1.x, p1.y, 1.0, 0.0, p2.x, p2.y, 1.0, 1.0, p3.x, p3.y, 0.0, 1.0,
        ];

        // Two triangles: 0/1/2 - 0/2/3
        // Starting from bottom right, going ccw
        Self {
            vertex_data: vertex_data,
            index_data: vec![0, 1, 2, 0, 2, 3],
        }
    }
}

impl Ui {
    pub fn init() -> Self {
        let vert_shader =
            Shader::from_vert_source(&CString::new(include_str!("ui.vert")).unwrap()).unwrap();

        let frag_shader =
            Shader::from_frag_source(&CString::new(include_str!("ui.frag")).unwrap()).unwrap();

        let shader_program = Program::from_shaders(&[vert_shader, frag_shader]).unwrap();

        let rekt2 = UiRect::new(-0.1, -0.1, 0.1, 0.1);
        let rekt3 = UiRect::new(-0.5, -0.4, 0.1, 0.1);

        let rects = vec![rekt2, rekt3];

        let ui_vertex_data = rects.iter().fold(Vec::new(), |mut all, r| {
            all.extend_from_slice(r.vertex_data.as_slice());
            all
        });

        let ui_index_data = rects
            .iter()
            .fold((Vec::new(), 0), |(mut all, mut offset), r| {
                all.extend_from_slice(
                    r.index_data
                        .iter()
                        .map(|index| index + offset)
                        .collect::<Vec<_>>()
                        .as_slice(),
                );
                offset += 4;
                (all, offset)
            })
            .0;

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
                (ui_vertex_data.len() * SIZEOF_FLOAT) as GLsizeiptr,
                ui_vertex_data.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (ui_index_data.len() * SIZEOF_FLOAT) as GLsizeiptr,
                ui_index_data.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            // Position
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                (4 * SIZEOF_FLOAT) as GLsizei,
                std::ptr::null(),
            );

            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                (4 * SIZEOF_FLOAT) as GLsizei,
                (2 * SIZEOF_FLOAT) as *const GLvoid,
            );

            let texture = texture::load_from_file("assets/prototype.png");

            shader_program.set_i32("texture0", texture as i32);

            // Unbinding
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        Self {
            vao: vao,
            program: shader_program,
            index_data: ui_index_data,
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
