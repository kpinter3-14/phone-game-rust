use sdl2::event::Event;
use sdl2::rect::Rect;
use std::collections::HashSet;
use std::convert::TryInto;

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

pub fn run<S, I, U, R, H>(state: S, init: I, update: U, render: R, handle_event: H)
where
  I: Fn(&mut GContext),
  U: Fn(S, i32) -> S,
  R: Fn(&mut GContext, &S),
  H: Fn(S, &sdl2::event::Event) -> S,
{
  const SCREEN_SIZE: V2U = V2U::new(84, 48);
  const SCALE: u32 = 16;
  let mut gcontext = GContext::new(SCREEN_SIZE, SCALE);
  init(&mut gcontext);

  game_loop(&mut gcontext, state, update, render, handle_event);
}

fn game_loop<S, U, R, H>(gcontext: &mut GContext, state: S, update: U, render: R, handle_event: H)
where
  U: Fn(S, i32) -> S,
  R: Fn(&mut GContext, &S),
  H: Fn(S, &sdl2::event::Event) -> S,
{
  const TICK_INTERVAL: u32 = 50;
  let mut state = state;
  let mut ms_since_start_last_frame = 0;
  let mut ms_until_game_tick = 0;
  let mut game_tick_counter = 0;
  while !gcontext.want_to_quit {
    let ms_since_start = gcontext.timer_subsystem.ticks();
    let delta_ticks = ms_since_start - ms_since_start_last_frame;
    ms_since_start_last_frame = ms_since_start;

    for event in gcontext.event_pump.poll_iter() {
      handle_system_events(
        &mut gcontext.want_to_quit,
        &mut gcontext.window_size,
        &mut gcontext.pressed_keys,
        &event,
      );
      state = handle_event(state, &event);
    }

    ms_until_game_tick += delta_ticks;
    while ms_until_game_tick > TICK_INTERVAL {
      ms_until_game_tick -= TICK_INTERVAL;
      state = update(state, game_tick_counter);
      game_tick_counter += 1;
    }

    gcontext.reset_screen();
    render(gcontext, &mut state);
    gcontext.present();
  }
}

fn handle_system_events(
  want_to_quit: &mut bool,
  window_size: &mut V2U,
  pressed_keys: &mut HashSet<sdl2::keyboard::Keycode>,
  event: &sdl2::event::Event,
) {
  match *event {
    Event::Quit { .. } => *want_to_quit = true,
    Event::Window {
      win_event: sdl2::event::WindowEvent::Resized(w, h),
      ..
    } => *window_size = V2U::new(w.try_into().unwrap(), h.try_into().unwrap()),
    Event::KeyDown {
      keycode: Some(keycode),
      ..
    } => {
      pressed_keys.insert(keycode);
    }
    Event::KeyUp {
      keycode: Some(keycode),
      ..
    } => {
      pressed_keys.remove(&keycode);
    }
    _ => {}
  }
}

pub struct GContext<'a> {
  pub event_pump: sdl2::EventPump,
  pub texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext>,
  pub screen_buffer: sdl2::render::Texture,
  pub pixel_data_surface: sdl2::surface::Surface<'a>,
  pub timer_subsystem: sdl2::TimerSubsystem,
  pub canvas: sdl2::render::Canvas<sdl2::video::Window>,
  screen_size: V2U,
  scale: u32,
  pub window_size: V2U,
  pub camera: P2I,
  font_sprite: sdl2::surface::Surface<'a>,
  sprite_store: Store<sdl2::surface::Surface<'a>>,
  pub want_to_quit: bool,
  pub pressed_keys: HashSet<sdl2::keyboard::Keycode>,
}

const TRANSPARENT_COLOR: sdl2::pixels::Color = sdl2::pixels::Color::RGBA(0, 0, 0, 0);
const DARK_COLOR: sdl2::pixels::Color = sdl2::pixels::Color::RGB(67, 82, 61);
const BRIGHT_COLOR: sdl2::pixels::Color = sdl2::pixels::Color::RGB(199, 240, 216);

