use crate::config::*;
use crate::types::*;
use crate::Config;
use crate::KeyStatus;
use sdl2::rect::Rect;

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
  sprite_store: Store<sdl2::surface::Surface<'a>>,
  pub want_to_quit: bool,
  pub key_status: KeyStatus,
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

    let sprite_store = Store::new();

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
      sprite_store,
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
    let r2: i32 = r * r;
    let mut x: i32 = 0;
    let mut y: i32 = r;
    let mut y2: i32 = y * y;
    let mut dy2: i32 = 2 * y - 1;
    let mut sum: i32 = r2;

    while x <= y {
      if x % m == 0 {
        self.set_pixel(cx + x, cy + y, color);
        self.set_pixel(cx + x, cy - y, color);
        self.set_pixel(cx - x, cy + y, color);
        self.set_pixel(cx - x, cy - y, color);
        self.set_pixel(cx + y, cy + x, color);
        self.set_pixel(cx + y, cy - x, color);
        self.set_pixel(cx - y, cy + x, color);
        self.set_pixel(cx - y, cy - x, color);
      }

      sum -= 1 + x * 2;
      x += 1;
      if sum <= y2 {
        y -= 1;
        y2 -= dy2;
        dy2 -= 2;
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
    lines: &[T],
    background_color: sdl2::pixels::Color,
  ) {
    let dialogue_x = 2 * 8;
    let dialogue_y = (self.get_config().screen_size.y as i32 / 8 - lines.len() as i32) / 2 * 8;
    self.draw_rect(
      dialogue_x,
      dialogue_y,
      10 * 8,
      lines.len() as u32 * 8,
      background_color,
    );
    let mut ix = 0;
    for line in lines {
      self.draw_text(dialogue_x + 1, dialogue_y + ix * 8 + 1, line.as_ref());
      ix += 1;
    }
  }

  pub fn add_sprite(&mut self, sprite_name: &str, data: Vec<&str>) {
    self.sprite_store.insert(
      sprite_name.to_string(),
      surface_from_strvec(&self.config.palette, &data),
    );
  }

  pub fn draw_sprite(&mut self, x: i32, y: i32, sprite_name: &str) {
    let sprite = self.sprite_store.get(sprite_name).unwrap();
    sprite
      .blit(
        None,
        &mut self.pixel_data_surface,
        sdl2::rect::Rect::new(x, y, sprite.width(), sprite.height()),
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
