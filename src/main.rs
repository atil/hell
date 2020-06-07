// #![allow(dead_code)]

extern crate cgmath;
extern crate gl;
extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

mod assets;
mod geom;
mod keys;
mod light;
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

    let (meshes, materials) = assets::load_obj("assets/test_lighting.obj");

    let mut objects = Vec::new();
    for (mesh, mat) in meshes.iter().zip(materials.iter()) {
        objects.push(object::Object::new(&mesh, &mat));
    }

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
