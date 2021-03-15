use game_lib::incmap::*;
use game_lib::types::*;
use game_lib::*;
use rand::prelude::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
mod assets;
mod entity;
mod game_map;
use entity::*;
use game_map::*;

fn main() {
  game_lib::run(
    qqvga_config(4),
    State::new(),
    init,
    update,
    render,
    handle_event,
  );
}

fn init(gcontext: &mut GContext) {
  crate::assets::load_sprites(gcontext);
}

enum Interaction {
  Walking,
  Take {
    available_items: Vec<(ItemId, ItemType)>,
  },
  Inventory,
  Drink,
  Open,
}

struct State {
  rng: ThreadRng,

  char_pos: P2I,
  game_map: GameMap,
  items: IncMap<Item>,
  enemies: IncMap<Enemy>,
  inventory: Vec<ItemType>,

  active_interaction: Interaction,
  status_text: Option<String>,
}

impl State {
  fn new() -> State {
    let rng = rand::thread_rng();
    let mut state = State {
      rng,
      char_pos: P2I::new(10, 5),
      game_map: GameMap::new(MAP_SIZE.x, MAP_SIZE.y),
      items: IncMap::new(),
      enemies: IncMap::new(),
      inventory: Vec::new(),
      active_interaction: Interaction::Walking,
      status_text: None,
    };
    state.generate_game_map();
    state.game_map.update_fog(state.char_pos);
    state
  }

  fn add_item(&mut self, x: i32, y: i32, item_type: ItemType) {
    self.items.insert(Item {
      pos: P2I::new(x, y),
      item_type,
    });
  }

  fn add_enemy(&mut self, x: i32, y: i32, enemy_type: EnemyType) {
    self.enemies.insert(Enemy {
      pos: P2I::new(x, y),
      enemy_type,
    });
  }

  fn inside_map(p: &P2I, rooms: &Arr2d<(CellType, bool)>) -> bool {
    0 <= p.x && p.x < rooms.width() as i32 && 0 <= p.y && p.y < rooms.height() as i32
  }

  fn pick_random_neighbor(
    p: &P2I,
    rooms: &Arr2d<(CellType, bool)>,
    rng: &mut ThreadRng,
  ) -> Option<P2I> {
    let neighbors = |p: &P2I| {
      vec![
        P2I::new(p.x + 1, p.y),
        P2I::new(p.x - 1, p.y),
        P2I::new(p.x, p.y + 1),
        P2I::new(p.x, p.y - 1),
      ]
    };
    let open_room = |p: &P2I, rooms: &Arr2d<(CellType, bool)>| {
      let room = rooms.get_unsafe(p.x as u32, p.y as u32);
      room.0 != CellType::Nothing && room.1 == false
    };
    neighbors(p)
      .iter()
      .filter(|p| Self::inside_map(p, rooms) && open_room(p, rooms))
      .choose(rng)
      .map(|x| *x)
  }

