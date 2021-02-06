use crate::game_lib::types::*;
use crate::game_lib::*;
use rand::prelude::*;
use sdl2::event::Event;
use sdl2::rect::Rect;

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
      "  #     ",
      " # #    ",
      " #  #   ",
      " #   ## ",
      " ## #_##",
      "#_## ###",
      "#### ## ",
      " ##     ",
    ],
  );
}

pub struct State {
  pub rng: rand::prelude::ThreadRng,

  pub paddle_rect: sdl2::rect::Rect,
  pub paddle_dir: V2I,

  pub ball_rect: sdl2::rect::Rect,
  pub ball_dir: V2I,

  pub fruits: Vec<sdl2::rect::Rect>,
  pub score: i32,
}

const BALL_SPEED: i32 = 1;
const BALL_SIZE: u32 = 6;
const FRUIT_SPEED: i32 = 2;
const FRUIT_SIZE: u32 = 8;
const PADDLE_SPEED: i32 = 2;
const PADDLE_SIZE: V2U = V2U::new(3, 10);

impl State {
  pub fn new() -> State {
    State {
      rng: rand::thread_rng(),

      paddle_rect: Rect::new(1, 1, PADDLE_SIZE.x, PADDLE_SIZE.y),
      paddle_dir: V2I::new(0, 0),

      ball_rect: Rect::new(0, 20, BALL_SIZE, BALL_SIZE),
      ball_dir: V2I::new(BALL_SPEED, BALL_SPEED),

      fruits: Vec::new(),
      score: 0,
    }
  }
}

pub fn update(state: State, game_tick_counter: i32) -> State {
  let mut state = state;
  // update paddle
  state.paddle_rect.y += state.paddle_dir.y;
  if state.paddle_rect.y < 1 {
    state.paddle_rect.y = 1;
    state.paddle_dir.y = 0;
  }
  if state.paddle_rect.y + PADDLE_SIZE.y as i32 > 47 {
    state.paddle_rect.y = 48 - 1 - PADDLE_SIZE.y as i32;
    state.paddle_dir.y = 0;
  }

  // update ball
  state.ball_rect.x += state.ball_dir.x;
  state.ball_rect.y += state.ball_dir.y;
  if state.ball_rect.x < 0 {
    // TODO game over
  }
  if state.ball_rect.x + BALL_SIZE as i32 >= 84 {
    state.ball_dir.x = -BALL_SPEED;
  }
  if state.ball_rect.y < 0 {
    state.ball_dir.y = BALL_SPEED;
  }
  if state.ball_rect.y + BALL_SIZE as i32 >= 48 {
    state.ball_dir.y = -BALL_SPEED;
  }
  let ball_rect = Rect::new(state.ball_rect.x, state.ball_rect.y, BALL_SIZE, BALL_SIZE);
  if state.paddle_rect.intersection(ball_rect).is_some() {
    state.ball_dir.x = BALL_SPEED;
  }

  // update fruits
  if game_tick_counter % 14 == 0 {
    state.fruits.push(Rect::new(
      state.rng.gen_range(10..80),
      -5,
      FRUIT_SIZE,
      FRUIT_SIZE,
    ));
  }
  for fruit in &mut state.fruits {
    fruit.y += FRUIT_SPEED;
  }
  state.score += state
    .fruits
    .iter()
    .filter(|&fruit| ball_rect.intersection(*fruit).is_some())
    .count() as i32;
  state.fruits = state
    .fruits
    .iter()
    .filter(|&fruit| ball_rect.intersection(*fruit).is_none() && fruit.y < 50)
    .map(|x| *x)
    .collect();

  state
}

pub fn render(gcontext: &mut GContext, state: &State) {
  for fruit in &state.fruits {
    gcontext.draw_sprite(fruit.x, fruit.y, "cherry");
  }
  gcontext.draw_sprite(state.ball_rect.x, state.ball_rect.y, "ball");
  gcontext.draw_dark_rect(
    state.paddle_rect.x,
    state.paddle_rect.y,
    PADDLE_SIZE.x,
    PADDLE_SIZE.y,
  );
  gcontext.draw_text(74, 1, &state.score.to_string());
}

pub fn handle_event(state: State, event: &sdl2::event::Event) -> State {
  match *event {
    Event::KeyDown {
      keycode: Some(keycode),
      ..
    } => handle_keypress(state, keycode),
    _ => state
  }
}

fn handle_keypress(state: State, keycode: sdl2::keyboard::Keycode) -> State {
  let mut state = state;
  match keycode {
    sdl2::keyboard::Keycode::W => {
      if state.paddle_dir.y >= 0 {
        state.paddle_dir.y = -PADDLE_SPEED;
      } else {
        state.paddle_dir.y = 0;
      }
    }
    sdl2::keyboard::Keycode::S => {
      if state.paddle_dir.y <= 0 {
        state.paddle_dir.y = PADDLE_SPEED;
      } else {
        state.paddle_dir.y = 0;
      }
    }
    _ => {}
  }
  state
}
