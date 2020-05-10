use sdl2::keyboard::*;

pub struct Keys {
    prev_keys: Vec<Keycode>,
    pressed_this_frame: Vec<Keycode>,
    released_this_frame: Vec<Keycode>,
}

impl Keys {
    pub fn new() -> Keys {
        Keys {
            prev_keys: Vec::new(),
            pressed_this_frame: Vec::new(),
            released_this_frame: Vec::new(),
        }
    }

    pub fn tick(&mut self, keys: Vec<Keycode>) {
        self.pressed_this_frame.clear();

        // TODO: We don't need to clone here. Fix the reference problem
        self.pressed_this_frame = keys
            .clone()
            .into_iter()
            .filter(|key| !self.prev_keys.contains(key))
            .collect::<Vec<Keycode>>();

        self.released_this_frame = self
            .prev_keys
            .clone()
            .into_iter()
            .filter(|key| !keys.contains(key))
            .collect::<Vec<Keycode>>();

        self.prev_keys = keys;
    }

    pub fn get_key(&self, key: Keycode) -> bool {
        self.prev_keys.contains(&key)
    }

    pub fn get_key_down(&self, key: Keycode) -> bool {
        self.pressed_this_frame.contains(&key)
    }

    pub fn get_key_up(&self, key: Keycode) -> bool {
        self.released_this_frame.contains(&key)
    }
}
