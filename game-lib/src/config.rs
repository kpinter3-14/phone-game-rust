use crate::types::*;
use std::collections::HashMap;

pub type Palette = HashMap<char, sdl2::pixels::Color>;

pub struct Config {
  pub scale: u32,
  pub screen_size: V2U,
  pub font_color: sdl2::pixels::Color,
  pub background_color: sdl2::pixels::Color,
  pub palette: Palette,
}

pub const TRANSPARENT: sdl2::pixels::Color = sdl2::pixels::Color::RGBA(0, 0, 0, 0);
pub const DARK_COLOR: sdl2::pixels::Color = sdl2::pixels::Color::RGB(67, 82, 61);
pub const BRIGHT_COLOR: sdl2::pixels::Color = sdl2::pixels::Color::RGB(199, 240, 216);

pub fn cell_phone_config(scale: u32) -> Config {
  Config {
    scale,
    screen_size: V2U::new(84, 48),
    font_color: DARK_COLOR,
    background_color: BRIGHT_COLOR,
    palette: [('#', DARK_COLOR), ('_', BRIGHT_COLOR)]
      .iter()
      .cloned()
      .collect(),
  }
}

pub fn qqvga_config(scale: u32) -> Config {
  Config {
    scale,
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
  }
}

pub fn qvga_config(scale: u32) -> Config {
  Config {
    scale,
    screen_size: V2U::new(320, 240),
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
  }
}

pub fn vga_config(scale: u32) -> Config {
  Config {
    scale,
    screen_size: V2U::new(640, 480),
    font_color: sdl2::pixels::Color::WHITE,
    background_color: sdl2::pixels::Color::BLACK,
    palette: [].iter().cloned().collect(),
  }
}
