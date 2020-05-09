#![allow(dead_code)]

extern crate cgmath;
extern crate gl;
extern crate sdl2;

use cgmath::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::path::Path;

mod geom;
mod keys;
mod material;
mod math;
mod mesh;
mod object;
mod physics;
mod player;
mod render;
mod shader;
mod texture;
mod time;
mod ui;
mod ui_batch;

fn main() {
    let sdl_context = sdl2::init().unwrap();

    let mut renderer = render::Renderer::init(&sdl_context);
    let mut time = time::Time::new(&sdl_context);
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut keys = keys::Keys::new();
    let mut player = player::Player::new();

    let (tobj_models, tobj_mats) = match tobj::load_obj(&Path::new("assets/test_pb.obj")) {
        Ok(cube_obj) => cube_obj,
        Err(e) => panic!("Error during loading models: {}", e),
    };

    let (vertex_data, index_data) = mesh::read_vertex_array(&tobj_models[0].mesh);
    let material = material::Material::new(
        vertex_data,
        index_data,
        &tobj_mats[0],
        render::get_projection_matrix(),
    );

    let mesh = mesh::Mesh::new(&tobj_models[0].mesh);
    let mut object1 = object::Object::new(&material, &mesh);
    object1.translate(Vector3::new(10.0, 0.0, -10.0));

    let mut object2 = object::Object::new(&material, &mesh);
    object2.translate(Vector3::new(70.0, 0.0, 0.0));

    let objects = vec![object1, object2];

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

        keys.tick(
            event_pump
                .keyboard_state()
                .pressed_scancodes()
                .filter_map(Keycode::from_scancode)
                .collect(),
        );

        player.tick(&keys, (mouse_x, mouse_y), &objects, dt);

        renderer.render(&objects, player.get_view_matrix());
    }
}
