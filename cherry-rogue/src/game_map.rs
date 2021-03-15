use game_lib::types::*;
use game_lib::*;

#[derive(Copy, Clone, PartialEq)]
pub enum Tile {
  Void,
  Wall,
  Floor,
  Door { is_open: bool },
}

#[derive(Copy, Clone, PartialEq)]
pub enum Fog {
  Dark,
  Seen,
  Visible,
}

pub struct GameMap {
  map_arr2d: Arr2d<Tile>,
  fog_of_war: Arr2d<Fog>,
}

pub const TILE_SIZE: u32 = 8;
pub const MAP_SIZE: V2U = V2U::new(20, 14);

impl GameMap {
  pub fn new(w: u32, h: u32) -> GameMap {
    GameMap {
      map_arr2d: Arr2d::new(w, h, Tile::Void),
      fog_of_war: Arr2d::new(w, h, Fog::Dark),
    }
  }

  pub fn add_rect(&mut self, x: i32, y: i32, w: u32, h: u32, t: Tile) {
    for x_ix in x..x + w as i32 {
      for y_ix in y..y + h as i32 {
        self.map_arr2d.set(x_ix, y_ix, t);
      }
    }
  }

  pub fn add_room(&mut self, x: i32, y: i32, w: u32, h: u32) {
    self.add_rect(x, y, w, h, Tile::Floor);

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
    !Self::blocking_tile(tile)
  }

  pub fn tile_visibility(&self, pos: P2I) -> Fog {
    *self.fog_of_war.get(pos.x, pos.y).unwrap_or(&Fog::Dark)
  }

  fn blocking_tile(tile: Tile) -> bool {
    match tile {
      Tile::Door { is_open: false } => true,
      Tile::Wall => true,
      _ => false,
    }
  }

  pub fn update_fog(&mut self, pos: P2I) {
    self
      .fog_of_war
      .update(|x| if x == Fog::Visible { Fog::Seen } else { x });

    let r = 4;
    let map_arr2d = &self.map_arr2d;
    let tile_is_blocking = |pos: P2I| {
      map_arr2d
        .get(pos.x, pos.y)
        .map(|x| Self::blocking_tile(*x))
        .unwrap_or(true)
    };
    let fog_of_war = &mut self.fog_of_war;
    let mut mark_visible = |pos: P2I| {
      fog_of_war.get_mut(pos.x, pos.y).map(|x| *x = Fog::Visible);
    };

    let half_tile = V2F::new(0.5, 0.5);

    let pos_f = P2F::new(pos.x as f32, pos.y as f32) + half_tile;
    let mut mark_tiles_on_ray = |x, y| {
      dda(
        tile_is_blocking,
        &mut mark_visible,
        pos_f,
        V2F::new(x as f32, y as f32),
        r as f32,
      )
    };
    for x in -r..=r {
      mark_tiles_on_ray(x, -r);
      mark_tiles_on_ray(x, r);
    }
    for y in -r..=r {
      mark_tiles_on_ray(-r, y);
      mark_tiles_on_ray(r, y);
    }
  }
}
