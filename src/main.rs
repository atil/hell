// #![allow(dead_code)]

extern crate cgmath;
extern crate gl;
extern crate sdl2;
extern crate serde;
extern crate serde_json;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

mod assets;
mod geom;
mod keys;
mod math;
mod mesh;
mod physics;
mod player;
mod render;
mod static_object;
mod time;

// the problem is that we instantiate the meshes here, and the instance is dropped at the
// end of the for loop. the problem is that, the static_objects keep references to those meshes.
// actually we wanted to have a sort of "repository" of assets (meshes and materials)
//
// "one obj file -> one prefab"
// struct prefab: vec<mesh>, vec<material>
// prefab is the obj file transform into the usable stuff
//
// vec<prefab> is the repository
// repository: readonly data that's transformed from the persisted data
//
// staticobject -> ctor(&prefab, transform)
// rendering: calling the same draw call, with the same vbo, but different uniforms

fn main() {
    let sdl_context = sdl2::init().unwrap();

    let mut renderer = render::renderer::Renderer::init(&sdl_context);
    let mut ui = render::ui::Ui::init();
    let mut time = time::Time::new(&sdl_context);
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut keys = keys::Keys::new();
    let mut player = player::Player::new();

    let prefabs = assets::load_prefabs("assets/prefabs.json");
    let static_objects = assets::create_static_objects("assets/scene.json", &prefabs);

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

        player.tick(&keys, (mouse_x, mouse_y), &static_objects, dt);

        unsafe {
            renderer.render(&static_objects, player.get_view_matrix());
            ui.draw(&player);
        }

        renderer.finish_render();
    }
}
