use crate::object::Object;
use crate::render;
use crate::render::shader::*;
use crate::render::{BufferHandle, TextureHandle};
use cgmath::*;

pub struct PointLight {
    pub position: Point3<f32>,
    pub intensity: f32,
    pub attenuation: f32,

    shader: Shader,
}

impl PointLight {
    pub fn new(
        position: Point3<f32>,
        intensity: f32,
        attenuation: f32,
        light_index: u32,
    ) -> PointLight {
        let shader = Shader::from_file("src/shaders/shadowmap_depth_point.glsl", true)
            .expect("\nProblem loading point shadowmap depth shader\n");

        let proj = cgmath::perspective(
            cgmath::Deg(90.0),
            render::SHADOWMAP_SIZE as f32 / render::SHADOWMAP_SIZE as f32,
            render::NEAR_PLANE,
            render::FAR_PLANE,
        );

        let pos = position;

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

        unsafe {
            shader.set_used();
            shader.set_mat4("u_shadow_matrices[0]", proj * v0);
            shader.set_mat4("u_shadow_matrices[1]", proj * v1);
            shader.set_mat4("u_shadow_matrices[2]", proj * v2);
            shader.set_mat4("u_shadow_matrices[3]", proj * v3);
            shader.set_mat4("u_shadow_matrices[4]", proj * v4);
            shader.set_mat4("u_shadow_matrices[5]", proj * v5);
            shader.set_f32("u_far_plane", render::FAR_PLANE);
            shader.set_vec3("u_light_pos", pos.x, pos.y, pos.z);
            shader.set_i32("u_light_index", light_index as i32);
        }

        PointLight {
            position: position,
            intensity: intensity,
            attenuation: attenuation,
            shader: shader,
        }
    }

    pub unsafe fn fill_depth_cubemap(&mut self, objects: &Vec<Object>) {
        self.shader.set_used();
        for obj in objects {
            self.shader.set_mat4("u_model", obj.transform);
            obj.material.draw();
        }
    }
}

pub unsafe fn create_point_light_framebuffer(depth_cubemap_handle: TextureHandle) -> BufferHandle {
    let mut depth_fbo = 0;
    gl::GenFramebuffers(1, &mut depth_fbo);
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

    depth_fbo
}

pub unsafe fn create_cubemap_array(point_light_count: usize) -> TextureHandle {
    let mut cubemap_array_handle = 0;
    gl::GenTextures(1, &mut cubemap_array_handle);
    gl::BindTexture(gl::TEXTURE_CUBE_MAP_ARRAY, cubemap_array_handle);
    gl::TexParameteri(
        gl::TEXTURE_CUBE_MAP_ARRAY,
        gl::TEXTURE_MIN_FILTER,
        gl::LINEAR as i32,
    );
    gl::TexParameteri(
        gl::TEXTURE_CUBE_MAP_ARRAY,
        gl::TEXTURE_MAG_FILTER,
        gl::LINEAR as i32,
    );
    gl::TexParameteri(
        gl::TEXTURE_CUBE_MAP_ARRAY,
        gl::TEXTURE_WRAP_S,
        gl::CLAMP_TO_EDGE as i32,
    );
    gl::TexParameteri(
        gl::TEXTURE_CUBE_MAP_ARRAY,
        gl::TEXTURE_WRAP_T,
        gl::CLAMP_TO_EDGE as i32,
    );
    gl::TexParameteri(
        gl::TEXTURE_CUBE_MAP_ARRAY,
        gl::TEXTURE_WRAP_R,
        gl::CLAMP_TO_EDGE as i32,
    );
    gl::TexImage3D(
        gl::TEXTURE_CUBE_MAP_ARRAY,
        0,
        gl::DEPTH_COMPONENT as i32,
        render::SHADOWMAP_SIZE,
        render::SHADOWMAP_SIZE,
        (6 * point_light_count) as i32,
        0,
        gl::DEPTH_COMPONENT,
        gl::FLOAT,
        std::ptr::null(),
    );

    cubemap_array_handle
}
