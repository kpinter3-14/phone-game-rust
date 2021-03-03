use crate::config::*;
use crate::types::*;
use crate::Config;
use crate::KeyStatus;
use sdl2::rect::Rect;
use std::collections::HashMap;

pub struct GContext<'a> {
  pub ms_since_start_last_frame: u32,
  pub ms_until_game_tick: u32,
  pub game_tick_counter: u32,

  pub event_pump: sdl2::EventPump,
  pub texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext>,
  pub screen_buffer: sdl2::render::Texture,
  pub pixel_data_surface: sdl2::surface::Surface<'a>,
  pub timer_subsystem: sdl2::TimerSubsystem,
  pub canvas: sdl2::render::Canvas<sdl2::video::Window>,
  config: Config,
  pub window_size: V2U,
  pub camera: P2I,
  font_sprite: sdl2::surface::Surface<'a>,
  surface_store: HashMap<SurfaceName, sdl2::surface::Surface<'a>>,
  sprite_sheet_store: HashMap<SpriteSheetName, SheetData>,
  sprite_store: HashMap<SpriteName, SpriteData>,
  pub want_to_quit: bool,
  pub key_status: KeyStatus,
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct SurfaceName(pub String);
#[derive(Eq, PartialEq, Hash, Clone)]
pub struct SpriteSheetName(pub String);
#[derive(Eq, PartialEq, Hash, Clone)]
pub struct SpriteName(pub String);

pub fn surface(name: &str) -> SurfaceName {
  SurfaceName(name.to_string())
}

pub fn sprite_sheet(name: &str) -> SpriteSheetName {
  SpriteSheetName(name.to_string())
}

pub fn sprite(name: &str) -> SpriteName {
  SpriteName(name.to_string())
}

struct SheetData {
  surface_name: SurfaceName,
  size: V2U,
}

struct SpriteData {
  sheet_name: SpriteSheetName,
  sheet_coords: V2U,
}

impl<'a> GContext<'a> {
  pub fn new(config: Config) -> GContext<'a> {
    let sdl_context = sdl2::init().unwrap();
    let event_pump = sdl_context.event_pump().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let timer_subsystem = sdl_context.timer().unwrap();
    let window_size = config.screen_size * config.scale;

    let window = video_subsystem
      .window("", window_size.x, window_size.y)
      .position_centered()
      .resizable()
      .build()
      .map_err(|e| e.to_string())
      .unwrap();

    let canvas = window
      .into_canvas()
      .target_texture()
      .present_vsync()
      .build()
      .map_err(|e| e.to_string())
      .unwrap();

    let texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext> =
      canvas.texture_creator();

    let screen_buffer = texture_creator
      .create_texture_streaming(
        sdl2::pixels::PixelFormatEnum::BGR888,
        config.screen_size.x,
        config.screen_size.y,
      )
      .unwrap();

    let pixel_data_surface = sdl2::surface::Surface::new(
      config.screen_size.x,
      config.screen_size.y,
      sdl2::pixels::PixelFormatEnum::BGR888,
    )
    .unwrap();

    let font_sprite = surface_from_strvec(
      &[('o', config.font_color)].iter().cloned().collect(),
      FONT_DATA,
    );

