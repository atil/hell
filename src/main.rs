extern crate cgmath;
extern crate gl;
extern crate sdl2;
extern crate tobj;

use cgmath::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::path::Path;

pub mod camera;
use camera::Camera;
pub mod render;
pub mod shader;

struct Screen {
    x: u32,
    y: u32,
}

fn main() {
    const SCREEN_SIZE: Screen = Screen { x: 800, y: 600 };

    let sdl_context = sdl2::init().unwrap();
    let sdl_video: sdl2::VideoSubsystem = sdl_context.video().unwrap();

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
    let _gl = gl::load_with(|s| sdl_video.gl_get_proc_address(s) as *const std::os::raw::c_void);

    use std::ffi::CString;
    let vert_shader =
        shader::Shader::from_vert_source(&CString::new(include_str!("triangle.vert")).unwrap())
            .unwrap();

    let frag_shader =
        shader::Shader::from_frag_source(&CString::new(include_str!("triangle.frag")).unwrap())
            .unwrap();

    let shader_program = shader::Program::from_shaders(&[vert_shader, frag_shader]).unwrap();

    let cube_obj = tobj::load_obj(&Path::new("cube.obj"));
    assert!(cube_obj.is_ok());
    let (models, _) = cube_obj.unwrap();

    let mut camera = Camera::new();

    let vertices = models[0].mesh.positions.clone(); // This is bad
    let indices = models[0].mesh.indices.clone();

    let mut vbo: gl::types::GLuint = 0;
    let mut ibo: gl::types::GLuint = 0;
    let mut vao: gl::types::GLuint = 0;
    unsafe {
        let projection: Matrix4<f32> = cgmath::perspective(
            Deg(45.0),
            SCREEN_SIZE.x as f32 / SCREEN_SIZE.y as f32,
            0.1,
            100.0,
        );
        shader_program.set_matrix("projection", projection);

        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ibo);
        gl::GenVertexArrays(1, &mut vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,                                                       // target
            (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr, // size of data in bytes
            vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
            gl::STATIC_DRAW,                               // usage
        );
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER, // target
            (indices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr, // size of data in bytes
            indices.as_ptr() as *const gl::types::GLvoid, // pointer to data
            gl::STATIC_DRAW,                              // usage
        );

        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
        gl::VertexAttribPointer(
            0,         // index of the generic vertex attribute ("layout (location = 0)")
            3,         // the number of components per generic vertex attribute
            gl::FLOAT, // data type
            gl::FALSE, // normalized (int-to-float conversion)
            (3 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
            std::ptr::null(),                                     // offset of the first component
        );

        // Unbinding
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);

        gl::Viewport(
            0,
            0,
            SCREEN_SIZE.x as gl::types::GLint,
            SCREEN_SIZE.y as gl::types::GLint,
        );
        gl::ClearColor(0.5, 0.3, 0.3, 1.0);
    }

    let mut event_pump = sdl_context.event_pump().unwrap();
    let timer = sdl_context.timer().unwrap();

    let mut dt: f32;
    let mut last_tick_time: u64;
    let mut now_tick_time = timer.performance_counter();

    'main: loop {
        let mut mouse_x = 0.0;
        let mut mouse_y = 0.0;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'main,
                Event::MouseMotion { xrel, yrel, .. } => {
                    mouse_x = xrel as f32;
                    mouse_y = yrel as f32;
                }
                _ => {}
            }
        }

        let sensitivity_x = 0.004;
        let horz_rot =
            Quaternion::from_axis_angle(Vector3::unit_y(), Rad(-mouse_x) * sensitivity_x);
        camera.forward = horz_rot.rotate_point(camera.forward);

        let sensitivity_y = 0.004;
        let left = EuclideanSpace::to_vec(camera.forward).cross(Vector3::unit_y());
        camera.forward = Quaternion::from_axis_angle(left, Rad(-mouse_y) * sensitivity_y)
            .rotate_point(camera.forward);

        last_tick_time = now_tick_time;
        now_tick_time = timer.performance_counter();
        dt = ((now_tick_time - last_tick_time) as f32) * 1000.0
            / timer.performance_frequency() as f32;

        let keys: Vec<Keycode> = event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        let speed = 0.004;
        if keys.contains(&Keycode::W) {
            camera.position += cgmath::EuclideanSpace::to_vec(camera.forward) * speed * dt;
        } else if keys.contains(&Keycode::S) {
            camera.position -= cgmath::EuclideanSpace::to_vec(camera.forward) * speed * dt;
        } else if keys.contains(&Keycode::A) {
            let local_left =
                cgmath::EuclideanSpace::to_vec(camera.forward).cross(Vector3::unit_y());
            camera.position -= local_left * speed * dt;
        } else if keys.contains(&Keycode::D) {
            let local_left =
                cgmath::EuclideanSpace::to_vec(camera.forward).cross(Vector3::unit_y());
            camera.position += local_left * speed * dt;
        }

        unsafe {
            shader_program.set_used();
            shader_program.set_matrix("view", camera.get_view_matrix());

            let model: Matrix4<f32> = Matrix4::from_translation(Vector3::new(0.0, -1.0, -10.0));
            shader_program.set_matrix("model", model);

            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::BindVertexArray(vao);
            gl::DrawElements(
                gl::TRIANGLES,
                indices.len() as i32,
                gl::UNSIGNED_INT,
                indices.as_ptr() as *const std::os::raw::c_void,
            );
        }

        window.gl_swap_window();
    }

    unsafe {
        gl::DeleteVertexArrays(1, &vao);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteBuffers(1, &ibo);
    }
}
