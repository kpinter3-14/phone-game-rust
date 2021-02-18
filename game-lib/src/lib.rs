use sdl2::event::Event;
use sdl2::rect::Rect;
use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::TryInto;

pub mod incmap;
pub use incmap::*;

pub mod types {
  pub type V2U = cgmath::Vector2<u32>;
  pub type V2I = cgmath::Vector2<i32>;
  pub type V2F = cgmath::Vector2<f32>;
  pub type V3I = cgmath::Vector3<i32>;
  pub type V3F = cgmath::Vector3<f32>;
  pub type P2I = cgmath::Point2<i32>;
  pub type P2U = cgmath::Point2<u32>;
  pub type P2F = cgmath::Point2<f32>;
  pub type P3F = cgmath::Point3<f32>;
  pub type P4U8 = cgmath::Vector4<u8>;
  pub type M4F = cgmath::Matrix4<f32>;
  pub type Store<T> = std::collections::HashMap<String, T>;

  #[derive(Copy, Clone)]
  pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
  }

  impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Rect {
      Rect { x, y, w, h }
    }

    pub fn intersects(&self, other: &Rect) -> bool {
      let x_overlaps = !(self.x > other.x + other.w || self.x + self.w < other.x);
      let y_overlaps = !(self.y > other.y + other.h || self.y + self.h < other.y);
      x_overlaps && y_overlaps
    }
  }
}

use types::*;

type Palette = HashMap<char, sdl2::pixels::Color>;

pub struct Config {
  pub scale: u32,
  pub screen_size: V2U,
  pub font_color: sdl2::pixels::Color,
  pub background_color: sdl2::pixels::Color,
  pub palette: Palette,
}

pub fn run<S: 'static, I, U: 'static, R: 'static, H: 'static>(
  config: Config,
  state: S,
  init: I,
  update: U,
  render: R,
  handle_event: H,
) where
  I: Fn(&mut GContext),
  U: Fn(&mut S, &KeyStatus, u32),
  R: Fn(&mut GContext, &S),
  H: Fn(&mut S, &sdl2::event::Event),
{
  let game = Game::new(config, state, init, update, render, handle_event);
  emscripten_main_loop::run(game);
}

pub struct Game<'a, S, U, R, H>
where
  U: Fn(&mut S, &KeyStatus, u32),
  R: Fn(&mut GContext, &S),
  H: Fn(&mut S, &sdl2::event::Event),
{
  gcontext: GContext<'a>,
  state: S,
  update: U,
  render: R,
  handle_event: H,
}

const TICK_INTERVAL: u32 = 50;

impl<'a, S, U, R, H> Game<'a, S, U, R, H>
where
  U: Fn(&mut S, &KeyStatus, u32),
  R: Fn(&mut GContext, &S),
  H: Fn(&mut S, &sdl2::event::Event),
{
  pub fn new<I>(
    config: Config,
    state: S,
    init: I,
    update: U,
    render: R,
    handle_event: H,
  ) -> Game<'a, S, U, R, H>
  where
    I: Fn(&mut GContext),
  {
    let mut gcontext = GContext::new(config);
    init(&mut gcontext);
    Game {
      gcontext,
      state,
      update,
      render,
      handle_event,
    }
  }

  pub fn game_loop(&mut self) {
    let ms_since_start = self.gcontext.timer_subsystem.ticks();
    let delta_ticks = ms_since_start - self.gcontext.ms_since_start_last_frame;
    self.gcontext.ms_since_start_last_frame = ms_since_start;

    for event in self.gcontext.event_pump.poll_iter() {
      handle_system_events(
        &mut self.gcontext.want_to_quit,
        &mut self.gcontext.window_size,
        &mut self.gcontext.key_status,
        &event,
      );
      (self.handle_event)(&mut self.state, &event);
    }

    self.gcontext.ms_until_game_tick += delta_ticks;
    while self.gcontext.ms_until_game_tick > TICK_INTERVAL {
      self.gcontext.ms_until_game_tick -= TICK_INTERVAL;
      (self.update)(
        &mut self.state,
        &self.gcontext.key_status,
        self.gcontext.game_tick_counter,
      );
      self.gcontext.game_tick_counter += 1;
    }

    self.gcontext.reset_screen();
    (self.render)(&mut self.gcontext, &mut self.state);
    self.gcontext.present();
  }
}

