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
