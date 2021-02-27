use game_lib::types::*;
use game_lib::*;

#[derive(Copy, Clone, PartialEq)]
pub enum Tile {
  Void,
  Wall,
  Floor,
  Door { is_open: bool },
}

pub struct GameMap {
  map_arr2d: Arr2d<Tile>,
}

pub const TILE_SIZE: u32 = 8;
pub const MAP_SIZE: V2U = V2U::new(20, 14);

impl GameMap {
  pub fn new(w: u32, h: u32) -> GameMap {
    GameMap {
      map_arr2d: Arr2d::new(w, h, Tile::Void),
    }
  }

  pub fn add_room(&mut self, x: i32, y: i32, w: u32, h: u32) {
    for x_ix in x..x + w as i32 {
      for y_ix in y..y + h as i32 {
        self.map_arr2d.set(x_ix, y_ix, Tile::Floor);
      }
    }

    for x_ix in x - 1..x + w as i32 + 1 {
      let top_row_tile: &mut Tile = &mut self.map_arr2d.get_mut_unsafe(x_ix as u32, (y - 1) as u32);
      if *top_row_tile != Tile::Floor {
        *top_row_tile = Tile::Wall;
      }
      let bottom_row_tile: &mut Tile =
        &mut self.map_arr2d.get_mut_unsafe(x_ix as u32, y as u32 + h);
      if *bottom_row_tile != Tile::Floor {
        *bottom_row_tile = Tile::Wall;
      }
    }

    for y_ix in y - 1..y + h as i32 + 1 {
      let left_col_tile: &mut Tile =
        &mut self.map_arr2d.get_mut_unsafe((x - 1) as u32, y_ix as u32);
      if *left_col_tile != Tile::Floor {
        *left_col_tile = Tile::Wall;
      }
      let right_col_tile: &mut Tile = &mut self.map_arr2d.get_mut_unsafe(x as u32 + w, y_ix as u32);
      if *right_col_tile != Tile::Floor {
        *right_col_tile = Tile::Wall;
      }
    }
  }

  pub fn set_tile(&mut self, pos: P2I, tile: Tile) {
    self.map_arr2d.set(pos.x, pos.y, tile);
  }

  pub fn get_tile(&self, pos: P2I) -> &Tile {
    self.map_arr2d.get_unsafe(pos.x as u32, pos.y as u32)
  }

  pub fn is_open_tile(&self, pos: P2I) -> bool {
    let tile = *self.get_tile(pos);
    tile == Tile::Floor || tile == Tile::Door { is_open: true }
  }
}
