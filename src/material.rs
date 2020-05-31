extern crate tobj;
use crate::light::DirectionalLight;
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
    vbo: u32,
    ibo: u32,
    vao: u32,
    fbo: u32,
    m_type: MaterialType,
    program: Program,
    depth_program: Program,
    index_data: Vec<u32>,
}

impl Material {
    pub fn new(
        vertex_data: Vec<f32>,
        index_data: Vec<u32>,
        tobj_mat: tobj::Material,
        projection: Matrix4<f32>,
    ) -> Material {
        let (mat_type, shader_path) = get_material_type(&tobj_mat);
        let program = Program::from_shader(shader_path).expect("Problem loading world shader");
        let depth_program =
            Program::from_shader("src/shaders/depth.glsl").expect("Problem loading depth shader");

        let mut vbo: GLuint = 0;
        let mut ibo: GLuint = 0;
        let mut vao: GLuint = 0;
        let mut fbo: GLuint = 0;

        unsafe {
            gl::GenFramebuffers(1, &mut fbo);
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
                    program.set_i32("u_shadowmap", 1);

                    let d = Vector3::new(-1.0, -1.0, 0.0).normalize();
                    program.set_vec3("u_light_dir", d.x, d.y, d.z);
                }
                MaterialType::Color(color) => {
                    program.set_vec3("color0", color.r, color.g, color.b);
                }
            }

            // Framebuffer
            // gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
            // let depth_texture_handle = texture::create_depth_texture();
            // gl::FramebufferTexture2D(
            //     gl::FRAMEBUFFER,
            //     gl::DEPTH_ATTACHMENT,
            //     gl::TEXTURE_2D,
            //     depth_texture_handle,
            //     0,
            // );
            // gl::ReadBuffer(gl::NONE);
            // gl::DrawBuffer(gl::NONE);
            // gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            program.set_matrix("u_projection", projection);
        }

        Material {
            vbo: vbo,
            ibo: ibo,
            vao: vao,
            fbo: fbo,
            m_type: mat_type,
            program: program,
            depth_program: depth_program,
            index_data: index_data,
        }
    }

    pub unsafe fn draw_to_depth_buffer(&self, model: Matrix4<f32>, light: &DirectionalLight) {
        self.depth_program.set_matrix("u_light_v", light.view);
        self.depth_program.set_matrix("u_light_p", light.projection);
        self.depth_program.set_matrix("u_light_model", light.model);
        self.depth_program.set_matrix("u_model", model);
        gl::Viewport(0, 0, 1024, 1024);
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
        gl::Clear(gl::DEPTH_BUFFER_BIT);
        gl::BindVertexArray(self.vao);
        gl::DrawElements(
            gl::TRIANGLES,
            self.index_data.len() as i32,
            gl::UNSIGNED_INT,
            self.index_data.as_ptr() as *const std::os::raw::c_void,
        );
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }

    pub unsafe fn draw(&self, model: Matrix4<f32>, view: Matrix4<f32>, light: &DirectionalLight) {
        // self.depth_program.set_used();
        // let light_model = Matrix4::from_translation(Vector3::new(100.0, 100.0, 0.0));
        // let light_view = Matrix4::look_at(
        //     Point3::new(100.0, 100.0, 0.0),
        //     Point3::new(0.0, 0.0, 0.0),
        //     Vector3::unit_y(),
        // );
        // let light_projection = cgmath::ortho(-10.0, 10.0, -10.0, 10.0, 0.1, 1000.0);
        // let light_vp = light_projection * light_view;
        // self.depth_program.set_matrix("u_light_vp", light_vp);
        // self.depth_program.set_matrix("u_light_model", light_model);
        // self.depth_program.set_matrix("u_model", model);
        // gl::Viewport(0, 0, 1024, 1024);
        // gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
        // gl::Clear(gl::DEPTH_BUFFER_BIT);
        // gl::BindVertexArray(self.vao);
        // gl::DrawElements(
        //     gl::TRIANGLES,
        //     self.index_data.len() as i32,
        //     gl::UNSIGNED_INT,
        //     self.index_data.as_ptr() as *const std::os::raw::c_void,
        // );
        // gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        //
        self.program.set_used();
        self.program.set_matrix("u_model", model);
        self.program.set_matrix("u_view", view);
        self.program.set_matrix("u_light_v", light.view);
        self.program.set_matrix("u_light_p", light.projection);
        self.program.set_i32("u_texture0", 0);
        self.program.set_i32("u_shadowmap", 1);

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
            gl::DeleteFramebuffers(1, &self.fbo);
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
