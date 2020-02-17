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
pub mod object;
pub mod shader;
use object::Object;

struct Screen {
    x: u32,
    y: u32,
}

struct Config {
    x_sensitivity: f32,
    y_sensitivity: f32,
    screen: Screen,
    move_speed: f32,
}

const CONFIG: Config = Config {
    x_sensitivity: 0.004,
    y_sensitivity: 0.004,
    screen: Screen { x: 800, y: 600 },
    move_speed: 0.004,
};

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let sdl_video: sdl2::VideoSubsystem = sdl_context.video().unwrap();

    let gl_attr = sdl_video.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 1);

    let window = sdl_video
        .window("This is how we began", CONFIG.screen.x, CONFIG.screen.y)
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
        CONFIG.screen.x as f32 / CONFIG.screen.y as f32,
        0.1,
        100.0,
    );

    let (vertex_data, index_data) = Mesh::read_vertex_data(&tobj_models[0].mesh);
    let material = Material::new(&vertex_data, index_data, &tobj_mats[0], projection);

    let mut object = Object::new(&material);
    object.translate(Vector3::new(0.0, -1.0, -10.0));
    object.rotate(Vector3::unit_y(), 30.0);

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::Viewport(0, 0, CONFIG.screen.x as GLint, CONFIG.screen.y as GLint);
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

        let horz_rot =
            Quaternion::from_axis_angle(Vector3::unit_y(), Rad(-mouse_x) * CONFIG.x_sensitivity);
        camera.forward = horz_rot.rotate_point(camera.forward);

        let left = EuclideanSpace::to_vec(camera.forward).cross(Vector3::unit_y());
        camera.forward = Quaternion::from_axis_angle(left, Rad(-mouse_y) * CONFIG.y_sensitivity)
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

        if keys.contains(&Keycode::W) {
            camera.position +=
                cgmath::EuclideanSpace::to_vec(camera.forward) * CONFIG.move_speed * dt;
        } else if keys.contains(&Keycode::S) {
            camera.position -=
                cgmath::EuclideanSpace::to_vec(camera.forward) * CONFIG.move_speed * dt;
        } else if keys.contains(&Keycode::A) {
            let local_left =
                cgmath::EuclideanSpace::to_vec(camera.forward).cross(Vector3::unit_y());
            camera.position -= local_left * CONFIG.move_speed * dt;
        } else if keys.contains(&Keycode::D) {
            let local_left =
                cgmath::EuclideanSpace::to_vec(camera.forward).cross(Vector3::unit_y());
            camera.position += local_left * CONFIG.move_speed * dt;
        }

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            object.draw(camera.get_view_matrix());
        }

        window.gl_swap_window();
    }
}
