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
  let config = qqvga_config(4);

  let mut state = State::new();
  state.entities.insert(Entity {
    pos: V2I::new(2, 3),
    entity_type: EntityType::Item {
      item_type: ItemType::Cherry,
    },
  });
  state.entities.insert(Entity {
    pos: V2I::new(3, 4),
    entity_type: EntityType::Item {
      item_type: ItemType::Coin,
    },
  });
  state.entities.insert(Entity {
    pos: V2I::new(3, 4),
    entity_type: EntityType::Item {
      item_type: ItemType::Cherry,
    },
  });
  state.entities.insert(Entity {
    pos: V2I::new(5, 5),
    entity_type: EntityType::Enemy {
      enemy_type: EnemyType::Gin,
    },
  });
  state.game_map.add_room(2, 3, 4, 6);
  state.game_map.add_room(6, 4, 3, 1);
  state.game_map.add_room(9, 2, 5, 5);

  game_lib::run(config, state, init, update, render, handle_event);
}

fn init(gcontext: &mut GContext) {
  crate::assets::load_sprites(gcontext);
}

enum Menu {
  Take {
    available_items: Vec<(EntityId, ItemType)>,
  },
  Inventory,
}

struct State {
  rng: rand::prelude::ThreadRng,

  char_pos: V2I,
  game_map: GameMap,
  entities: IncMap<Entity>,
  inventory: Vec<ItemType>,

  active_menu: Option<Menu>,
  score: i32,
}

impl State {
  fn new() -> State {
    let mut rng = rand::thread_rng();
    let mut game_map = GameMap {
      map_array: [[Tile::Void; MAP_SIZE.y as usize]; MAP_SIZE.x as usize],
    };
    State {
      rng,
      char_pos: V2I::new(10, 5),
      game_map,
      entities: IncMap::new(),
      inventory: Vec::new(),
      active_menu: None,
      score: 0,
    }
  }

  fn get_entities_at(&self, pos: V2I) -> Vec<EntityId> {
    self
      .entities
      .into_iter()
      .filter(|(_, entity)| entity.pos == pos)
      .map(|(id, _)| *id)
      .collect()
  }

  fn process_ai(&mut self) {
    let enemies: Vec<(EntityId, EnemyType)> = self
      .entities
      .into_iter()
      .filter_map(|(entity_id, entity)| match entity.entity_type {
        EntityType::Enemy { enemy_type } => Some((*entity_id, enemy_type)),
        _ => None,
      })
      .collect();
    for (entity_id, enemy_type) in enemies {
      let random_dir = self.rng.gen_range(0..=3);
      let f = |x: i32| (x % 2) * (-1 as i32).pow(x as u32 / 2);
      let movement_dir = V2I::new(f(random_dir), f(random_dir + 1));
      let entity = self.entities.get_mut(entity_id).unwrap();
      let next_pos = entity.pos + movement_dir;
      if self.game_map.is_open_tile(next_pos) {
        entity.pos = next_pos;
      }
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
      let tile_name = match state.game_map.map_array[x as usize][y as usize] {
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
    let entity_name = entity.entity_type.to_string();
    gcontext.draw_sprite(
      entity.pos.x * TILE_SIZE as i32,
      entity.pos.y * TILE_SIZE as i32,
      &entity_name,
    );
  }
  gcontext.draw_sprite(
    state.char_pos.x * TILE_SIZE as i32,
    state.char_pos.y * TILE_SIZE as i32,
    "ball",
  );

  match &state.active_menu {
    None => (),
    Some(Menu::Take { available_items }) => {
      let mut lines: Vec<String> = available_items
        .iter()
        .enumerate()
        .map(|(ix, (_, item_type))| {
          let entity_name = item_type.to_string();
          ((ix as u8 + 'a' as u8) as char).to_string() + " " + &entity_name
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
    Some(Menu::Inventory) => {
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
  match &state.active_menu {
    None => {
      let move_dir = match keycode {
        Keycode::Up => Some(V2I::new(0, -1)),
        Keycode::Left => Some(V2I::new(-1, 0)),
        Keycode::Down => Some(V2I::new(0, 1)),
        Keycode::Right => Some(V2I::new(1, 0)),
        _ => None,
      };
      match move_dir {
        Some(move_dir) => {
          let next_pos = state.char_pos + move_dir;
          if state.game_map.map_array[next_pos.x as usize][next_pos.y as usize] != Tile::Wall {
            state.char_pos = next_pos;
          }
          state.process_ai();
        }
        None => match keycode {
          Keycode::T => {
            let entities_at_player = state.get_entities_at(state.char_pos);
            let available_items: Vec<(EntityId, ItemType)> = entities_at_player
              .iter()
              .filter_map(|&entity_id| {
                let entity = state.entities.get(entity_id).unwrap();
                match entity.entity_type {
                  EntityType::Item { item_type } => Some((entity_id, item_type)),
                  _ => None,
                }
              })
              .collect();
            if available_items.len() > 0 {
              state.active_menu = Some(Menu::Take { available_items });
            }
          }
          Keycode::I => {
            state.active_menu = Some(Menu::Inventory);
          }
          _ => (),
        },
      }
    }
    Some(Menu::Take { available_items }) => {
      let selection = keycode as i32 - 'a' as i32;
      if keycode == Keycode::Escape {
        state.active_menu = None;
      } else if 0 <= selection && selection < available_items.len() as i32 {
        let (entity_id, item_type) = available_items[selection as usize];
        state.inventory.push(item_type);
        state.entities.remove(entity_id);
        state.active_menu = None;
      }
    }
    Some(Menu::Inventory) => {
      state.active_menu = None;
    }
  }
}
