use crate::types::*;

pub struct Config {
  pub scale: u32,
  pub screen_size: V2U,
  pub font_color: sdl2::pixels::Color,
  pub background_color: sdl2::pixels::Color,
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
  }
}

pub fn qqvga_config(scale: u32) -> Config {
  Config {
    scale,
    screen_size: V2U::new(160, 120),
    font_color: sdl2::pixels::Color::WHITE,
    background_color: sdl2::pixels::Color::BLACK,
  }
}

pub fn qvga_config(scale: u32) -> Config {
  Config {
    scale,
    screen_size: V2U::new(320, 240),
    font_color: sdl2::pixels::Color::WHITE,
    background_color: sdl2::pixels::Color::BLACK,
  }
}

pub fn vga_config(scale: u32) -> Config {
  Config {
    scale,
    screen_size: V2U::new(640, 480),
    font_color: sdl2::pixels::Color::WHITE,
    background_color: sdl2::pixels::Color::BLACK,
  }
}
