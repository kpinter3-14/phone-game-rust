use crate::game_lib::types::*;
use crate::game_lib::*;
use rand::prelude::*;
use sdl2::event::Event;

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
  rng: rand::prelude::ThreadRng,

  paddle_pos: P2F,
  paddle_dir: f32,

  ball_pos: P2F,
  ball_dir: V2F,

  fruits: Vec<Rect>,
  score: i32,
}

const BALL_SPEED: f32 = 1.0;
const BALL_SIZE: u32 = 6;
const FRUIT_SPEED: i32 = 2;
const FRUIT_SIZE: u32 = 8;
const PADDLE_SPEED: i32 = 2;
const PADDLE_SIZE: V2U = V2U::new(3, 10);

impl State {
  pub fn new() -> State {
    State {
      rng: rand::thread_rng(),

      paddle_pos: P2F::new(1.0, 1.0),
      paddle_dir: 0.0,

      ball_pos: P2F::new(0.0, 20.0),
      ball_dir: V2F::new(BALL_SPEED, BALL_SPEED),

      fruits: Vec::new(),
      score: 0,
    }
  }
}

pub fn update(state: State, game_tick_counter: i32) -> State {
  let mut state = state;
  // update paddle
  state.paddle_pos.y += state.paddle_dir;
  if state.paddle_pos.y < 1.0 {
    state.paddle_pos.y = 1.0;
    state.paddle_dir = 0.0;
  }
  if state.paddle_pos.y + PADDLE_SIZE.y as f32 > 47.0 {
    state.paddle_pos.y = 48.0 - 1.0 - PADDLE_SIZE.y as f32;
    state.paddle_dir = 0.0;
  }

  // update ball
  state.ball_pos.x += state.ball_dir.x;
  state.ball_pos.y += state.ball_dir.y;
  if state.ball_pos.x < 0.0 {
    // TODO game over
    state = State::new();
  }
  if state.ball_pos.x + BALL_SIZE as f32 >= 84.0 {
    state.ball_dir.x = -BALL_SPEED;
  }
  if state.ball_pos.y < 0.0 {
    state.ball_dir.y = BALL_SPEED;
  }
  if state.ball_pos.y + BALL_SIZE as f32 >= 48.0 {
    state.ball_dir.y = -BALL_SPEED;
  }
  let ball_rect = Rect::new(
    state.ball_pos.x,
    state.ball_pos.y,
    BALL_SIZE as f32,
    BALL_SIZE as f32,
  );
  let paddle_rect = Rect::new(
    state.paddle_pos.x,
    state.paddle_pos.y,
    PADDLE_SIZE.x as f32,
    PADDLE_SIZE.y as f32,
  );
  if paddle_rect.intersects(&ball_rect) {
    state.ball_dir.x = BALL_SPEED;
  }

  // update fruits
  if game_tick_counter % 14 == 0 {
    state.fruits.push(Rect::new(
      state.rng.gen_range(10..80) as f32,
      -5.0,
      FRUIT_SIZE as f32,
      FRUIT_SIZE as f32,
    ));
  }
  for fruit in &mut state.fruits {
    fruit.y += FRUIT_SPEED as f32;
  }
  state.score += state
    .fruits
    .iter()
    .filter(|&fruit| ball_rect.intersects(fruit))
    .count() as i32;
  state.fruits = state
    .fruits
    .iter()
    .filter(|&fruit| !ball_rect.intersects(fruit) && fruit.y < 50.0)
    .map(|x| *x)
    .collect();

  state
}

pub fn render(gcontext: &mut GContext, state: &State) {
  for fruit in &state.fruits {
    gcontext.draw_sprite(fruit.x as i32, fruit.y as i32, "cherry");
  }
  gcontext.draw_sprite(state.ball_pos.x as i32, state.ball_pos.y as i32, "ball");
  gcontext.draw_dark_rect(
    state.paddle_pos.x as i32,
    state.paddle_pos.y as i32,
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
    _ => state,
  }
}

fn handle_keypress(state: State, keycode: sdl2::keyboard::Keycode) -> State {
  let mut state = state;
  match keycode {
    sdl2::keyboard::Keycode::W => {
      if state.paddle_dir >= 0.0 {
        state.paddle_dir = -PADDLE_SPEED as f32;
      } else {
        state.paddle_dir = 0.0;
      }
    }
    sdl2::keyboard::Keycode::S => {
      if state.paddle_dir <= 0.0 {
        state.paddle_dir = PADDLE_SPEED as f32;
      } else {
        state.paddle_dir = 0.0;
      }
    }
    _ => {}
  }
  state
}
