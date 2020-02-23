use crate::object::Object;
use cgmath::*;
use gl::types::*;

pub struct Render {
    window: sdl2::video::Window,
}

struct Screen {
    x: u32,
    y: u32,
}

const SCREEN_SIZE: Screen = Screen { x: 800, y: 600 };

impl Render {
    pub fn new(sdl_context: &sdl2::Sdl) -> Render {
        let sdl_video = sdl_context.video().unwrap();
        let gl_attr = sdl_video.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(4, 1);
        let window = sdl_video
            .window("This is how we began", SCREEN_SIZE.x, SCREEN_SIZE.y)
            .opengl()
            .resizable()
            .build()
            .unwrap();
        sdl_context.mouse().set_relative_mouse_mode(true);
        let _gl_context = window.gl_create_context().unwrap();
        let _gl =
            gl::load_with(|s| sdl_video.gl_get_proc_address(s) as *const std::os::raw::c_void);

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Viewport(0, 0, SCREEN_SIZE.x as GLint, SCREEN_SIZE.y as GLint);
            gl::ClearColor(0.5, 0.3, 0.3, 1.0);
        }

        Render { window: window }
    }

    pub fn render(&self, objects: &Vec<Object>, view_matrix: Matrix4<f32>) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        for obj in objects {
            obj.draw(view_matrix);
        }

        self.window.gl_swap_window();
    }
}

pub fn get_projection_matrix() -> Matrix4<f32> {
    cgmath::perspective(
        cgmath::Deg(45.0),
        SCREEN_SIZE.x as f32 / SCREEN_SIZE.y as f32,
        0.1,
        100.0,
    )
}
