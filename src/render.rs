use crate::light::DirectionalLight;
use crate::object::Object;
use crate::shader::*;
use crate::skybox::Skybox;
use crate::texture;
use cgmath::*;

pub const SCREEN_SIZE: (u32, u32) = (1280, 720);
pub const SHADOWMAP_SIZE: i32 = 2048;
pub const DRAW_FRAMEBUFFER_SIZE: (u32, u32) = (640, 360);
pub const SIZEOF_FLOAT: usize = std::mem::size_of::<f32>();

#[allow(dead_code)] // The glContext needs to be kept alive, even though not being read
pub struct Renderer {
    window: sdl2::video::Window,
    gl_context: sdl2::video::GLContext,

    world_shader: Shader,
    depth_fbo: u32,
    depth_shader: Shader,
    light: DirectionalLight,

    skybox: Skybox,

    draw_fbo: u32,
}

impl Renderer {
    pub fn init(sdl_context: &sdl2::Sdl) -> Self {
        let sdl_video = sdl_context.video().unwrap();
        let gl_attr = sdl_video.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(4, 1);
        let window = sdl_video
            .window("Progress.", SCREEN_SIZE.0, SCREEN_SIZE.1)
            .opengl()
            .resizable()
            .build()
            .unwrap();
        sdl_context.mouse().set_relative_mouse_mode(true);
        let gl_context = window.gl_create_context().unwrap();
        gl::load_with(|s| sdl_video.gl_get_proc_address(s) as *const std::os::raw::c_void);

        let projection = cgmath::perspective(
            cgmath::Deg(45.0),
            SCREEN_SIZE.0 as f32 / SCREEN_SIZE.1 as f32,
            0.1,
            1000.0,
        );

        let mut depth_fbo: u32 = 0;
        let depth_texture_handle: u32;
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::ClearColor(0.1, 0.05, 0.05, 1.0);

            // Depth buffer init
            gl::GenFramebuffers(1, &mut depth_fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, depth_fbo);
            depth_texture_handle = texture::create_depth_texture();
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
        }

        let light = DirectionalLight::new();

        let world_shader =
            Shader::from_file("src/shaders/triangle.glsl").expect("Problem loading world shader");
        unsafe {
            world_shader.set_used();
            world_shader.set_i32("u_texture0", 0);
            world_shader.set_i32("u_shadowmap", 1);
            world_shader.set_vec3(
                "u_light_pos",
                light.position.x,
                light.position.y,
                light.position.z,
            );
            world_shader.set_mat4("u_light_v", light.view);
            world_shader.set_mat4("u_light_p", light.projection);
            world_shader.set_vec4(
                "u_light_color",
                light.color.x,
                light.color.y,
                light.color.z,
                light.color.w,
            );
            world_shader.set_mat4("u_projection", projection);

            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, depth_texture_handle);
        }

        let depth_shader =
            Shader::from_file("src/shaders/depth.glsl").expect("Problem loading depth shader");
        unsafe {
            depth_shader.set_used();
            depth_shader.set_mat4("u_light_v", light.view);
            depth_shader.set_mat4("u_light_p", light.projection);
            gl::ActiveTexture(gl::TEXTURE0); // TODO: Is this necessary here?
            gl::BindTexture(gl::TEXTURE_2D, depth_texture_handle);
        }

        // Drawing backbuffer
        let mut draw_fbo = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut draw_fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, draw_fbo);
            let _draw_target_texture_handle = texture::create_draw_target_texture();

            let mut draw_rbo = 0;
            gl::GenRenderbuffers(1, &mut draw_rbo);
            gl::BindRenderbuffer(gl::RENDERBUFFER, draw_rbo);
            gl::RenderbufferStorage(
                gl::RENDERBUFFER,
                gl::DEPTH24_STENCIL8,
                DRAW_FRAMEBUFFER_SIZE.0 as i32,
                DRAW_FRAMEBUFFER_SIZE.1 as i32,
            );
            gl::FramebufferRenderbuffer(
                gl::FRAMEBUFFER,
                gl::DEPTH_STENCIL_ATTACHMENT,
                gl::RENDERBUFFER,
                draw_rbo,
            );

            let err = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            if err != gl::FRAMEBUFFER_COMPLETE {
                println!("Problem while doing framebuffer stuff: {}", err);
            }
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        Self {
            window: window,
            gl_context: gl_context,
            world_shader: world_shader,

            light: light,
            depth_fbo: depth_fbo,
            depth_shader: depth_shader,

            skybox: Skybox::new(projection),
            draw_fbo: draw_fbo,
        }
    }

    pub unsafe fn render(&mut self, objects: &Vec<Object>, player_v: Matrix4<f32>) {
        // Render to depth buffer from the light's perspective
        self.depth_shader.set_used();
        gl::Viewport(0, 0, SHADOWMAP_SIZE, SHADOWMAP_SIZE);
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.depth_fbo);
        gl::Clear(gl::DEPTH_BUFFER_BIT);
        for obj in objects {
            self.depth_shader.set_mat4("u_model", obj.transform);
            obj.material.draw();
        }
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        // Render to backbuffer
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.draw_fbo);
        self.world_shader.set_used();
        self.world_shader.set_mat4("u_view", player_v);
        gl::Viewport(
            0,
            0,
            DRAW_FRAMEBUFFER_SIZE.0 as i32,
            DRAW_FRAMEBUFFER_SIZE.1 as i32,
        );
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        for obj in objects {
            self.world_shader.set_mat4("u_model", obj.transform);
            obj.material.draw();
        }

        self.skybox.draw(player_v);

        // Render to the default framebuffer (the screen)
        gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.draw_fbo);
        gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
        gl::BlitFramebuffer(
            0,
            0,
            DRAW_FRAMEBUFFER_SIZE.0 as i32,
            DRAW_FRAMEBUFFER_SIZE.1 as i32,
            0,
            0,
            SCREEN_SIZE.0 as i32,
            SCREEN_SIZE.1 as i32,
            gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT,
            gl::NEAREST,
        );
    }

    pub fn finish_render(&mut self) {
        self.window.gl_swap_window();
    }
}

pub unsafe fn check_gl_error(tag: &str) {
    let error = gl::GetError();

    if error != 0 {
        println!("[{0}] error: {1}", tag, error);
    }
}
