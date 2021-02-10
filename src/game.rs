use crate::game_lib::types::*;
use crate::game_lib::*;
use cgmath::prelude::*;
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
  #[rustfmt::skip]
  gcontext.add_sprite(
    "coin",
    vec![
      "  ####  ",
      " ##   # ",
      "##  #  #",
      "##  #  #",
      "##  #  #",
      "##  #  #",
      " ##   # ",
      "  ####  ",
    ],
  );
}

#[derive(Copy, Clone)]
enum ItemType {
  Cherry,
  Coin,
}

#[derive(Copy, Clone)]
struct Item {
  item_type: ItemType,
  velocity: V2F,
  pos: V2F,
}

pub struct State {
  rng: rand::prelude::ThreadRng,

  paddle_pos: P2F,
  paddle_dir: f32,

  ball_pos: P2F,
  ball_dir: V2F,

  items: Vec<Item>,
  score: i32,
}

const BALL_SPEED: f32 = 1.4;
const BALL_SIZE: u32 = 6;
const ITEM_SPEED: f32 = 3.0;
const ITEM_SIZE: u32 = 8;
const PADDLE_SPEED: i32 = 2;
const PADDLE_SIZE: V2U = V2U::new(3, 10);
const D_MAX: f32 = BALL_SIZE as f32 / 2.0 + PADDLE_SIZE.y as f32 / 2.0;

impl State {
  pub fn new() -> State {
    State {
      rng: rand::thread_rng(),

      paddle_pos: P2F::new(1.0, 1.0),
      paddle_dir: 0.0,

      ball_pos: P2F::new(0.0, 20.0),
      ball_dir: V2F::new(1.0, 1.0).normalize() * BALL_SPEED,

      items: Vec::new(),
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
  state.ball_pos += state.ball_dir;
  if state.ball_pos.x < 0.0 {
    // TODO game over
    state = State::new();
  }
  if state.ball_pos.x + BALL_SIZE as f32 >= 84.0 {
    state.ball_dir.x *= -1.0;
  }
  if state.ball_pos.y < 0.0 {
    state.ball_dir.y *= -1.0;
  }
  if state.ball_pos.y + BALL_SIZE as f32 >= 48.0 {
    state.ball_dir.y *= -1.0;
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
    let paddle_center = state.paddle_pos.x + PADDLE_SIZE.x as f32 / 2.0;
    let ball_center = state.ball_pos.x + BALL_SIZE as f32 / 2.0;
    let d = ball_center - paddle_center;
    state.ball_dir = (state.ball_dir + V2F::new(2.0, d / D_MAX)).normalize() * BALL_SPEED;
  }

  // update items
  if game_tick_counter % 20 == 0 {
    if state.items.len() < 3 {
      state.items.push(Item {
        item_type: if state.rng.gen_range(0..10) == 0 {
          ItemType::Coin
        } else {
          ItemType::Cherry
        },
        velocity: V2F::new(
          state.rng.gen_range(-0.5..0.5),
          state.rng.gen_range(0.0..0.5) + 0.5,
        )
        .normalize()
          * (ITEM_SPEED * state.rng.gen_range(0.5..1.5)),
        pos: V2F::new(state.rng.gen_range(10..80) as f32, -5.0),
      });
    }
  }
  for item in &mut state.items {
    item.pos += item.velocity;
    item.velocity *= 0.9;
  }
  state.score += state
    .items
    .iter()
    .map(|&item| {
      if ball_rect.intersects(&Rect::new(
        item.pos.x,
        item.pos.y,
        ITEM_SIZE as f32,
        ITEM_SIZE as f32,
      )) {
        match item.item_type {
          ItemType::Cherry => 1,
          ItemType::Coin => 5,
        }
      } else {
        0
      }
    })
    .sum::<i32>();
  state.items = state
    .items
    .iter()
    .filter(|&item| {
      !ball_rect.intersects(&Rect::new(
        item.pos.x,
        item.pos.y,
        ITEM_SIZE as f32,
        ITEM_SIZE as f32,
      )) && item.pos.y < 50.0
        && PADDLE_SIZE.x as f32 + 2.0 < item.pos.x
        && item.pos.x < 84.0
    })
    .map(|x| *x)
    .collect();

  state
}

pub fn render(gcontext: &mut GContext, state: &State) {
  for fruit in &state.items {
    let item_name = match fruit.item_type {
      ItemType::Cherry => "cherry",
      ItemType::Coin => "coin",
    };
    gcontext.draw_sprite(fruit.pos.x as i32, fruit.pos.y as i32, item_name);
  }
  gcontext.draw_sprite(state.ball_pos.x as i32, state.ball_pos.y as i32, "ball");
  gcontext.draw_dark_rect(
    state.paddle_pos.x as i32,
    state.paddle_pos.y as i32,
    PADDLE_SIZE.x,
    PADDLE_SIZE.y,
  );
  gcontext.draw_text(84 - 5 * 3, 1, &state.score.to_string());
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
