use cgmath::prelude::*;
use game_lib::types::*;
use game_lib::*;
use rand::prelude::*;
use sdl2::event::Event;

fn main() {
  game_lib::run(
    Config {
      scale: 4,
      screen_size: V2U::new(160, 120),
      font_color: sdl2::pixels::Color::WHITE,
      background_color: sdl2::pixels::Color::BLACK,
      palette: [
        ('#', sdl2::pixels::Color::RED),
        ('_', sdl2::pixels::Color::WHITE),
        ('/', sdl2::pixels::Color::GREEN),
        ('^', sdl2::pixels::Color::YELLOW),
        ('&', sdl2::pixels::Color::RGB(255, 127, 0)),
      ]
      .iter()
      .cloned()
      .collect(),
    },
    State::new(),
    init,
    update,
    render,
    handle_event,
  );
}

pub fn init(gcontext: &mut GContext) {
  #[rustfmt::skip]
  gcontext.add_sprite(
    "ball",
    vec![
      "  ##  ",
      " #_## ",
      "#_####",
      "######",
      " #### ",
      "  ##  ",
    ],
  );
  #[rustfmt::skip]
  gcontext.add_sprite(
    "cherry",
    vec![
      "  /     ",
      " / /    ",
      " /  /   ",
      " /   ## ",
      " ## #_##",
      "#_## ###",
      "#### ## ",
      " ##     ",
    ],
  );
  #[rustfmt::skip]
  gcontext.add_sprite(
    "coin",
    vec![
      "  &&&&  ",
      " &&^^^& ",
      "&&^^&^^&",
      "&&^^&^^&",
      "&&^^&^^&",
      "&&^^&^^&",
      " &&^^^& ",
      "  &&&&  ",
    ],
  );
}

pub struct State {
  rng: rand::prelude::ThreadRng,

  score: i32,
}

#[derive(Copy, Clone, PartialEq)]
pub enum ControlScheme {
  Hold,
  Toggle,
}

impl State {
  pub fn new() -> State {
    let mut rng = rand::thread_rng();
    State { rng, score: 0 }
  }
}

pub fn update(state: &mut State, key_status: &KeyStatus, game_tick_counter: u32) {}

pub fn render(gcontext: &mut GContext, state: &State) {
  gcontext.draw_text(10, 10, "cherry rogue");

  gcontext.draw_sprite(10, 20, "ball");
  gcontext.draw_sprite(20, 20, "cherry");
  gcontext.draw_sprite(30, 20, "coin");
}

pub fn handle_event(state: &mut State, event: &sdl2::event::Event) {
  match *event {
    Event::KeyDown {
      keycode: Some(keycode),
      ..
    } => handle_keypress(state, keycode),
    _ => (),
  }
}

fn handle_keypress(state: &mut State, keycode: sdl2::keyboard::Keycode) {}
