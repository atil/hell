use gl::types::*;

const SIZEOF_FLOAT: usize = std::mem::size_of::<f32>();

pub struct Cube {
    pub vao: u32,
}

impl Cube {
    pub fn new() -> Cube {
        let vertex_data = vec![
            // back face
            -1.0, -1.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, // bottom-left
            1.0, 1.0, -1.0, 0.0, 0.0, -1.0, 1.0, 1.0, // top-right
            1.0, -1.0, -1.0, 0.0, 0.0, -1.0, 1.0, 0.0, // bottom-right
            1.0, 1.0, -1.0, 0.0, 0.0, -1.0, 1.0, 1.0, // top-right
            -1.0, -1.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, // bottom-left
            -1.0, 1.0, -1.0, 0.0, 0.0, -1.0, 0.0, 1.0, // top-left
            // front face
            -1.0, -1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, // bottom-left
            1.0, -1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, // bottom-right
            1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, // top-right
            1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, // top-right
            -1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, // top-left
            -1.0, -1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, // bottom-left
            // left face
            -1.0, 1.0, 1.0, -1.0, 0.0, 0.0, 1.0, 0.0, // top-right
            -1.0, 1.0, -1.0, -1.0, 0.0, 0.0, 1.0, 1.0, // top-left
            -1.0, -1.0, -1.0, -1.0, 0.0, 0.0, 0.0, 1.0, // bottom-left
            -1.0, -1.0, -1.0, -1.0, 0.0, 0.0, 0.0, 1.0, // bottom-left
            -1.0, -1.0, 1.0, -1.0, 0.0, 0.0, 0.0, 0.0, // bottom-right
            -1.0, 1.0, 1.0, -1.0, 0.0, 0.0, 1.0, 0.0, // top-right
            // right face
            1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, // top-left
            1.0, -1.0, -1.0, 1.0, 0.0, 0.0, 0.0, 1.0, // bottom-right
            1.0, 1.0, -1.0, 1.0, 0.0, 0.0, 1.0, 1.0, // top-right
            1.0, -1.0, -1.0, 1.0, 0.0, 0.0, 0.0, 1.0, // bottom-right
            1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, // top-left
            1.0, -1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, // bottom-left
            // bottom face
            -1.0, -1.0, -1.0, 0.0, -1.0, 0.0, 0.0, 1.0, // top-right
            1.0, -1.0, -1.0, 0.0, -1.0, 0.0, 1.0, 1.0, // top-left
            1.0, -1.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, // bottom-left
            1.0, -1.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, // bottom-left
            -1.0, -1.0, 1.0, 0.0, -1.0, 0.0, 0.0, 0.0, // bottom-right
            -1.0, -1.0, -1.0, 0.0, -1.0, 0.0, 0.0, 1.0, // top-right
            // top face
            -1.0, 1.0, -1.0, 0.0, 1.0, 0.0, 0.0, 1.0, // top-left
            1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, // bottom-right
            1.0, 1.0, -1.0, 0.0, 1.0, 0.0, 1.0, 1.0, // top-right
            1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, // bottom-right
            -1.0, 1.0, -1.0, 0.0, 1.0, 0.0, 0.0, 1.0, // top-left
            -1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, // bottom-left
        ];

        let mut cube_vao = 0;
        let mut cube_vbo = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut cube_vao);
            gl::GenBuffers(1, &mut cube_vbo);
            gl::BindVertexArray(cube_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, cube_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertex_data.len() * SIZEOF_FLOAT) as GLsizeiptr,
                vertex_data.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                (8 * SIZEOF_FLOAT) as i32,
                std::ptr::null(),
            );
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                (8 * SIZEOF_FLOAT) as i32,
                (3 * SIZEOF_FLOAT) as *const GLvoid,
            );
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                (8 * SIZEOF_FLOAT) as i32,
                (6 * SIZEOF_FLOAT) as *const GLvoid,
            );
        }

        Cube { vao: cube_vao }
    }

    pub unsafe fn draw(&self) {
        gl::BindVertexArray(self.vao);
        gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 36);
        gl::BindVertexArray(0);
    }
}

pub struct Quad {
    pub vao: u32,
}

impl Quad {
    pub fn new() -> Quad {
        let vertex_data = vec![
            -1.0, 1.0, 0.0, 0.0, 1.0, -1.0, -1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0,
            -1.0, 0.0, 1.0, 0.0,
        ];

        let mut quad_vao: u32 = 0;
        unsafe {
            let mut quad_vbo: u32 = 0;
            gl::GenVertexArrays(1, &mut quad_vao);
            gl::GenBuffers(1, &mut quad_vbo);
            gl::BindVertexArray(quad_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, quad_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertex_data.len() * SIZEOF_FLOAT) as GLsizeiptr,
                vertex_data.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                (5 * SIZEOF_FLOAT) as i32,
                std::ptr::null(),
            );
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                (5 * SIZEOF_FLOAT) as i32,
                (3 * SIZEOF_FLOAT) as *const GLvoid,
            );
        }

        Quad { vao: quad_vao }
    }

    pub unsafe fn draw(&self) {
        gl::BindVertexArray(self.vao);
        gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 36);
        gl::BindVertexArray(0);
    }
}

pub struct Plane {
    pub vao: u32,
}

impl Plane {
    pub fn new() -> Plane {
        let vertex_data = vec![
            25.0, -0.5, 25.0, 0.0, 1.0, 0.0, 25.0, 0.0, -25.0, -0.5, 25.0, 0.0, 1.0, 0.0, 0.0, 0.0,
            -25.0, -0.5, -25.0, 0.0, 1.0, 0.0, 0.0, 25.0, 25.0, -0.5, 25.0, 0.0, 1.0, 0.0, 25.0,
            0.0, -25.0, -0.5, -25.0, 0.0, 1.0, 0.0, 0.0, 25.0, 25.0, -0.5, -25.0, 0.0, 1.0, 0.0,
            25.0, 25.0,
        ];

        let mut plane_vao = 0;
        let mut plane_vbo = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut plane_vao);
            gl::GenBuffers(1, &mut plane_vbo);
            gl::BindVertexArray(plane_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, plane_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertex_data.len() * SIZEOF_FLOAT) as GLsizeiptr,
                vertex_data.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                (8 * SIZEOF_FLOAT) as i32,
                std::ptr::null(),
            );
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                (8 * SIZEOF_FLOAT) as i32,
                (3 * SIZEOF_FLOAT) as *const GLvoid,
            );
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                (8 * SIZEOF_FLOAT) as i32,
                (6 * SIZEOF_FLOAT) as *const GLvoid,
            );
        }

        Plane { vao: plane_vao }
    }
}

pub struct Triangle {
    pub vao: u32,
}

impl Triangle {
    pub fn new() -> Triangle {
        let s = 0.5;
        let vertex_data = vec![
            -s, -s, 0.0, // left
            s, -s, 0.0, // right
            0.0, s, 0.0, // top
        ];

        let mut triangle_vao = 0;
        let mut triangle_vbo = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut triangle_vao);
            gl::GenBuffers(1, &mut triangle_vbo);
            gl::BindVertexArray(triangle_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, triangle_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertex_data.len() * SIZEOF_FLOAT) as GLsizeiptr,
                vertex_data.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                (3 * SIZEOF_FLOAT) as i32,
                std::ptr::null(),
            );
        }

        Triangle { vao: triangle_vao }
    }
}

impl Drop for Triangle {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}
