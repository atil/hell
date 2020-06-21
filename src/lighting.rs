use crate::object::Object;
use crate::render;
use crate::shader::*;
use cgmath::*;

pub struct DirectionalLight {
    pub direction: Vector3<f32>,
    pub view: Matrix4<f32>,
    pub projection: Matrix4<f32>,
    pub color: Vector4<f32>,
}

impl DirectionalLight {
    pub fn new() -> DirectionalLight {
        let s = 100.0;
        let direction = Vector3::new(100.0, -100.0, -20.0);
        DirectionalLight {
            direction: direction,
            view: Matrix4::look_at(
                EuclideanSpace::from_vec(-direction),
                Point3::new(0.0, 0.0, 0.0),
                Vector3::unit_y(),
            ),
            projection: cgmath::ortho(-s, s, -s, s, render::NEAR_PLANE, render::FAR_PLANE),
            color: Vector4::new(0.2, 0.1, 0.0, 1.0),
        }
    }
}

pub struct DirectionalShadowmap {
    fbo: u32,
    shader: Shader,
    pub depth_texture_handle: u32,
}

impl DirectionalShadowmap {
    pub fn new(light: &DirectionalLight) -> DirectionalShadowmap {
        let mut depth_fbo: u32 = 0;
        let depth_texture_handle: u32;

        let shader = Shader::from_file("src/shaders/shadowmap_depth_directional.glsl", false)
            .expect("\nProblem loading directional shadowmap depth shader\n");

        unsafe {
            gl::GenFramebuffers(1, &mut depth_fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, depth_fbo);
            depth_texture_handle = DirectionalShadowmap::create_shadowmap_texture();
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

        DirectionalShadowmap {
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

    unsafe fn create_shadowmap_texture() -> u32 {
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
            render::SHADOWMAP_SIZE as i32,
            render::SHADOWMAP_SIZE as i32,
            0,
            gl::DEPTH_COMPONENT,
            gl::FLOAT,
            std::ptr::null(),
        );

        depth_texture_handle
    }
}

pub struct PointLight {
    pub position: Point3<f32>,
    pub intensity: f32,
    pub attenuation: f32,
}

impl PointLight {
    pub fn new() -> PointLight {
        PointLight {
            position: Point3::new(24.0, 2.0, -3.0),
            intensity: 1.0,
            attenuation: 0.2,
        }
    }
}

pub struct PointShadowmap {
    fbo: u32,
    shader: Shader,
    pub depth_cubemap_handle: u32,
}

impl PointShadowmap {
    pub fn new() -> PointShadowmap {
        let mut depth_fbo: u32 = 0;
        let depth_cubemap_handle: u32;

        let shader = Shader::from_file("src/shaders/shadowmap_depth_point.glsl", true)
            .expect("\nProblem loading point shadowmap depth shader\n");

        unsafe {
            gl::GenFramebuffers(1, &mut depth_fbo);
            depth_cubemap_handle = PointShadowmap::create_cubemap();
            gl::BindFramebuffer(gl::FRAMEBUFFER, depth_fbo);
            gl::FramebufferTexture(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                depth_cubemap_handle,
                0,
            );
            gl::ReadBuffer(gl::NONE);
            gl::DrawBuffer(gl::NONE);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        PointShadowmap {
            fbo: depth_fbo,
            shader: shader,
            depth_cubemap_handle: depth_cubemap_handle,
        }
    }

    pub unsafe fn draw(&mut self, point_light: &PointLight, objects: &Vec<Object>) {
        let proj = cgmath::perspective(
            cgmath::Deg(90.0),
            render::SHADOWMAP_SIZE as f32 / render::SHADOWMAP_SIZE as f32,
            render::NEAR_PLANE,
            render::FAR_PLANE,
        );

        let pos = point_light.position;

        let v0 = Matrix4::look_at_dir(
            pos,
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
        );
        let v1 = Matrix4::look_at_dir(
            pos,
            Vector3::new(-1.0, 0.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
        );
        let v2 = Matrix4::look_at_dir(
            pos,
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
        );
        let v3 = Matrix4::look_at_dir(
            pos,
            Vector3::new(0.0, -1.0, 0.0),
            Vector3::new(0.0, 0.0, -1.0),
        );
        let v4 = Matrix4::look_at_dir(
            pos,
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(0.0, -1.0, 0.0),
        );
        let v5 = Matrix4::look_at_dir(
            pos,
            Vector3::new(0.0, 0.0, -1.0),
            Vector3::new(0.0, -1.0, 0.0),
        );

        self.shader.set_used();
        self.shader.set_mat4("u_shadow_matrices[0]", proj * v0);
        self.shader.set_mat4("u_shadow_matrices[1]", proj * v1);
        self.shader.set_mat4("u_shadow_matrices[2]", proj * v2);
        self.shader.set_mat4("u_shadow_matrices[3]", proj * v3);
        self.shader.set_mat4("u_shadow_matrices[4]", proj * v4);
        self.shader.set_mat4("u_shadow_matrices[5]", proj * v5);
        self.shader.set_f32("u_far_plane", render::FAR_PLANE);
        self.shader.set_vec3("u_light_pos", pos.x, pos.y, pos.z);

        gl::Viewport(0, 0, render::SHADOWMAP_SIZE, render::SHADOWMAP_SIZE);
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
        gl::Clear(gl::DEPTH_BUFFER_BIT);
        for obj in objects {
            self.shader.set_mat4("u_model", obj.transform);
            obj.material.draw();
        }
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }

    unsafe fn create_cubemap() -> u32 {
        let mut cubemap_handle = 0;
        gl::GenTextures(1, &mut cubemap_handle);
        gl::BindTexture(gl::TEXTURE_CUBE_MAP, cubemap_handle);

        for i in 0..6 {
            gl::TexImage2D(
                gl::TEXTURE_CUBE_MAP_POSITIVE_X + i,
                0,
                gl::DEPTH_COMPONENT as i32,
                render::SHADOWMAP_SIZE,
                render::SHADOWMAP_SIZE,
                0,
                gl::DEPTH_COMPONENT,
                gl::FLOAT,
                std::ptr::null(),
            );
        }

        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_WRAP_S,
            gl::CLAMP_TO_EDGE as i32,
        );
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_WRAP_T,
            gl::CLAMP_TO_EDGE as i32,
        );
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_WRAP_R,
            gl::CLAMP_TO_EDGE as i32,
        );
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_MIN_FILTER,
            gl::LINEAR as i32,
        );
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_MAG_FILTER,
            gl::LINEAR as i32,
        );
        cubemap_handle
    }
}
