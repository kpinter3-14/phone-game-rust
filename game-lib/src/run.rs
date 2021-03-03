use crate::config::*;
use crate::gcontext::*;
use crate::key_status::*;
use crate::types::*;
use sdl2::event::Event;
use std::convert::TryInto;

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
        &self.gcontext.canvas,
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
  canvas: &sdl2::render::Canvas<sdl2::video::Window>,
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
      #[cfg(not(target_os = "emscripten"))]
      if keycode == sdl2::keyboard::Keycode::F12 {
        GContext::take_screenshot(canvas)
      }
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
