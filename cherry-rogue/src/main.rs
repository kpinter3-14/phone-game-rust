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
    available_items: Vec<(EntityId, ItemType)>,
  },
  Inventory,
  Drink,
  Open,
}

struct State {
  rng: rand::prelude::ThreadRng,

  char_pos: P2I,
  game_map: GameMap,
  entities: IncMap<Entity>,
  inventory: Vec<ItemType>,

  active_interaction: Interaction,
  status_text: Option<String>,
}

impl State {
  fn new() -> State {
    let rng = rand::thread_rng();
    let game_map = GameMap {
      map_array: [[Tile::Void; MAP_SIZE.y as usize]; MAP_SIZE.x as usize],
    };
    let mut state = State {
      rng,
      char_pos: P2I::new(10, 5),
      game_map,
      entities: IncMap::new(),
      inventory: Vec::new(),
      active_interaction: Interaction::Walking,
      status_text: None,
    };
    state.generate_game_map();
    state
  }

  fn generate_game_map(&mut self) {
    self.entities.insert(Entity {
      pos: P2I::new(2, 3),
      entity_type: EntityType::Item {
        item_type: ItemType::Cherry,
      },
    });
    self.entities.insert(Entity {
      pos: P2I::new(3, 4),
      entity_type: EntityType::Item {
        item_type: ItemType::Coin,
      },
    });
    self.entities.insert(Entity {
      pos: P2I::new(3, 4),
      entity_type: EntityType::Item {
        item_type: ItemType::Cherry,
      },
    });
    self.entities.insert(Entity {
      pos: P2I::new(5, 5),
      entity_type: EntityType::Enemy {
        enemy_type: EnemyType::Gin,
      },
    });
    self.game_map.add_room(2, 3, 4, 6);
    self.game_map.add_room(6, 4, 3, 1);
    self.game_map.map_array[7][4] = Tile::Door { is_open: false };
    self.game_map.add_room(9, 2, 5, 5);
  }

  fn get_entities_at(&self, pos: P2I) -> Vec<EntityId> {
    self
      .entities
      .into_iter()
      .filter(|(_, entity)| entity.pos == pos)
      .map(|(id, _)| *id)
      .collect()
  }

  fn get_entities_at_projected<P, B>(&self, pos: P2I, project: P) -> Vec<(EntityId, B)>
  where
    P: Fn(&Entity) -> Option<B>,
  {
    self
      .entities
      .into_iter()
      .filter(|(_, entity)| entity.pos == pos)
      .filter_map(|(id, entity)| project(entity).map(|b| (*id, b)))
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
    for (entity_id, _enemy_type) in enemies {
      let should_move = self.rng.gen_range(0..2) == 0;
      if should_move {
        let random_dir = self.rng.gen_range(0..=3);
        let f = |x: i32| (x % 2) * (-1 as i32).pow(x as u32 / 2);
        let movement_dir = V2I::new(f(random_dir), f(random_dir + 1));
        let entity = self.entities.get_mut(entity_id).unwrap();
        let next_pos = entity.pos + movement_dir;
        if self.game_map.is_open_tile(next_pos) && self.char_pos != next_pos {
          entity.pos = next_pos;
        }
      }
    }
  }
}

fn update(_state: &mut State, _key_status: &KeyStatus, _game_tick_counter: u32) {}

fn render(gcontext: &mut GContext, state: &State) {
  for x in 0..MAP_SIZE.x {
    for y in 0..MAP_SIZE.y {
      let tile_name = match state.game_map.map_array[x as usize][y as usize] {
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

  match &state.active_interaction {
    Interaction::Take { available_items } => {
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
        let enemies_at_pos =
          state.get_entities_at_projected(next_pos, |entity| match entity.entity_type {
            EntityType::Enemy { enemy_type } => Some(enemy_type),
            _ => None,
          });
        let no_blocking_enemy = enemies_at_pos.is_empty();
        let no_blocking_wall = state.game_map.is_open_tile(next_pos);
        if no_blocking_wall && no_blocking_enemy {
          state.char_pos = next_pos;
        }
        state.process_ai();
      }
      None => match keycode {
        Keycode::T => {
          let available_items: Vec<(EntityId, ItemType)> =
            state.get_entities_at_projected(state.char_pos, |entity| match entity.entity_type {
              EntityType::Item { item_type } => Some(item_type),
              _ => None,
            });
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
        let (entity_id, item_type) = available_items[selection as usize];
        state.inventory.push(item_type);
        state.entities.remove(entity_id);
        state.active_interaction = Interaction::Walking;
      }
    }
    Interaction::Inventory => {
      state.active_interaction = Interaction::Walking;
    }
    Interaction::Drink => {
      map_key_to_dir(keycode).map(|drink_dir| {
        let drink_target_pos = state.char_pos + drink_dir;
        state
          .get_entities_at_projected(drink_target_pos, |entity| match entity.entity_type {
            EntityType::Enemy {
              enemy_type: EnemyType::Gin,
            } => Some(()),
            _ => None,
          })
          .iter()
          .nth(0)
          .map(|(gin_entity_id, _)| {
            state.entities.remove(*gin_entity_id);
            state.entities.insert(Entity {
              pos: drink_target_pos,
              entity_type: EntityType::Item {
                item_type: ItemType::DeadGin,
              },
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
