use crate::light::DirectionalLight;
use crate::object::Object;
use crate::shader::*;
use crate::texture;
use cgmath::*;
use gl::types::*;

pub const SCREEN_SIZE: (u32, u32) = (1280, 720);
pub const SHADOWMAP_SIZE: i32 = 2048;
pub const DRAW_FRAMEBUFFER_SIZE: (u32, u32) = (640, 360);
const SIZEOF_FLOAT: usize = std::mem::size_of::<f32>();

#[allow(dead_code)] // The glContext needs to be kept alive, even though not being read
pub struct Renderer {
    window: sdl2::video::Window,
    gl_context: sdl2::video::GLContext,

    world_shader: Shader,
    depth_fbo: u32,
    depth_shader: Shader,
    light: DirectionalLight,

    skybox_vao: u32,
    skybox_shader: Shader,
    skybox_cubemap_handle: u32,

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

        let mut depth_fbo: u32 = 0;
        let depth_texture_handle: u32;
        let mut skybox_vao: u32 = 0;
        let skybox_cubemap_handle: u32;
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

            // Skybox
            let mut skybox_vbo = 0;
            let skybox_vertex_data = skybox_vertex_data();
            gl::GenBuffers(1, &mut skybox_vbo);
            gl::GenVertexArrays(1, &mut skybox_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, skybox_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (skybox_vertex_data.len() * SIZEOF_FLOAT) as GLsizeiptr,
                skybox_vertex_data.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            gl::BindVertexArray(skybox_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, skybox_vbo);
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                (3 * SIZEOF_FLOAT) as i32,
                std::ptr::null(),
            );
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);

            skybox_cubemap_handle = texture::load_cubemap_from_file("assets/skybox/gehenna");
        }

        let projection = cgmath::perspective(
            cgmath::Deg(45.0),
            SCREEN_SIZE.0 as f32 / SCREEN_SIZE.1 as f32,
            0.1,
            1000.0,
        );

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

        let skybox_shader =
            Shader::from_file("src/shaders/skybox.glsl").expect("Problem loading skybox shader");

        unsafe {
            skybox_shader.set_used();
            skybox_shader.set_mat4("u_projection", projection);
            skybox_shader.set_i32("u_skybox", 0);
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

        // Screen quad
        let mut screen_quad_vao = 0;
        unsafe {
            let mut screen_quad_vbo = 0;
            gl::GenVertexArrays(1, &mut screen_quad_vao);
            gl::GenBuffers(1, &mut screen_quad_vbo);

            gl::BindVertexArray(screen_quad_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, screen_quad_vbo);
            let screen_quad_vertex_data = screen_quad_vertex_data();
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (screen_quad_vertex_data.len() * SIZEOF_FLOAT) as GLsizeiptr,
                screen_quad_vertex_data.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            // Position
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                (4 * SIZEOF_FLOAT) as i32,
                std::ptr::null(),
            );

            // Texcoord
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                (4 * SIZEOF_FLOAT) as i32,
                (2 * SIZEOF_FLOAT) as *const GLvoid,
            );

            check_gl_error("screen_quad");
        }

        Self {
            window: window,
            gl_context: gl_context,
            world_shader: world_shader,

            light: light,
            depth_fbo: depth_fbo,
            depth_shader: depth_shader,

            skybox_shader: skybox_shader,
            skybox_vao: skybox_vao,
            skybox_cubemap_handle: skybox_cubemap_handle,

            draw_fbo: draw_fbo,
        }
    }

    pub unsafe fn render(&mut self, objects: &Vec<Object>, player_v: Matrix4<f32>) {
        // Rendering to depth buffer
        self.depth_shader.set_used();
        gl::Viewport(0, 0, SHADOWMAP_SIZE, SHADOWMAP_SIZE);
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.depth_fbo);
        gl::Clear(gl::DEPTH_BUFFER_BIT);
        for obj in objects {
            self.depth_shader.set_mat4("u_model", obj.transform);
            obj.material.draw();
        }
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        // Rendering to backbuffer
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

        // Skybox
        self.skybox_shader.set_used();
        gl::DepthFunc(gl::LEQUAL);
        let skybox_view = Matrix4::from_cols(player_v.x, player_v.y, player_v.z, Vector4::zero());
        self.skybox_shader.set_mat4("u_view", skybox_view);
        gl::BindVertexArray(self.skybox_vao);
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.skybox_cubemap_handle);
        gl::DrawArrays(gl::TRIANGLES, 0, 36);
        gl::BindVertexArray(0);
        gl::DepthFunc(gl::LESS);

        // Render to screen quad
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

fn skybox_vertex_data() -> Vec<f32> {
    // Only positions as vec3
    vec![
        -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0,
        1.0, -1.0, -1.0, -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0,
        1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
        1.0, -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
        1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
        1.0, -1.0, 1.0, 1.0, -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0,
        1.0, -1.0, -1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0,
    ]
}

fn screen_quad_vertex_data() -> Vec<f32> {
    // ndc: vec2 pos, vec2 texcoord
    vec![
        -1.0, 1.0, 0.0, 1.0, -1.0, -1.0, 0.0, 0.0, 1.0, -1.0, 1.0, 0.0, -1.0, 1.0, 0.0, 1.0, 1.0,
        -1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0,
    ]
}
