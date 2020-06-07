use crate::object::Object;
use crate::render;
use crate::shader::*;
use crate::texture;
use cgmath::*;

pub struct DirectionalLight {
    pub position: Point3<f32>,
    pub view: Matrix4<f32>,
    pub projection: Matrix4<f32>,
    pub color: Vector4<f32>,
}

impl DirectionalLight {
    pub fn new() -> DirectionalLight {
        let s = 100.0;
        let pos = Point3::new(-100.0, 100.0, 20.0);
        DirectionalLight {
            position: pos,
            view: Matrix4::look_at(pos, Point3::new(0.0, 0.0, 0.0), Vector3::unit_y()),
            projection: cgmath::ortho(-s, s, -s, s, 0.1, 1000.0),
            color: Vector4::new(0.2, 0.1, 0.0, 1.0),
        }
    }
}

pub struct Shadowmap {
    fbo: u32,
    shader: Shader,
    pub depth_texture_handle: u32,
}

impl Shadowmap {
    pub fn new(light: &DirectionalLight) -> Shadowmap {
        let mut depth_fbo: u32 = 0;
        let depth_texture_handle: u32;

        let shader = Shader::from_file("src/shaders/shadowmap_depth.glsl")
            .expect("\nProblem loading shadowmap depth shader\n");

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::ClearColor(0.1, 0.05, 0.05, 1.0);

            gl::GenFramebuffers(1, &mut depth_fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, depth_fbo);
            depth_texture_handle = texture::create_shadowmap_texture();
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::TEXTURE_2D,
                depth_texture_handle,
                0,
            );
            gl::ReadBuffer(gl::NONE);
            gl::DrawBuffer(gl::NONE);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            shader.set_used();
            shader.set_mat4("u_light_v", light.view);
            shader.set_mat4("u_light_p", light.projection);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, depth_texture_handle);
        }

        Shadowmap {
            fbo: depth_fbo,
            shader: shader,
            depth_texture_handle: depth_texture_handle,
        }
    }

    pub unsafe fn draw(&mut self, objects: &Vec<Object>) {
        self.shader.set_used();
        gl::Viewport(0, 0, render::SHADOWMAP_SIZE, render::SHADOWMAP_SIZE);
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
        gl::Clear(gl::DEPTH_BUFFER_BIT);
        for obj in objects {
            self.shader.set_mat4("u_model", obj.transform);
            obj.material.draw();
        }
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }
}
