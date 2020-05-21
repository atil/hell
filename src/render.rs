use crate::object::Object;
use cgmath::*;
use gl::types::*;

struct Screen {
    x: u32,
    y: u32,
}

#[allow(dead_code)] // The glContext needs to be kept alive, even though not being read
pub struct Renderer {
    window: sdl2::video::Window,
    gl_context: sdl2::video::GLContext,
}

const SCREEN_SIZE: Screen = Screen { x: 800, y: 600 };

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

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Viewport(0, 0, SCREEN_SIZE.x as GLint, SCREEN_SIZE.y as GLint);
            gl::ClearColor(0.1, 0.05, 0.05, 1.0);
        }

        Self {
            window: window,
            gl_context: gl_context,
        }
    }

    pub unsafe fn render(&mut self, objects: &Vec<Object>, view_matrix: Matrix4<f32>) {
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        for obj in objects {
            obj.draw(view_matrix);
        }
    }

    pub fn finish_render(&mut self) {
        self.window.gl_swap_window();
    }
}

pub fn get_projection_matrix() -> Matrix4<f32> {
    cgmath::perspective(
        cgmath::Deg(45.0),
        SCREEN_SIZE.x as f32 / SCREEN_SIZE.y as f32,
        0.1,
        1000.0,
    )
}
