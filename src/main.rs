#![allow(dead_code)]

extern crate cgmath;
extern crate gl;
extern crate sdl2;

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
    let mut ui = ui::Ui::init();
    let mut time = time::Time::new(&sdl_context);
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut keys = keys::Keys::new();
    let mut player = player::Player::new();

    // Load obj files
    let (tobj_models, tobj_mats) = match tobj::load_obj(&Path::new("assets/test_parkour.obj")) {
        Ok(cube_obj) => cube_obj,
        Err(e) => panic!("Error during loading models: {}", e),
    };

    // Create meshes
    let mesh1 = mesh::Mesh::new(&tobj_models[0].mesh);
    let mesh2 = mesh::Mesh::new(&tobj_models[1].mesh);

    // Create materials
    let (vertex_data1, index_data1) = mesh::read_vertex_array(&tobj_models[0].mesh);
    let material1 = material::Material::new(
        vertex_data1,
        index_data1,
        tobj_mats[0].diffuse_texture.as_str(),
        render::get_projection_matrix(),
    );
    let (vertex_data2, index_data2) = mesh::read_vertex_array(&tobj_models[1].mesh);
    let material2 = material::Material::new(
        vertex_data2,
        index_data2,
        tobj_mats[1].diffuse_texture.as_str(),
        render::get_projection_matrix(),
    );

    // Create objects
    let object1 = object::Object::new(&material1, &mesh1);
    let object2 = object::Object::new(&material2, &mesh2);

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

        player.tick(&keys, (mouse_x, mouse_y), &objects, &mut ui, dt);

        unsafe {
            renderer.render(&objects, player.get_view_matrix());
            ui.draw();
        }

        renderer.finish_render();
    }
}
