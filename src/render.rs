use crate::lighting;
use crate::lighting::*;
use crate::object::Object;
use crate::shader::*;
use crate::skybox::Skybox;
use cgmath::*;

pub const SCREEN_SIZE: (u32, u32) = (1280, 720);
pub const SHADOWMAP_SIZE: i32 = 2048;
pub const DRAW_FRAMEBUFFER_SIZE: (u32, u32) = (640, 360);
pub const SIZEOF_FLOAT: usize = std::mem::size_of::<f32>();

#[allow(dead_code)] // The glContext needs to be kept alive, even though not being read
pub struct Renderer {
    window: sdl2::video::Window,
    gl_context: sdl2::video::GLContext,
    shadowmap: Shadowmap,
    skybox: Skybox,

    world_shader: Shader,
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
        let directional_light = DirectionalLight::new();
        let point_light = PointLight::new();
        let shadowmap = lighting::Shadowmap::new(&directional_light);
        let world_shader = Shader::from_file("src/shaders/triangle.glsl", false)
            .expect("\nProblem loading world shader\n");

        let draw_fbo = unsafe { create_draw_backbuffer() };

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::ClearColor(0.1, 0.05, 0.05, 1.0);

            world_shader.set_used();
            world_shader.set_i32("u_texture0", 0);
            world_shader.set_i32("u_shadowmap", 1);
            world_shader.set_vec3(
                "u_point_light_pos",
                point_light.position.x,
                point_light.position.y,
                point_light.position.z,
            );
            world_shader.set_f32("u_point_light_intensity", point_light.intensity);
            world_shader.set_f32("u_point_light_attenuation", point_light.attenuation);
            world_shader.set_vec3(
                "u_light_dir",
                directional_light.direction.x,
                directional_light.direction.y,
                directional_light.direction.z,
            );
            world_shader.set_mat4("u_light_v", directional_light.view);
            world_shader.set_mat4("u_light_p", directional_light.projection);
            world_shader.set_vec4(
                "u_light_color",
                directional_light.color.x,
                directional_light.color.y,
                directional_light.color.z,
                directional_light.color.w,
            );
            world_shader.set_mat4("u_projection", projection);

            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, shadowmap.depth_texture_handle);
        }

        Self {
            window: window,
            gl_context: gl_context,
            world_shader: world_shader,

            shadowmap: shadowmap,

            skybox: Skybox::new(projection),
            draw_fbo: draw_fbo,
        }
    }

    pub unsafe fn render(&mut self, objects: &Vec<Object>, player_v: Matrix4<f32>) {
        // Render to depth buffer from the light's perspective
        self.shadowmap.draw(&objects);

        // Render world to backbuffer
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

        // Fill the depth==1 fragments with sky texture
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

unsafe fn create_draw_backbuffer() -> u32 {
    let mut draw_fbo = 0;
    gl::GenFramebuffers(1, &mut draw_fbo);
    gl::BindFramebuffer(gl::FRAMEBUFFER, draw_fbo);

    let mut draw_texture_handle = 0;
    gl::GenTextures(1, &mut draw_texture_handle);
    gl::BindTexture(gl::TEXTURE_2D, draw_texture_handle);
    gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        gl::RGB as i32,
        DRAW_FRAMEBUFFER_SIZE.0 as i32,
        DRAW_FRAMEBUFFER_SIZE.1 as i32,
        0,
        gl::RGB,
        gl::UNSIGNED_BYTE,
        std::ptr::null(),
    );
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

    gl::FramebufferTexture2D(
        gl::FRAMEBUFFER,
        gl::COLOR_ATTACHMENT0,
        gl::TEXTURE_2D,
        draw_texture_handle,
        0,
    );

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

    draw_fbo
}
