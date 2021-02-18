use std::collections::HashSet;

pub struct KeyStatus {
  key_status: HashSet<sdl2::keyboard::Keycode>,
}

impl KeyStatus {
  pub fn new() -> KeyStatus {
    KeyStatus {
      key_status: HashSet::new(),
    }
  }

  pub fn set_key_pressed(&mut self, keycode: sdl2::keyboard::Keycode, pressed: bool) {
    if pressed {
      self.key_status.insert(keycode);
    } else {
      self.key_status.remove(&keycode);
    }
  }

  pub fn is_key_pressed(&self, keycode: sdl2::keyboard::Keycode) -> bool {
    self.key_status.contains(&keycode)
  }
}
