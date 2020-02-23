use crate::object::Object;
use cgmath::*;
use gl::types::*;

struct Screen {
    x: u32,
    y: u32,
}

const SCREEN_SIZE: Screen = Screen { x: 800, y: 600 };

pub fn init(sdl_context: &sdl2::Sdl) -> (sdl2::video::Window, sdl2::video::GLContext) {
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
    let gl_context = window.gl_create_context().unwrap();
    let _gl = gl::load_with(|s| sdl_video.gl_get_proc_address(s) as *const std::os::raw::c_void);

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::Viewport(0, 0, SCREEN_SIZE.x as GLint, SCREEN_SIZE.y as GLint);
        gl::ClearColor(0.5, 0.3, 0.3, 1.0);
    }

    (window, gl_context)
}

pub fn render(window: &sdl2::video::Window, objects: &Vec<Object>, view_matrix: Matrix4<f32>) {
    unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }

    for obj in objects {
        obj.draw(view_matrix);
    }

    window.gl_swap_window();
}

pub fn get_projection_matrix() -> Matrix4<f32> {
    cgmath::perspective(
        cgmath::Deg(45.0),
        SCREEN_SIZE.x as f32 / SCREEN_SIZE.y as f32,
        0.1,
        100.0,
    )
}