  fn generate_game_map(&mut self) {
    let d_x = 4;
    let d_y = 3;

    let player_start = P2I::new(
      self.rng.gen_range(0..d_x as i32),
      self.rng.gen_range(0..d_y as i32),
    );

    let mut rooms = Arr2d::new(d_x, d_y, (CellType::Room, false));
    let mut hor_walls = Arr2d::new(d_x, d_y - 1, Separator::Wall);
    let mut vert_walls = Arr2d::new(d_x - 1, d_y, Separator::Wall);

    // random void in top row
    *rooms.get_mut_unsafe(self.rng.gen_range(0..d_x), 0) = (CellType::Nothing, false);
    // random void in bottom row
    *rooms.get_mut_unsafe(self.rng.gen_range(0..d_x), d_y - 1) = (CellType::Nothing, false);

    // dfs
    let mut stack = Vec::new();
    stack.push(player_start);

    while let Some(&top) = stack.last() {
      match Self::pick_random_neighbor(&top, &rooms, &mut self.rng) {
        Some(random_neighbor) => {
          if self.rng.gen_range(1..=5) == 1 {
            // 20% chance that if a node is not a dead-end it gets converted into corridors
            rooms.get_mut_unsafe(top.x as u32, top.y as u32).0 = CellType::Corridors;
          }
          stack.push(random_neighbor);
          rooms
            .get_mut_unsafe(random_neighbor.x as u32, random_neighbor.y as u32)
            .1 = true;
          let carve_dir = random_neighbor - top;
          if carve_dir.x == 1 {
            // carved R
            *vert_walls.get_mut_unsafe(top.x as u32, top.y as u32) = Separator::Nothing;
          } else if carve_dir.x == -1 {
            // carved L
            *vert_walls.get_mut_unsafe((top.x - 1) as u32, top.y as u32) = Separator::Nothing;
          } else if carve_dir.y == 1 {
            // carved D
            *hor_walls.get_mut_unsafe(top.x as u32, top.y as u32) = Separator::Nothing;
          } else if carve_dir.y == -1 {
            // carved U
            *hor_walls.get_mut_unsafe(top.x as u32, (top.y - 1) as u32) = Separator::Nothing;
          }
        }
        None => {
          stack.pop();
          // TODO place doors and keys
        }
      }
    }
    // player start is always a room
    *rooms.get_mut_unsafe(player_start.x as u32, player_start.y as u32) = (CellType::Room, true);

    // draw tiles to map based on game map
    let map_x_div: u32 = (MAP_SIZE.x - if MAP_SIZE.x % d_x == 0 { 1 } else { 0 }) / d_x - 1;
    let map_y_div: u32 = (MAP_SIZE.y - if MAP_SIZE.y % d_y == 0 { 1 } else { 0 }) / d_y - 1;

    let mut room_widths = Vec::new();
    let mut room_heights = Vec::new();

    for _ in 0..d_x {
      room_widths.push(map_x_div);
    }
    for _ in 0..d_y {
      room_heights.push(map_y_div);
    }

    let sum_scan = |state: &mut u32, x: &u32| {
      let old_state = *state;
      *state = *state + x + 1;
      Some(old_state)
    };
    let room_x_starts: Vec<u32> = room_widths.iter().scan(1, sum_scan).collect();
    let room_y_starts: Vec<u32> = room_heights.iter().scan(1, sum_scan).collect();

    self.char_pos.x = room_x_starts[player_start.x as usize] as i32
      + room_widths[player_start.x as usize] as i32 / 2;
    self.char_pos.y = room_y_starts[player_start.y as usize] as i32
      + room_heights[player_start.y as usize] as i32 / 2;

    // we place all rooms
    for x in 0..d_x as usize {
      for y in 0..d_y as usize {
        let room_pos = P2I::new(room_x_starts[x] as i32, room_y_starts[y] as i32);
        match rooms.get_unsafe(x as u32, y as u32).0 {
          CellType::Room => {
            match self.rng.gen_range(0..10) {
              x if x == 0 => self.add_item(room_pos.x, room_pos.y, ItemType::Coin),
              x if 0 < x && x <= 3 => self.add_item(room_pos.x, room_pos.y, ItemType::Cherry),
              _ => (),
            }
            match self.rng.gen_range(0..5) {
              x if x == 0 => self.add_enemy(room_pos.x, room_pos.y, EnemyType::Gin),
              _ => (),
            }
            self.game_map.add_room(
              room_x_starts[x] as i32,
              room_y_starts[y] as i32,
              room_widths[x],
              room_heights[y],
            );
          }
          CellType::Corridors => {
            // TODO reduce code duplication here
            let room = P2I::new(x as i32, y as i32);
            let left = room + V2I::new(-1, 0);
            if Self::inside_map(&left, &rooms) {
              let sep = vert_walls.get_mut_unsafe((room.x - 1) as u32, room.y as u32);
              if *sep == Separator::Nothing || *sep == Separator::Door {
                *sep = Separator::Door;
                self.game_map.add_room(
                  room_x_starts[x] as i32 - 1,
                  room_y_starts[y] as i32 + room_heights[y] as i32 / 2,
                  room_widths[x] / 2 + 2,
                  1,
                );
              }
            }

            let right = room + V2I::new(1, 0);
            if Self::inside_map(&right, &rooms) {
              let sep = vert_walls.get_mut_unsafe(room.x as u32, room.y as u32);
              if *sep == Separator::Nothing || *sep == Separator::Door {
                *sep = Separator::Door;
                self.game_map.add_room(
                  room_x_starts[x] as i32 + room_widths[x] as i32 / 2,
                  room_y_starts[y] as i32 + room_heights[y] as i32 / 2,
                  room_widths[x] / 2 + 2,
                  1,
                );
              }
            }

            let up = room + V2I::new(0, -1);
            if Self::inside_map(&up, &rooms) {
              let sep = hor_walls.get_mut_unsafe(room.x as u32, (room.y - 1) as u32);
              if *sep == Separator::Nothing || *sep == Separator::Door {
                *sep = Separator::Door;
                self.game_map.add_room(
                  room_x_starts[x] as i32 + room_widths[x] as i32 / 2,
                  room_y_starts[y] as i32 - 1,
                  1,
                  room_heights[y] / 2 + 2,
                );
              }
            }

            let down = room + V2I::new(0, 1);
            if Self::inside_map(&down, &rooms) {
              let sep = hor_walls.get_mut_unsafe(room.x as u32, room.y as u32);
              if *sep == Separator::Nothing || *sep == Separator::Door {
                *sep = Separator::Door;
                self.game_map.add_room(
                  room_x_starts[x] as i32 + room_widths[x] as i32 / 2,
                  room_y_starts[y] as i32 + room_widths[x] as i32 / 2,
                  1,
                  room_heights[y] / 2 + 2,
                );
              }
            }
          }
          _ => (),
        }
      }
    }

    // then handle the separators (nothing, doorway, door)
    for x in 0..(d_x - 1) as usize {
      for y in 0..d_y as usize {
        match *vert_walls.get_unsafe(x as u32, y as u32) {
          Separator::Nothing => {
            if self.rng.gen_range(0..=2) > 0 {
              // add doorway
              self.game_map.add_rect(
                room_x_starts[x + 1] as i32 - 1,
                room_y_starts[y] as i32 + room_heights[y] as i32 / 2,
                1,
                1,
                Tile::Floor,
              )
            } else {
              // delete wall
              self.game_map.add_rect(
                room_x_starts[x + 1] as i32 - 1,
                room_y_starts[y] as i32,
                1,
                room_heights[y],
                Tile::Floor,
              )
            }
          }
          _ => (),
        }
      }
    }

    for x in 0..d_x as usize {
      for y in 0..(d_y - 1) as usize {
        match *hor_walls.get_unsafe(x as u32, y as u32) {
          Separator::Nothing => {
            if self.rng.gen_range(0..=2) > 0 {
              // add doorway
              self.game_map.add_rect(
                room_x_starts[x] as i32 + room_widths[x] as i32 / 2,
                room_y_starts[y + 1] as i32 - 1,
                1,
                1,
                Tile::Floor,
              )
            } else {
              // delete wall
              self.game_map.add_rect(
                room_x_starts[x] as i32,
                room_y_starts[y + 1] as i32 - 1,
                room_widths[x],
                1,
                Tile::Floor,
              )
            }
          }
          _ => (),
        }
      }
    }
  }