impl<'a, S, U, R, H> emscripten_main_loop::MainLoop for Game<'a, S, U, R, H>
where
  U: Fn(&mut S, &KeyStatus, u32),
  R: Fn(&mut GContext, &S),
  H: Fn(&mut S, &sdl2::event::Event),
{
  fn main_loop(&mut self) -> emscripten_main_loop::MainLoopEvent {
    if self.gcontext.want_to_quit {
      emscripten_main_loop::MainLoopEvent::Terminate
    } else {
      self.game_loop();
      emscripten_main_loop::MainLoopEvent::Continue
    }
  }
}

fn handle_system_events(
  want_to_quit: &mut bool,
  window_size: &mut V2U,
  key_status: &mut KeyStatus,
  event: &sdl2::event::Event,
) {
  match *event {
    Event::Quit { .. } => *want_to_quit = true,
    #[cfg(not(target_os = "emscripten"))]
    Event::Window {
      win_event: sdl2::event::WindowEvent::Resized(w, h),
      ..
    } => *window_size = V2U::new(w.try_into().unwrap(), h.try_into().unwrap()),
    Event::KeyDown {
      keycode: Some(keycode),
      ..
    } => {
      key_status.set_key_pressed(keycode, true);
    }
    Event::KeyUp {
      keycode: Some(keycode),
      ..
    } => {
      key_status.set_key_pressed(keycode, false);
    }
    _ => {}
  }
}

pub struct KeyStatus {
  key_status: HashSet<sdl2::keyboard::Keycode>,
}

impl KeyStatus {
  fn new() -> KeyStatus {
    KeyStatus {
      key_status: HashSet::new(),
    }
  }

  fn set_key_pressed(&mut self, keycode: sdl2::keyboard::Keycode, pressed: bool) {
    if pressed {
      self.key_status.insert(keycode);
    } else {
      self.key_status.remove(&keycode);
    }
  }

  pub fn is_key_pressed(&self, keycode: sdl2::keyboard::Keycode) -> bool {
    self.key_status.contains(&keycode)
  }
}

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

pub const TRANSPARENT: sdl2::pixels::Color = sdl2::pixels::Color::RGBA(0, 0, 0, 0);
pub const DARK_COLOR: sdl2::pixels::Color = sdl2::pixels::Color::RGB(67, 82, 61);
pub const BRIGHT_COLOR: sdl2::pixels::Color = sdl2::pixels::Color::RGB(199, 240, 216);

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

pub fn get_arg(arg_name: &str) -> Option<String> {
  let prefix = String::new() + "--" + arg_name + "=";
  std::env::args()
    .find(|s| s.starts_with(&prefix))
    .map(|s| s.strip_prefix(&prefix).map(|a| a.to_string()))
    .flatten()
}

const FONT_WIDTH: u32 = 4;
const FONT_HEIGHT: u32 = 5;

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

  "o  oo   o  oo  o oo ooo  oo ooo  oo oooo",
  "o o o   oooooo oo  oo  oo  oo  oo  o  o ",
  "oo  o   oo oo ooo  oooo o  oooo  o    o ",
  "o o o   o  oo  oo  oo   o ooo  oo  o  o ",
  "o  oooooo  oo  o oo o    oooo  o oo   o ",

  "o  oo  oo  oo  oo  ooooo                ",
  "o  oo  oo  oo  oo  o   o                ",
  "o  oo  oo oo oo  oo  oo                 ",
  "o  o o o ooo oo   o o                   ",
  " oo   o  oo o  o  o oooo                ",
];
