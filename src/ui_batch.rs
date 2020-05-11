use cgmath::Vector2;
use gl::types::*;

const SIZEOF_FLOAT: usize = std::mem::size_of::<f32>();

pub struct Rect {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
}

impl Rect {
    pub fn new(left: f32, top: f32, width: f32, height: f32) -> Self {
        Self {
            left: left,
            top: top,
            width: width,
            height: height,
        }
    }
}

pub struct Batch {
    vao: u32,
    texture: u32,
    index_data: Vec<u32>,
    pub draw_single_frame: bool,
}

impl Batch {
    pub fn new(rects: Vec<Rect>, texture: u32, draw_single_frame: bool) -> Self {
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
            draw_single_frame: draw_single_frame,
        }
    }

    pub unsafe fn draw(&self) {
        gl::BindVertexArray(self.vao);

        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, self.texture);

        gl::DrawElements(
            gl::TRIANGLES,
            self.index_data.len() as i32,
            gl::UNSIGNED_INT,
            self.index_data.as_ptr() as *const std::os::raw::c_void,
        );
    }
}
