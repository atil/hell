use crate::object::Object;
use crate::render::shader::*;
use crate::render::*;
use cgmath::*;

pub struct DirectionalLight {
    pub direction: Vector3<f32>,
    pub view: Matrix4<f32>,
    pub projection: Matrix4<f32>,
    pub color: Vector4<f32>,

    fbo: BufferHandle,
    shader: Shader,
    pub depth_texture_handle: TextureHandle,
}

impl DirectionalLight {
    pub fn new() -> DirectionalLight {
        let s = 100.0;
        let direction = Vector3::new(100.0, -100.0, -20.0);
        let view = Matrix4::look_at(
            EuclideanSpace::from_vec(-direction),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::unit_y(),
        );
        let projection = cgmath::ortho(-s, s, -s, s, NEAR_PLANE, FAR_PLANE);
        let color = Vector4::new(0.2, 0.1, 0.0, 1.0);

        let mut depth_fbo: BufferHandle = 0;
        let depth_texture_handle: TextureHandle;

        let shader = Shader::from_file("src/shaders/shadowmap_depth_directional.glsl", false)
            .expect("\nProblem loading directional shadowmap depth shader\n");

        unsafe {
            gl::GenFramebuffers(1, &mut depth_fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, depth_fbo);
            depth_texture_handle = DirectionalLight::create_shadowmap_texture();
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
            shader.set_mat4("u_light_v", view);
            shader.set_mat4("u_light_p", projection);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, depth_texture_handle);
        }

        DirectionalLight {
            direction: direction,
            view: view,
            projection: projection,
            color: color,
            fbo: depth_fbo,
            shader: shader,
            depth_texture_handle: depth_texture_handle,
        }
    }

    pub unsafe fn fill_depth_texture(&mut self, objects: &Vec<Object>) {
        self.shader.set_used();
        gl::Viewport(0, 0, SHADOWMAP_SIZE, SHADOWMAP_SIZE);
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
        gl::Clear(gl::DEPTH_BUFFER_BIT);
        for obj in objects {
            self.shader.set_mat4("u_model", obj.transform);
            obj.material.draw();
        }
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }

    unsafe fn create_shadowmap_texture() -> TextureHandle {
        let mut depth_texture_handle = 0;
        gl::GenTextures(1, &mut depth_texture_handle);
        gl::BindTexture(gl::TEXTURE_2D, depth_texture_handle);
        gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_WRAP_S,
            gl::CLAMP_TO_BORDER as i32,
        );
        gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_WRAP_T,
            gl::CLAMP_TO_BORDER as i32,
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

        let border_color = vec![1.0, 1.0, 1.0, 1.0];
        gl::TexParameterfv(
            gl::TEXTURE_2D,
            gl::TEXTURE_BORDER_COLOR,
            border_color.as_ptr(),
        );

        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::DEPTH_COMPONENT as i32,
            SHADOWMAP_SIZE as i32,
            SHADOWMAP_SIZE as i32,
            0,
            gl::DEPTH_COMPONENT,
            gl::FLOAT,
            std::ptr::null(),
        );

        depth_texture_handle
    }
}
