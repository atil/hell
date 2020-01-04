extern crate cgmath;
extern crate gl;
extern crate sdl2;
extern crate tobj;

use cgmath::*;
use std::path::Path;
pub mod render;
pub mod shader;

fn main() {
    const SCREEN_SIZE: (u32, u32) = (640, 480);

    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();

    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 1);

    let window = video_subsystem
        .window("This is how we began", SCREEN_SIZE.0, SCREEN_SIZE.1)
        .opengl()
        .resizable()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().unwrap();
    let _gl =
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

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

    unsafe {
        let mut model: Matrix4<f32> = Matrix4::from_translation(Vector3::new(0.0, 0.0, -10.0));
        model = model * Matrix4::from_angle_y(Deg(10.0));
        model = model * Matrix4::from_angle_x(Deg(30.0));
        shader_program.set_matrix("model", model);

        let view: Matrix4<f32> = Matrix4::look_at(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(0.0, 0.0, -1.0),
            vec3(0.0, 1.0, 0.0),
        );
        shader_program.set_matrix("view", view);

        let projection: Matrix4<f32> = perspective(
            Deg(45.0),
            SCREEN_SIZE.0 as f32 / SCREEN_SIZE.1 as f32,
            0.1,
            100.0,
        );
        shader_program.set_matrix("projection", projection);
    }

    let vertices = models[0].mesh.positions.clone(); // This is bad
    let indices = models[0].mesh.indices.clone();

    let mut vbo: gl::types::GLuint = 0;
    let mut ibo: gl::types::GLuint = 0;
    let mut vao: gl::types::GLuint = 0;

    unsafe {
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

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);

        gl::Viewport(
            0,
            0,
            SCREEN_SIZE.0 as gl::types::GLint,
            SCREEN_SIZE.1 as gl::types::GLint,
        );
        gl::ClearColor(0.5, 0.3, 0.3, 1.0);
    }

    let mut event_pump = sdl.event_pump().unwrap();
    let timer = sdl.timer().unwrap();

    let mut dt: f32;
    let mut last_tick_time: u64;
    let mut now_tick_time = timer.performance_counter();
    let mut rotation_angle = 0f32;

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                _ => {}
            }
        }

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        shader_program.set_used();

        unsafe {
            last_tick_time = now_tick_time;
            now_tick_time = timer.performance_counter();
            dt = ((now_tick_time - last_tick_time) as f32) * 1000.0
                / timer.performance_frequency() as f32;

            rotation_angle += 0.1 * dt;

            let mut model: Matrix4<f32> = Matrix4::from_translation(Vector3::new(0.0, -1.0, -10.0));
            // model = model * Matrix4::from_angle_y(Deg(rotation_angle));
            // model = model * Matrix4::from_angle_x(Deg(30.0));
            shader_program.set_matrix("model", model);

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
}
