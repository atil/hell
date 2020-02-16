extern crate cgmath;
extern crate gl;
extern crate sdl2;

use cgmath::*;
use gl::types::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::path::Path;

pub mod camera;
use camera::Camera;
pub mod material;
use material::Material;
pub mod mesh;
use mesh::Mesh;
pub mod shader;

struct Screen {
    x: u32,
    y: u32,
}

const SCREEN_SIZE: Screen = Screen { x: 800, y: 600 };

fn main() {
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

    let (tobj_models, tobj_mats) = match tobj::load_obj(&Path::new("test.obj")) {
        Ok(cube_obj) => cube_obj,
        Err(e) => panic!("Error during loading models: {}", e),
    };

    let projection: Matrix4<f32> = cgmath::perspective(
        cgmath::Deg(45.0),
        SCREEN_SIZE.x as f32 / SCREEN_SIZE.y as f32,
        0.1,
        100.0,
    );

    let mesh = Mesh::new(&tobj_models[0].mesh);
    let material = Material::new(
        &mesh.vertex_data,
        &mesh.index_data,
        &tobj_mats[0],
        projection,
    );

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::Viewport(0, 0, SCREEN_SIZE.x as GLint, SCREEN_SIZE.y as GLint);
        gl::ClearColor(0.5, 0.3, 0.3, 1.0);
    }

    let mut event_pump = sdl_context.event_pump().unwrap();
    let timer = sdl_context.timer().unwrap();

    let mut dt: f32;
    let mut last_tick_time: u64;
    let mut now_tick_time = timer.performance_counter();
    let mut camera = Camera::new();

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

        const SPEED: f32 = 0.004;
        if keys.contains(&Keycode::W) {
            camera.position += cgmath::EuclideanSpace::to_vec(camera.forward) * SPEED * dt;
        } else if keys.contains(&Keycode::S) {
            camera.position -= cgmath::EuclideanSpace::to_vec(camera.forward) * SPEED * dt;
        } else if keys.contains(&Keycode::A) {
            let local_left =
                cgmath::EuclideanSpace::to_vec(camera.forward).cross(Vector3::unit_y());
            camera.position -= local_left * SPEED * dt;
        } else if keys.contains(&Keycode::D) {
            let local_left =
                cgmath::EuclideanSpace::to_vec(camera.forward).cross(Vector3::unit_y());
            camera.position += local_left * SPEED * dt;
        }

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            let mut transform = Matrix4::<f32>::identity();
            transform = transform * Matrix4::from_translation(Vector3::new(0.0, -1.0, -10.0));
            transform = transform * Matrix4::from_axis_angle(Vector3::unit_y(), Deg(30.0));
            material.draw(&mesh.index_data, transform, camera.get_view_matrix())
        }

        window.gl_swap_window();
    }
}
