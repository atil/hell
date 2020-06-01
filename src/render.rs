use crate::light::DirectionalLight;
use crate::object::Object;
use crate::shader::*;
use crate::texture;
use cgmath::*;

struct Screen {
    x: u32,
    y: u32,
}

#[allow(dead_code)] // The glContext needs to be kept alive, even though not being read
pub struct Renderer {
    window: sdl2::video::Window,
    gl_context: sdl2::video::GLContext,
    world_program: Program,
    depth_fbo: u32,
    depth_program: Program,
    light: DirectionalLight,
}

const SCREEN_SIZE: Screen = Screen { x: 1366, y: 768 };

impl Renderer {
    pub fn init(sdl_context: &sdl2::Sdl) -> Self {
        let sdl_video = sdl_context.video().unwrap();
        let gl_attr = sdl_video.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(4, 1);
        let window = sdl_video
            .window("Progress.", SCREEN_SIZE.x, SCREEN_SIZE.y)
            .opengl()
            .resizable()
            .build()
            .unwrap();
        sdl_context.mouse().set_relative_mouse_mode(true);
        let gl_context = window.gl_create_context().unwrap();
        gl::load_with(|s| sdl_video.gl_get_proc_address(s) as *const std::os::raw::c_void);

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

        let projection = cgmath::perspective(
            cgmath::Deg(45.0),
            SCREEN_SIZE.x as f32 / SCREEN_SIZE.y as f32,
            0.1,
            1000.0,
        );

        let light = DirectionalLight::new();

        let world_program = Program::from_shader("src/shaders/triangle.glsl")
            .expect("Problem loading world shader");
        unsafe {
            world_program.set_used();
            world_program.set_i32("u_texture0", 0);
            world_program.set_i32("u_shadowmap", 1);
            world_program.set_vec3(
                "u_light_pos",
                light.position.x,
                light.position.y,
                light.position.z,
            );
            world_program.set_mat4("u_light_v", light.view);
            world_program.set_mat4("u_light_p", light.projection);
            world_program.set_vec4(
                "u_light_color",
                light.color.x,
                light.color.y,
                light.color.z,
                light.color.w,
            );
            world_program.set_mat4("u_projection", projection);

            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, depth_texture_handle);
        }

        let depth_program =
            Program::from_shader("src/shaders/depth.glsl").expect("Problem loading depth shader");
        unsafe {
            depth_program.set_used();
            depth_program.set_mat4("u_light_v", light.view);
            depth_program.set_mat4("u_light_p", light.projection);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, depth_texture_handle);
        }

        Self {
            window: window,
            gl_context: gl_context,
            depth_fbo: depth_fbo,
            depth_program: depth_program,
            world_program: world_program,
            light: light,
        }
    }

    pub unsafe fn render(&mut self, objects: &Vec<Object>, player_v: Matrix4<f32>) {
        // Rendering to depth buffer
        self.depth_program.set_used();
        gl::Viewport(0, 0, 1024, 1024);
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.depth_fbo);
        gl::Clear(gl::DEPTH_BUFFER_BIT);
        for obj in objects {
            self.depth_program.set_mat4("u_model", obj.transform);
            obj.material.draw();
        }
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        // Rendering to screen
        self.world_program.set_used();
        self.world_program.set_mat4("u_view", player_v);
        gl::Viewport(0, 0, SCREEN_SIZE.x as i32, SCREEN_SIZE.y as i32);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        for obj in objects {
            self.world_program.set_mat4("u_model", obj.transform);
            obj.material.draw();
        }
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
