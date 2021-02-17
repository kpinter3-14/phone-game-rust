use cgmath::prelude::*;
use game_lib::incmap::*;
use game_lib::types::*;
use game_lib::*;
use rand::prelude::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
mod assets;

fn main() {
  let config = Config {
    scale: 4,
    screen_size: V2U::new(160, 120),
    font_color: sdl2::pixels::Color::WHITE,
    background_color: sdl2::pixels::Color::BLACK,
    palette: [
      ('#', sdl2::pixels::Color::RED),
      ('/', sdl2::pixels::Color::GREEN),
      ('^', sdl2::pixels::Color::YELLOW),
      ('b', sdl2::pixels::Color::BLUE),
      ('c', sdl2::pixels::Color::CYAN),
      ('&', sdl2::pixels::Color::RGB(255, 127, 0)), // orange
      ('_', sdl2::pixels::Color::WHITE),
      ('o', sdl2::pixels::Color::GREY),
      ('B', sdl2::pixels::Color::BLACK),
    ]
    .iter()
    .cloned()
    .collect(),
  };

  let mut state = State::new();
  state.entities.insert(Entity {
    pos: V2I::new(2, 3),
    item_type: ItemType::Cherry,
  });
  state.entities.insert(Entity {
    pos: V2I::new(3, 4),
    item_type: ItemType::Coin,
  });
  add_room(2, 3, 4, 6, &mut state.map_array);
  add_room(6, 4, 3, 1, &mut state.map_array);
  add_room(9, 2, 5, 5, &mut state.map_array);

  game_lib::run(config, state, init, update, render, handle_event);
}

fn init(gcontext: &mut GContext) {
  crate::assets::load_sprites(gcontext);
}

#[derive(Copy, Clone, PartialEq)]
enum Tile {
  Void,
  Wall,
  Floor,
  Door { is_open: bool },
}

#[derive(Copy, Clone, PartialEq)]
struct Entity {
  pos: V2I,
  item_type: ItemType,
}

#[derive(Copy, Clone, PartialEq)]
enum ItemType {
  Cherry,
  Coin,
}

const TILE_SIZE: u32 = 8;
const MAP_SIZE: V2U = V2U::new(20, 14);
type MapArray = [[Tile; MAP_SIZE.y as usize]; MAP_SIZE.x as usize];

fn add_room(x: i32, y: i32, w: i32, h: i32, map_array: &mut MapArray) {
  for x_ix in x..x + w {
    for y_ix in y..y + h {
      map_array[x_ix as usize][y_ix as usize] = Tile::Floor;
    }
  }

  for x_ix in x - 1..x + w + 1 {
    let top_row_tile: &mut Tile = &mut map_array[x_ix as usize][(y - 1) as usize];
    if *top_row_tile != Tile::Floor {
      *top_row_tile = Tile::Wall;
    }
    let bottom_row_tile: &mut Tile = &mut map_array[x_ix as usize][(y + h) as usize];
    if *bottom_row_tile != Tile::Floor {
      *bottom_row_tile = Tile::Wall;
    }
  }

  for y_ix in y - 1..y + h + 1 {
    let left_col_tile: &mut Tile = &mut map_array[(x - 1) as usize][y_ix as usize];
    if *left_col_tile != Tile::Floor {
      *left_col_tile = Tile::Wall;
    }
    let right_col_tile: &mut Tile = &mut map_array[(x + w) as usize][y_ix as usize];
    if *right_col_tile != Tile::Floor {
      *right_col_tile = Tile::Wall;
    }
  }
}

struct State {
  rng: rand::prelude::ThreadRng,

  char_pos: V2I,
  map_array: MapArray,
  entities: IncMap<Entity>,

  score: i32,
}

impl State {
  fn new() -> State {
    let mut rng = rand::thread_rng();
    let mut map_array = [[Tile::Void; MAP_SIZE.y as usize]; MAP_SIZE.x as usize];
    State {
      rng,
      char_pos: V2I::new(10, 5),
      entities: IncMap::new(),
      map_array,
      score: 0,
    }
  }
}

fn update(state: &mut State, key_status: &KeyStatus, game_tick_counter: u32) {}

fn render(gcontext: &mut GContext, state: &State) {
  gcontext.draw_text(
    1,
    (gcontext.get_config().screen_size.y - game_lib::FONT_HEIGHT) as i32,
    "cherry rogue",
  );

  for x in 0..MAP_SIZE.x {
    for y in 0..MAP_SIZE.y {
      let tile_name = match state.map_array[x as usize][y as usize] {
        Tile::Void => None,
        Tile::Wall => Some("wall"),
        Tile::Floor => Some("floor"),
        Tile::Door { is_open } => Some("door"),
      };
      tile_name.map(|tile_name| {
        gcontext.draw_sprite(
          x as i32 * TILE_SIZE as i32,
          y as i32 * TILE_SIZE as i32,
          tile_name,
        )
      });
    }
  }

  for (_, entity) in &state.entities {
    let entity_name = match entity.item_type {
      ItemType::Cherry => "cherry",
      ItemType::Coin => "coin",
    };
    gcontext.draw_sprite(
      entity.pos.x * TILE_SIZE as i32,
      entity.pos.y * TILE_SIZE as i32,
      entity_name,
    );
  }
  gcontext.draw_sprite(
    state.char_pos.x * TILE_SIZE as i32,
    state.char_pos.y * TILE_SIZE as i32,
    "ball",
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

fn handle_keypress(state: &mut State, keycode: Keycode) {
  let move_dir = match keycode {
    Keycode::Up => V2I::new(0, -1),
    Keycode::Left => V2I::new(-1, 0),
    Keycode::Down => V2I::new(0, 1),
    Keycode::Right => V2I::new(1, 0),
    _ => V2I::new(0, 0),
  };
  let next_pos = state.char_pos + move_dir;
  if state.map_array[next_pos.x as usize][next_pos.y as usize] != Tile::Wall {
    state.char_pos = next_pos;
  }
  if keycode == Keycode::T {
    // TODO take item
  }
}