  fn process_ai(&mut self) {
    let rng = &mut self.rng;
    let enemies_marked_for_movement: Vec<EnemyId> = self
      .enemies
      .into_iter()
      .filter(|_| rng.gen_range(0..2) == 0)
      .map(|(enemy_id, _)| *enemy_id)
      .collect();
    for enemy_id in enemies_marked_for_movement {
      let random_dir = rng.gen_range(0..=3);
      let f = |x: i32| (x % 2) * (-1 as i32).pow(x as u32 / 2);
      let movement_dir = V2I::new(f(random_dir), f(random_dir + 1));
      let enemy = self.enemies.get_mut(enemy_id).unwrap();
      let next_pos = enemy.pos + movement_dir;
      if self.game_map.is_open_tile(next_pos) && self.char_pos != next_pos {
        enemy.pos = next_pos;
      }
    }
  }
}

#[derive(Copy, Clone, PartialEq)]
enum Separator {
  Nothing,
  Wall,
  Door,
}

#[derive(Copy, Clone, PartialEq)]
enum CellType {
  Nothing,
  Room,
  Corridors,
}

fn update(_state: &mut State, _key_status: &KeyStatus, _game_tick_counter: u32) {}

fn render(gcontext: &mut GContext, state: &State) {
  gcontext.reset_screen();
  for x in 0..MAP_SIZE.x {
    for y in 0..MAP_SIZE.y {
      let tile_visibility = state.game_map.tile_visibility(P2I::new(x as i32, y as i32));
      let screen_x = x as i32 * TILE_SIZE as i32;
      let screen_y = y as i32 * TILE_SIZE as i32;
      if tile_visibility != Fog::Dark {
        let tile_name = match state.game_map.get_tile(P2I::new(x as i32, y as i32)) {
          Tile::Void => None,
          Tile::Wall => Some("wall"),
          Tile::Floor => Some("floor"),
          Tile::Door { is_open: true } => Some("open door"),
          Tile::Door { is_open: false } => Some("closed door"),
        };
        tile_name.map(|tile_name| gcontext.draw_sprite(screen_x, screen_y, tile_name));
      }
      if tile_visibility == Fog::Seen {
        gcontext.draw_sprite(screen_x, screen_y, "shadow");
      }
    }

    state.status_text.as_ref().map(|status_text| {
      gcontext.draw_text(
        1,
        (gcontext.get_config().screen_size.y - game_lib::FONT_HEIGHT) as i32,
        status_text.as_str(),
      )
    });
  }

  for (_, enemy) in &state.enemies {
    let tile_visibility = state
      .game_map
      .tile_visibility(P2I::new(enemy.pos.x, enemy.pos.y));
    if tile_visibility == Fog::Visible {
      let enemy_name = enemy.enemy_type.to_string();
      gcontext.draw_sprite(
        enemy.pos.x * TILE_SIZE as i32,
        enemy.pos.y * TILE_SIZE as i32,
        &enemy_name,
      );
    }
  }

  for (_, item) in &state.items {
    let tile_visibility = state
      .game_map
      .tile_visibility(P2I::new(item.pos.x, item.pos.y));
    if tile_visibility == Fog::Visible {
      let item_name = item.item_type.to_string();
      gcontext.draw_sprite(
        item.pos.x * TILE_SIZE as i32,
        item.pos.y * TILE_SIZE as i32,
        &item_name,
      );
    }
  }
  gcontext.draw_sprite(
    state.char_pos.x * TILE_SIZE as i32,
    state.char_pos.y * TILE_SIZE as i32,
    "ball",
  );

  match &state.active_interaction {
    Interaction::Take { available_items } => {
      let mut lines: Vec<String> = available_items
        .iter()
        .enumerate()
        .map(|(ix, (_, item_type))| {
          let item_name = item_type.to_string();
          ((ix as u8 + 'a' as u8) as char).to_string() + " " + &item_name
        })
        .collect();
      lines.insert(0, "select item".to_string());
      gcontext.draw_text_box(
        HorPos::Center,
        VertPos::Top,
        &lines,
        sdl2::pixels::Color::RGB(40, 40, 40),
      );
    }
    Interaction::Inventory => {
      let inventory_lines: Vec<String> = if state.inventory.len() == 0 {
        vec!["inventory empty".to_string()]
      } else {
        state.inventory.iter().map(|x| x.to_string()).collect()
      };
      gcontext.draw_text_box(
        HorPos::Center,
        VertPos::Center,
        &inventory_lines,
        sdl2::pixels::Color::RGB(10, 10, 40),
      );
    }
    _ => (),
  }
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
  match &state.active_interaction {
    Interaction::Walking => match map_key_to_dir(keycode) {
      Some(move_dir) => {
        let next_pos = state.char_pos + move_dir;
        let enemies_at_pos: Vec<(&EnemyId, &Enemy)> = state
          .enemies
          .into_iter()
          .filter(|(_, enemy)| enemy.pos == next_pos)
          .collect();
        let no_blocking_enemy = enemies_at_pos.is_empty();
        let no_blocking_wall = state.game_map.is_open_tile(next_pos);
        if no_blocking_wall && no_blocking_enemy {
          state.char_pos = next_pos;
        }
        state.process_ai();
        state.game_map.update_fog(state.char_pos);
      }
      None => match keycode {
        Keycode::T => {
          let available_items: Vec<(ItemId, ItemType)> = state
            .items
            .into_iter()
            .filter(|(_, item)| item.pos == state.char_pos)
            .map(|(item_id, item)| (*item_id, item.item_type))
            .collect();
          if available_items.len() > 0 {
            state.active_interaction = Interaction::Take { available_items };
          }
        }
        Keycode::I => {
          state.active_interaction = Interaction::Inventory;
        }
        Keycode::D => {
          state.active_interaction = Interaction::Drink;
          state.status_text = Some("select drink direction".to_owned());
        }
        Keycode::O => {
          state.active_interaction = Interaction::Open;
          state.status_text = Some("select open direction".to_owned());
        }
        _ => (),
      },
    },
    Interaction::Take { available_items } => {
      let selection = keycode as i32 - 'a' as i32;
      if keycode == Keycode::Escape {
        state.active_interaction = Interaction::Walking;
      } else if 0 <= selection && selection < available_items.len() as i32 {
        let (item_id, item_type) = available_items[selection as usize];
        state.inventory.push(item_type);
        state.items.remove(item_id);
        state.active_interaction = Interaction::Walking;
      }
    }
    Interaction::Inventory => {
      state.active_interaction = Interaction::Walking;
    }
    Interaction::Drink => {
      map_key_to_dir(keycode).map(|drink_dir| {
        let drink_target_pos = state.char_pos + drink_dir;
        let gin_at_target_pos = state
          .enemies
          .into_iter()
          .filter(|(_, enemy)| enemy.pos == drink_target_pos && enemy.enemy_type == EnemyType::Gin)
          .map(|(enemy_id, enemy)| (*enemy_id, *enemy))
          .nth(0);

        gin_at_target_pos.map(|(gin_id, _)| {
          state.enemies.remove(gin_id);
          state.items.insert(Item {
            pos: drink_target_pos,
            item_type: ItemType::DeadGin,
          })
        });
      });
      state.active_interaction = Interaction::Walking;
      state.status_text = None;
    }
    Interaction::Open => {
      map_key_to_dir(keycode).map(|open_dir| {
        let open_target_pos = state.char_pos + open_dir;
        match state.game_map.get_tile(open_target_pos) {
          Tile::Door { is_open: false } => state
            .game_map
            .set_tile(open_target_pos, Tile::Door { is_open: true }),
          _ => (),
        }
      });
      state.active_interaction = Interaction::Walking;
      state.status_text = None;
    }
  }
}

fn map_key_to_dir(keycode: Keycode) -> Option<V2I> {
  match keycode {
    Keycode::Up => Some(V2I::new(0, -1)),
    Keycode::Left => Some(V2I::new(-1, 0)),
    Keycode::Down => Some(V2I::new(0, 1)),
    Keycode::Right => Some(V2I::new(1, 0)),
    _ => None,
  }
}
