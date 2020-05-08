use crate::shader::*;
use crate::texture;
use cgmath::Vector2;
use gl::types::*;
use rusttype::{point, Font, Scale};
use std::ffi::CString;

const SIZEOF_FLOAT: usize = std::mem::size_of::<f32>();

struct Rect {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
}

impl Rect {
    pub fn new(top: f32, left: f32, width: f32, height: f32) -> Self {
        Self {
            left: left,
            top: top,
            width: width,
            height: height,
        }
    }
}

struct Batch {
    vao: u32,
    texture: u32,
    index_data: Vec<u32>,
}

impl Batch {
    pub fn new(rects: Vec<Rect>, texture: u32) -> Self {
        let mut batch_vertex_data = Vec::new();
        let mut batch_index_data = Vec::new();

        let mut index_offset = 0;
        for r in rects {
            let p0 = Vector2::new(r.left, r.top - r.height);
            let p1 = Vector2::new(r.left + r.width, r.top - r.height);
            let p2 = Vector2::new(r.left + r.width, r.top);
            let p3 = Vector2::new(r.left, r.top);

            let mut rect_vertex_data = vec![
                p0.x, p0.y, 0.0, 0.0, p1.x, p1.y, 1.0, 0.0, p2.x, p2.y, 1.0, 1.0, p3.x, p3.y, 0.0,
                1.0,
            ];

            batch_vertex_data.append(&mut rect_vertex_data);

            let mut rect_index_data = vec![0, 1, 2, 0, 2, 3]
                .iter()
                .map(|i| i + index_offset)
                .collect::<Vec<_>>();
            index_offset += 4;

            batch_index_data.append(&mut rect_index_data);
        }

        let mut vbo: GLuint = 0;
        let mut ibo: GLuint = 0;
        let mut vao: GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ibo);
            gl::GenVertexArrays(1, &mut vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (batch_vertex_data.len() * SIZEOF_FLOAT) as GLsizeiptr,
                batch_vertex_data.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (batch_index_data.len() * SIZEOF_FLOAT) as GLsizeiptr,
                batch_index_data.as_ptr() as *const GLvoid,
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

            // UV
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                (4 * SIZEOF_FLOAT) as GLsizei,
                (2 * SIZEOF_FLOAT) as *const GLvoid,
            );

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        Self {
            vao: vao,
            texture: texture,
            index_data: batch_index_data,
        }
    }
}

pub struct Ui {
    batches: Vec<Batch>,
    program: Program,
}

impl Ui {
    pub fn init() -> Self {
        let font_data = include_bytes!("../assets/RobotoMono-Regular.ttf");
        let font = Font::try_from_bytes(font_data as &[u8]).expect("Error constructing Font");

        let vert_shader =
            Shader::from_vert_source(&CString::new(include_str!("ui.vert")).unwrap()).unwrap();

        let frag_shader =
            Shader::from_frag_source(&CString::new(include_str!("ui.frag")).unwrap()).unwrap();

        let texture1 = texture::load_from_file("assets/prototype.png");
        let texture2 = texture::create_from_text("NOPE", 32.0, font);

        let program = Program::from_shaders(&[vert_shader, frag_shader]).unwrap();

        let rekt1 = Rect::new(0.5, -0.5, 1.0, 1.0);
        let rekt2 = Rect::new(-0.9, -0.4, 0.1, 0.1);

        let batches = vec![
            Batch::new(vec![rekt1], texture1),
            Batch::new(vec![rekt2], texture2),
        ];

        unsafe {
            program.set_i32("texture_ui", 0);
        }

        Self {
            batches: batches,
            program: program,
        }
    }

    pub fn draw(&mut self) {
        unsafe {
            self.program.set_used();

            for batch in self.batches.iter() {
                gl::BindVertexArray(batch.vao);

                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, batch.texture);

                gl::DrawElements(
                    gl::TRIANGLES,
                    batch.index_data.len() as i32,
                    gl::UNSIGNED_INT,
                    batch.index_data.as_ptr() as *const std::os::raw::c_void,
                );
            }
        }
    }
}