impl<'a> GContext<'a> {
  pub fn new(screen_size: V2U, scale: u32) -> GContext<'a> {
    let sdl_context = sdl2::init().unwrap();
    let event_pump = sdl_context.event_pump().unwrap();
    let _sdl_image_context = sdl2::image::init(sdl2::image::InitFlag::PNG).unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let timer_subsystem = sdl_context.timer().unwrap();
    let window_size = screen_size * scale;

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
        screen_size.x,
        screen_size.y,
      )
      .unwrap();

    let sprite_store = Store::new();

    let pixel_data_surface = sdl2::surface::Surface::new(
      screen_size.x,
      screen_size.y,
      sdl2::pixels::PixelFormatEnum::BGR888,
    )
    .unwrap();

    let font_sprite = surface_from_strvec(FONT_DATA);

    GContext {
      event_pump,
      timer_subsystem,
      texture_creator,
      screen_buffer,
      pixel_data_surface,
      canvas,
      screen_size,
      scale,
      window_size,
      camera: P2I::new(0, 0),
      font_sprite,
      sprite_store,
      want_to_quit: false,
      pressed_keys: HashSet::new(),
    }
  }

  pub fn reset_screen(&mut self) {
    self
      .pixel_data_surface
      .fill_rect(None, BRIGHT_COLOR)
      .unwrap();
  }

  fn set_pixel(&mut self, x: i32, y: i32, color: sdl2::pixels::Color) {
    if x < 0 || self.screen_size.x as i32 <= x || y < 0 || self.screen_size.y as i32 <= y {
      return;
    }
    let screen_width = self.screen_size.x as i32;
    self.pixel_data_surface.with_lock_mut(|surf: &mut [u8]| {
      let off = (x + y * screen_width) as usize * 4;
      surf[off + 0] = color.b;
      surf[off + 1] = color.g;
      surf[off + 2] = color.r;
    });
  }

  pub fn set_bright(&mut self, x: i32, y: i32) {
    self.set_pixel(x, y, BRIGHT_COLOR);
  }

  pub fn set_dark(&mut self, x: i32, y: i32) {
    self.set_pixel(x, y, DARK_COLOR);
  }

  pub fn draw_dark_rect(&mut self, x: i32, y: i32, w: u32, h: u32) {
    self
      .pixel_data_surface
      .fill_rect(sdl2::rect::Rect::new(x, y, w, h), DARK_COLOR)
      .unwrap();
  }

  pub fn draw_text(&mut self, x: i32, y: i32, text: &str) {
    let mut i = 0;
    for ch in text.chars() {
      let ix = ch as u8 - '0' as u8;
      self
        .font_sprite
        .blit(
          Rect::new(ix as i32 * FONT_WIDTH as i32, 0, FONT_WIDTH, FONT_HEIGHT),
          &mut self.pixel_data_surface,
          sdl2::rect::Rect::new(x + i * (1 + FONT_WIDTH as i32), y, FONT_WIDTH, 5),
        )
        .unwrap();
      i += 1;
    }
  }

  pub fn add_sprite(&mut self, sprite_name: &str, data: Vec<&str>) {
    self
      .sprite_store
      .insert(sprite_name.to_string(), surface_from_strvec(&data));
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
    let screen_width = self.screen_size.x;
    self.pixel_data_surface.with_lock(|surf: &[u8]| {
      screen_buffer
        .update(None, surf, (screen_width * 4) as usize)
        .unwrap();
    });
    let scaled_screen_size = self.screen_size * self.scale;
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

fn surface_from_strvec<'a>(data: &[&str]) -> sdl2::surface::Surface<'a> {
  let width = data[0].len() as u32;
  let height = data.len() as u32;
  let mut surface =
    sdl2::surface::Surface::new(width, height, sdl2::pixels::PixelFormatEnum::ABGR8888).unwrap();
  surface.with_lock_mut(|surf: &mut [u8]| {
    for x in 0..width {
      for y in 0..height {
        let off = (x + y * width) as usize * 4;
        let ch = data[y as usize].as_bytes()[x as usize] as char;
        let color = match ch {
          ' ' => TRANSPARENT_COLOR,
          '_' => BRIGHT_COLOR,
          _ => DARK_COLOR,
        };
        surf[off + 0] = color.r;
        surf[off + 1] = color.g;
        surf[off + 2] = color.b;
        surf[off + 3] = color.a;
      }
    }
  });
  surface
}

const FONT_WIDTH: u32 = 4;
const FONT_HEIGHT: u32 = 5;

#[rustfmt::skip]
const FONT_DATA: &[&str] = &[
// L   L   L   L   L   L   L   L   L   L   L
  " ##    # ##  ## #   #### ####### ##  ## ",
  "#  #   ##  ##  ##   #   #      ##  ##  #",
  "#  #   #  #   # # #  ## ###   #  ##  ###",
  "#  #   # #  #  #####   ##  # #  #  #   #",
  " ##    ##### ##   # ###  ## #    ##  ## ",
];
