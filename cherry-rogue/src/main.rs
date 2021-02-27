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
  rng: rand::prelude::ThreadRng,

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

  fn generate_game_map(&mut self) {
    self.add_item(2, 3, ItemType::Cherry);
    self.add_item(3, 4, ItemType::Coin);
    self.add_item(3, 4, ItemType::Cherry);
    self.add_enemy(5, 5, EnemyType::Gin);
    self.game_map.add_room(2, 3, 4, 6);
    self.game_map.add_room(6, 4, 3, 1);
    self
      .game_map
      .set_tile(P2I::new(7, 4), Tile::Door { is_open: false });
    self.game_map.add_room(9, 2, 5, 5);
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

fn update(_state: &mut State, _key_status: &KeyStatus, _game_tick_counter: u32) {}

fn render(gcontext: &mut GContext, state: &State) {
  for x in 0..MAP_SIZE.x {
    for y in 0..MAP_SIZE.y {
      let tile_name = match state.game_map.get_tile(P2I::new(x as i32, y as i32)) {
        Tile::Void => None,
        Tile::Wall => Some("wall"),
        Tile::Floor => Some("floor"),
        Tile::Door { is_open: true } => Some("open door"),
        Tile::Door { is_open: false } => Some("closed door"),
      };
      tile_name.map(|tile_name| {
        gcontext.draw_sprite(
          x as i32 * TILE_SIZE as i32,
          y as i32 * TILE_SIZE as i32,
          tile_name,
        )
      });
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
    let enemy_name = enemy.enemy_type.to_string();
    gcontext.draw_sprite(
      enemy.pos.x * TILE_SIZE as i32,
      enemy.pos.y * TILE_SIZE as i32,
      &enemy_name,
    );
  }

  for (_, item) in &state.items {
    let item_name = item.item_type.to_string();
    gcontext.draw_sprite(
      item.pos.x * TILE_SIZE as i32,
      item.pos.y * TILE_SIZE as i32,
      &item_name,
    );
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
