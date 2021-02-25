use game_lib::types::*;

#[derive(Copy, Clone, PartialEq)]
pub enum Tile {
  Void,
  Wall,
  Floor,
  Door { is_open: bool },
}

pub struct GameMap {
  pub map_array: GameMapArray,
}

pub const TILE_SIZE: u32 = 8;
pub const MAP_SIZE: V2U = V2U::new(20, 14);
pub type GameMapArray = [[Tile; MAP_SIZE.y as usize]; MAP_SIZE.x as usize];

impl GameMap {
  pub fn add_room(&mut self, x: i32, y: i32, w: i32, h: i32) {
    for x_ix in x..x + w {
      for y_ix in y..y + h {
        self.map_array[x_ix as usize][y_ix as usize] = Tile::Floor;
      }
    }

    for x_ix in x - 1..x + w + 1 {
      let top_row_tile: &mut Tile = &mut self.map_array[x_ix as usize][(y - 1) as usize];
      if *top_row_tile != Tile::Floor {
        *top_row_tile = Tile::Wall;
      }
      let bottom_row_tile: &mut Tile = &mut self.map_array[x_ix as usize][(y + h) as usize];
      if *bottom_row_tile != Tile::Floor {
        *bottom_row_tile = Tile::Wall;
      }
    }

    for y_ix in y - 1..y + h + 1 {
      let left_col_tile: &mut Tile = &mut self.map_array[(x - 1) as usize][y_ix as usize];
      if *left_col_tile != Tile::Floor {
        *left_col_tile = Tile::Wall;
      }
      let right_col_tile: &mut Tile = &mut self.map_array[(x + w) as usize][y_ix as usize];
      if *right_col_tile != Tile::Floor {
        *right_col_tile = Tile::Wall;
      }
    }
  }

  pub fn set_tile(&mut self, pos: V2I, tile: Tile) {
    self.map_array[pos.x as usize][pos.y as usize] = tile;
  }

  pub fn get_tile(&self, pos: V2I) -> Tile {
    self.map_array[pos.x as usize][pos.y as usize]
  }

  pub fn is_open_tile(&self, pos: V2I) -> bool {
    let tile = self.map_array[pos.x as usize][pos.y as usize];
    tile == Tile::Floor || tile == Tile::Door { is_open: true }
  }
}
