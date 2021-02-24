use game_lib::incmap::*;
use game_lib::types::*;
use game_lib::*;
use rand::prelude::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

fn main() {
  let config = qvga_config(2);

  let mut state = State::new();
  game_lib::run(config, state, init, update, render, handle_event);
}

struct State {}

impl State {
  fn new() -> State {
    State {}
  }
}

fn init(gcontext: &mut GContext) {}

fn update(state: &mut State, key_status: &KeyStatus, game_tick_counter: u32) {}

fn render(gcontext: &mut GContext, state: &State) {
  gcontext.draw_circle(20, 20, 15, 3, sdl2::pixels::Color::CYAN);
  gcontext.draw_circle(20, 60, 15, 1, sdl2::pixels::Color::CYAN);
  gcontext.draw_text_box(
    HorPos::Center,
    VertPos::Center,
    &vec!["collect resources", "or else"],
    sdl2::pixels::Color::RED,
  );
}

fn handle_event(state: &mut State, event: &sdl2::event::Event) {
  match *event {
    Event::KeyDown {
      keycode: Some(keycode),
      ..
    } => handle_keypress(state, keycode),
    _ => (),
  }
}

fn handle_keypress(state: &mut State, keycode: Keycode) {}