    GContext {
      ms_since_start_last_frame: 0,
      ms_until_game_tick: 0,
      game_tick_counter: 0,
      event_pump,
      timer_subsystem,
      texture_creator,
      screen_buffer,
      pixel_data_surface,
      canvas,
      config,
      window_size,
      camera: P2I::new(0, 0),
      font_sprite,
      sprite_store: HashMap::new(),
      sprite_sheet_store: HashMap::new(),
      surface_store: HashMap::new(),
      want_to_quit: false,
      key_status: KeyStatus::new(),
    }
  }

  pub fn reset_screen(&mut self) {
    self
      .pixel_data_surface
      .fill_rect(None, self.config.background_color)
      .unwrap();
  }

  pub fn take_screenshot(canvas: &sdl2::render::Canvas<sdl2::video::Window>) {
    let pic_data = canvas
      .read_pixels(None, sdl2::pixels::PixelFormatEnum::ABGR8888)
      .unwrap();
    let path = std::path::Path::new(r"screenshot.png");
    let file = std::fs::File::create(path).unwrap();
    let ref mut w = std::io::BufWriter::new(file);

    let (width, height) = canvas.window().size();

    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::RGBA);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();

    writer.write_image_data(&pic_data).unwrap();
  }

  fn set_pixel(&mut self, x: i32, y: i32, color: sdl2::pixels::Color) {
    if x < 0
      || self.config.screen_size.x as i32 <= x
      || y < 0
      || self.config.screen_size.y as i32 <= y
    {
      return;
    }
    let screen_width = self.config.screen_size.x as i32;
    self.pixel_data_surface.with_lock_mut(|surf: &mut [u8]| {
      let off = (x + y * screen_width) as usize * 4;
      surf[off + 0] = color.b;
      surf[off + 1] = color.g;
      surf[off + 2] = color.r;
    });
  }

  // midpoint circle algorithm
  pub fn draw_circle(&mut self, cx: i32, cy: i32, r: i32, m: i32, color: sdl2::pixels::Color) {
    let mut f = 1 - r;
    let mut dx = 0;
    let mut dy = -2 * r;
    let mut x = 0;
    let mut y = r;

    self.set_pixel(cx, cy + r, color);
    self.set_pixel(cx, cy - r, color);
    self.set_pixel(cx + r, cy, color);
    self.set_pixel(cx - r, cy, color);

    while x < y {
      if f >= 0 {
        y -= 1;
        dy += 2;
        f += dy;
      }
      x += 1;
      dx += 2;
      f += dx + 1;
      if x % m == 0 {
        self.set_pixel(cx + x, cy + y, color);
        self.set_pixel(cx - x, cy + y, color);
        self.set_pixel(cx + x, cy - y, color);
        self.set_pixel(cx - x, cy - y, color);
        self.set_pixel(cx + y, cy + x, color);
        self.set_pixel(cx - y, cy + x, color);
        self.set_pixel(cx + y, cy - x, color);
        self.set_pixel(cx - y, cy - x, color);
      }
    }
  }

  pub fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: sdl2::pixels::Color) {
    self
      .pixel_data_surface
      .fill_rect(sdl2::rect::Rect::new(x, y, w, h), color)
      .unwrap();
  }

  pub fn draw_text(&mut self, x: i32, y: i32, text: &str) {
    let mut i = 0;
    for ch in text.chars() {
      if ch != ' ' {
        let ix = if '0' <= ch && ch <= '9' {
          ch as u8 - '0' as u8
        } else {
          10 + ch as u8 - 'a' as u8
        };
        let ix_x = ix % 10;
        let ix_y = ix / 10;
        self
          .font_sprite
          .blit(
            Rect::new(
              ix_x as i32 * FONT_WIDTH as i32,
              ix_y as i32 * FONT_HEIGHT as i32,
              FONT_WIDTH,
              FONT_HEIGHT,
            ),
            &mut self.pixel_data_surface,
            sdl2::rect::Rect::new(x + i * (1 + FONT_WIDTH as i32), y, FONT_WIDTH, 5),
          )
          .unwrap();
      }
      i += 1;
    }
  }

  pub fn draw_text_box<T: AsRef<str>>(
    &mut self,
    hor_pos: HorPos,
    vert_pos: VertPos,
    lines: &[T],
    background_color: sdl2::pixels::Color,
  ) {
    let text_box_w: u32 = lines
      .iter()
      .max_by(|x, y| x.as_ref().len().partial_cmp(&y.as_ref().len()).unwrap())
      .map(|x| x.as_ref().len())
      .unwrap_or(0) as u32
      * (FONT_WIDTH + 1)
      + 1;
    let text_box_h: u32 = lines.len() as u32 * (FONT_HEIGHT + 1) + 1;
    let dialogue_x = match hor_pos {
      HorPos::Left => 0,
      HorPos::Center => (self.get_config().screen_size.x as i32 - text_box_w as i32) / 2,
      HorPos::Right => self.get_config().screen_size.x as i32 - text_box_w as i32,
      HorPos::Abs { x } => x,
    };
    let dialogue_y = match vert_pos {
      VertPos::Top => 0,
      VertPos::Center => (self.get_config().screen_size.y as i32 - text_box_h as i32) / 2,
      VertPos::Bottom => self.get_config().screen_size.y as i32 - text_box_h as i32,
      VertPos::Abs { y } => y,
    };
    self.draw_rect(
      dialogue_x,
      dialogue_y,
      text_box_w,
      text_box_h,
      background_color,
    );
    let mut ix = 0;
    for line in lines {
      self.draw_text(
        dialogue_x + 1,
        dialogue_y + ix * (FONT_HEIGHT as i32 + 1) + 1,
        line.as_ref(),
      );
      ix += 1;
    }
  }

  pub fn add_surface(&mut self, surface_name: SurfaceName, data: Vec<&str>) {
    self.surface_store.insert(
      surface_name,
      surface_from_strvec(&self.config.palette, &data),
    );
  }

  pub fn add_sprite_sheet(&mut self, sprite_sheet_name: SpriteSheetName, sprite_sheet_path: &str, size: V2U) {
    let surface_name = SurfaceName("#".to_owned() + &sprite_sheet_name.0);

    self.surface_store.insert(
      surface_name.clone(),
      load_surface(sprite_sheet_path),
    );

    self.sprite_sheet_store.insert(
      sprite_sheet_name,
      SheetData {
        size,
        surface_name,
      },
    );
  }

  pub fn add_sprite(
    &mut self,
    sprite_sheet_name: SpriteSheetName,
    sprite_name: SpriteName,
    sheet_x: u32,
    sheet_y: u32,
  ) {
    self.sprite_store.insert(
      sprite_name,
      SpriteData {
        sheet_coords: V2U::new(sheet_x, sheet_y),
        sheet_name: sprite_sheet_name,
      },
    );
  }

  pub fn draw_surface(&mut self, x: i32, y: i32, surface_name: SurfaceName) {
    let surface = self.surface_store.get(&surface_name).unwrap();
    surface
      .blit(
        None,
        &mut self.pixel_data_surface,
        sdl2::rect::Rect::new(x, y, surface.width(), surface.height()),
      )
      .unwrap();
  }

  pub fn draw_sprite(&mut self, x: i32, y: i32, sprite_name: SpriteName) {
    let sprite = self.sprite_store.get(&sprite_name).unwrap();
    let sprite_sheet = self.sprite_sheet_store.get(&sprite.sheet_name).unwrap();
    let sprite_sheet_surface = self.surface_store.get(&sprite_sheet.surface_name).unwrap();
    let sprite_w = sprite_sheet_surface.width() / sprite_sheet.size.x;
    let sprite_h = sprite_sheet_surface.height() / sprite_sheet.size.y;
    let sprite_x = sprite_w * sprite.sheet_coords.x;
    let sprite_y = sprite_h * sprite.sheet_coords.y;
    sprite_sheet_surface
      .blit(
        sdl2::rect::Rect::new(sprite_x as i32, sprite_y as i32, sprite_w, sprite_h),
        &mut self.pixel_data_surface,
        sdl2::rect::Rect::new(x, y, sprite_w, sprite_h),
      )
      .unwrap();
  }

  pub fn present(&mut self) {
    let screen_buffer = &mut self.screen_buffer;
    let screen_width = self.config.screen_size.x;
    self.pixel_data_surface.with_lock(|surf: &[u8]| {
      screen_buffer
        .update(None, surf, (screen_width * 4) as usize)
        .unwrap();
    });
    let scaled_screen_size = self.config.screen_size * self.config.scale;
    self
      .canvas
      .copy(
        &self.screen_buffer,
        None,
        sdl2::rect::Rect::new(
          (self.window_size.x as i32 - scaled_screen_size.x as i32) / 2,
          (self.window_size.y as i32 - scaled_screen_size.y as i32) / 2,
          scaled_screen_size.x,
          scaled_screen_size.y,
        ),
      )
      .unwrap();
    self.canvas.present();
  }

  pub fn get_config(&self) -> &Config {
    &self.config
  }
}

