extern crate cgmath;
extern crate gl;
extern crate sdl2;

use cgmath::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::path::Path;

mod camera;
mod material;
mod mesh;
mod object;
mod render;
mod shader;
mod time;

fn main() {
    let sdl_context = sdl2::init().unwrap();

    // This GLContext needs to be alive. Renaming it to "_" makes the compiler
    // drop it immediately
    let (window, _gl_context) = render::init(&sdl_context);

    let mut time = time::Time::new(&sdl_context);
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut camera = camera::Camera::new();

    let (tobj_models, tobj_mats) = match tobj::load_obj(&Path::new("assets/cube.obj")) {
        Ok(cube_obj) => cube_obj,
        Err(e) => panic!("Error during loading models: {}", e),
    };

    let (vertex_data, index_data) = mesh::read_vertex_data(&tobj_models[0].mesh);
    let material = material::Material::new(
        &vertex_data,
        index_data,
        &tobj_mats[0],
        render::get_projection_matrix(),
    );

    let mut object = object::Object::new(&material);
    object.translate(Vector3::new(0.0, -1.0, -10.0));
    object.rotate(Vector3::unit_y(), 30.0);

    let objects = vec![object];

    'main: loop {
        let (mut mouse_x, mut mouse_y) = (0.0, 0.0);

        let dt = time.tick();

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

        let keys: Vec<Keycode> = event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        camera.tick(dt, keys, (mouse_x, mouse_y));

        render::render(&window, &objects, camera.get_view_matrix());
    }
}
