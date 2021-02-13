mod game_lib;

use crate::game_lib::types::*;
use crate::game_lib::*;
use cgmath::prelude::*;
use rand::prelude::*;
use sdl2::event::Event;

fn main() {
  let control_mode = get_arg("control")
    .map(|s| match s.as_str() {
      "toggle" => Some(ControlScheme::Toggle),
      "hold" => Some(ControlScheme::Hold),
      _ => None,
    })
    .flatten()
    .unwrap_or(ControlScheme::Toggle);

  let scale = get_arg("scale")
    .map(|s| s.parse().ok())
    .flatten()
    .unwrap_or(16);

  game_lib::run(
    scale,
    State::new(control_mode),
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
enum Age {
  Finite { remaining: i32 },
  Infinite,
}

impl Age {
  pub fn is_alive(&self) -> bool {
    match self {
      Age::Infinite => true,
      Age::Finite { remaining } => *remaining > 0,
    }
  }
}

#[derive(Copy, Clone)]
struct Item {
  item_type: ItemType,
  velocity: V2F,
  pos: V2F,
  age: Age,
}

impl Item {
  pub fn rect(&self) -> Rect {
    Rect::new(self.pos.x, self.pos.y, ITEM_SIZE as f32, ITEM_SIZE as f32)
  }
}

pub struct State {
  rng: rand::prelude::ThreadRng,

  control_scheme: ControlScheme,

  paddle_pos: P2F,
  paddle_dir: f32,

  ball_pos: P2F,
  ball_dir: V2F,

  rings: Vec<(f32, V2F)>,

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
const COIN_LIFETIME: u32 = 100;
const COIN_FLASH_THRESHOLD: u32 = 20;

#[derive(PartialEq)]
pub enum ControlScheme {
  Hold,
  Toggle,
}

impl State {
  pub fn new(control_scheme: ControlScheme) -> State {
    let mut rng = rand::thread_rng();
    let ball_dir_y = rng.gen_range(0.5..1.0);
    State {
      rng,

      control_scheme,

      paddle_pos: P2F::new(1.0, 1.0),
      paddle_dir: 0.0,

      ball_pos: P2F::new(0.0, 20.0),
      ball_dir: V2F::new(1.0, ball_dir_y).normalize() * BALL_SPEED,

      rings: Vec::new(),

      items: Vec::new(),
      score: 0,
    }
  }
}

pub fn update(state: State, key_status: &KeyStatus, game_tick_counter: i32) -> State {
  let mut state = state;
  // update paddle
  if state.control_scheme == ControlScheme::Hold {
    if key_status.is_key_pressed(sdl2::keyboard::Keycode::W) {
      state.paddle_dir = -PADDLE_SPEED as f32;
    } else if key_status.is_key_pressed(sdl2::keyboard::Keycode::S) {
      state.paddle_dir = PADDLE_SPEED as f32;
    } else {
      state.paddle_dir = 0.0;
    }
  }
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
    state = State::new(state.control_scheme);
  }
  if state.ball_pos.x + BALL_SIZE as f32 >= 84.0 {
    state.ball_pos.x = 84.0 - BALL_SIZE as f32;
    state.ball_dir.x *= -1.0;
  }
  if state.ball_pos.y < 0.0 {
    state.ball_pos.y = 0.0;
    state.ball_dir.y *= -1.0;
  }
  if state.ball_pos.y + BALL_SIZE as f32 >= 48.0 {
    state.ball_pos.y = 48.0 - BALL_SIZE as f32;
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
    let paddle_center = state.paddle_pos.y + PADDLE_SIZE.y as f32 / 2.0;
    let ball_center = state.ball_pos.y + BALL_SIZE as f32 / 2.0;
    let d = (ball_center - paddle_center) / D_MAX;
    state.ball_dir = V2F::new(
      state.ball_dir.x * -0.5 + 0.5 * d.abs() + 0.5,
      state.ball_dir.y * 0.5 + d * 2.0,
    )
    .normalize()
      * BALL_SPEED;
    state.ball_pos.x = PADDLE_SIZE.x as f32 + 1.0;
  }

  // update rings
  for ring in &mut state.rings {
    ring.0 += 1.0;
  }
  state.rings = state
    .rings
    .iter()
    .filter(|(ring_size, _)| *ring_size < 10.0)
    .map(|x| *x)
    .collect();

  // update items
  if game_tick_counter % 20 == 0 {
    if state.items.len() < 3 {
      let (item_type, age) = if state.rng.gen_range(0..10) == 0 {
        (
          ItemType::Coin,
          Age::Finite {
            remaining: COIN_LIFETIME as i32,
          },
        )
      } else {
        (ItemType::Cherry, Age::Infinite)
      };
      state.items.push(Item {
        item_type,
        velocity: V2F::new(
          state.rng.gen_range(-0.5..0.5),
          state.rng.gen_range(0.0..0.5) + 0.5,
        )
        .normalize()
          * (ITEM_SPEED * state.rng.gen_range(0.5..1.5)),
        pos: V2F::new(state.rng.gen_range(10..80) as f32, -5.0),
        age,
      });
    }
  }
  for item in &mut state.items {
    item.pos += item.velocity;
    item.velocity *= 0.9;
    match &mut item.age {
      Age::Finite { remaining } => *remaining -= 1,
      _ => (),
    }
  }
  let touched_items = state.items.iter().filter(|&item| {
    ball_rect.intersects(&item.rect())
      && item.pos.y < 50.0
      && PADDLE_SIZE.x as f32 + 2.0 < item.pos.x
      && item.pos.x < 84.0
  });
  state.score += touched_items
    .clone()
    .map(|&item| match item.item_type {
      ItemType::Cherry => 1,
      ItemType::Coin => 5,
    })
    .sum::<i32>();
  let mut new_rings: Vec<(f32, V2F)> = touched_items
    .map(|&item| {
      (
        0.0,
        item.pos + V2F::new(ITEM_SIZE as f32 / 2.0, ITEM_SIZE as f32 / 2.0),
      )
    })
    .collect();
  state.rings.append(&mut new_rings);
  state.items = state
    .items
    .iter()
    .filter(|&item| {
      !ball_rect.intersects(&item.rect())
        && item.pos.y < 50.0
        && PADDLE_SIZE.x as f32 + 2.0 < item.pos.x
        && item.pos.x < 84.0
        && item.age.is_alive()
    })
    .map(|x| *x)
    .collect();

  state
}

pub fn render(gcontext: &mut GContext, state: &State) {
  for (ring_size, ring_pos) in &state.rings {
    let m = (*ring_size / 4.0) as i32 + 1;
    gcontext.draw_circle(
      ring_pos.x as i32,
      ring_pos.y as i32,
      *ring_size as i32,
      m,
      crate::game_lib::DARK_COLOR,
    );
  }
  for item in &state.items {
    let item_name = match item.item_type {
      ItemType::Cherry => "cherry",
      ItemType::Coin => "coin",
    };
    let should_draw = match item.age {
      Age::Infinite => true,
      Age::Finite { remaining } => remaining > COIN_FLASH_THRESHOLD as i32 || remaining % 2 == 0,
    };
    if should_draw {
      gcontext.draw_sprite(item.pos.x as i32, item.pos.y as i32, item_name);
    }
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
  if state.control_scheme == ControlScheme::Toggle {
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
  }
  state
}