pub enum HorPos {
  Left,
  Right,
  Center,
  Abs { x: i32 },
}

pub enum VertPos {
  Top,
  Bottom,
  Center,
  Abs { y: i32 },
}

fn surface_from_strvec<'a>(palette: &Palette, data: &[&str]) -> sdl2::surface::Surface<'a> {
  let width = data[0].len() as u32;
  let height = data.len() as u32;
  let mut surface =
    sdl2::surface::Surface::new(width, height, sdl2::pixels::PixelFormatEnum::ABGR8888).unwrap();
  surface.with_lock_mut(|surf: &mut [u8]| {
    for x in 0..width {
      for y in 0..height {
        let off = (x + y * width) as usize * 4;
        let ch = data[y as usize].as_bytes()[x as usize] as char;
        let color = palette.get(&ch).unwrap_or(&TRANSPARENT);
        surf[off + 0] = color.r;
        surf[off + 1] = color.g;
        surf[off + 2] = color.b;
        surf[off + 3] = color.a;
      }
    }
  });
  surface
}

fn load_surface<'a>(file_path: &str) -> sdl2::surface::Surface<'a> {
  use sdl2::image::ImageRWops;
  sdl2::rwops::RWops::from_file(file_path, "r")
    .unwrap()
    .load()
    .unwrap()
}

pub const FONT_WIDTH: u32 = 4;
pub const FONT_HEIGHT: u32 = 5;

#[rustfmt::skip]
const FONT_DATA: &[&str] = &[
// L   L   L   L   L   L   L   L   L   L   L
  " oo    o oo  oo o   oooo ooooooo oo  oo ",
  "o  o   oo  oo  oo   o   o      oo  oo  o",
  "o  o   o  o   o o o  oo ooo   o  oo  ooo",
  "o  o   o o  o  ooooo   oo  o o  o  o   o",
  " oo    ooooo oo   o ooo  oo o    oo  oo ",

  " oo ooo  oo ooo oooooooo oo o  o  o    o",
  "o  oo  oo  oo  oo   o   o  oo  o  o    o",
  "ooooooo o   o  oooo ooo o   oooo  o    o",
  "o  oo  oo  oo  oo   o   o ooo  o  o o  o",
  "o  oooo  oo ooo ooooo    oo o  o  o  oo ",

  "o  oo   o  oo  o oo ooo  oo ooo  ooooooo",
  "o o o   oooooo oo  oo  oo  oo  oo     o ",
  "oo  o   oo oo ooo  oooo o  oooo  oo   o ",
  "o o o   o  oo  oo  oo   o ooo  o   o  o ",
  "o  oooooo  oo  o oo o    oooo  oooo   o ",

  "o  oo  oo  oo  oo  ooooo                ",
  "o  oo  oo  oo  oo  o   o                ",
  "o  oo  oo oo oo  oo  oo                 ",
  "o  o o o ooo oo   o o                   ",
  " oo   o  oo o  o  o oooo                ",
];
