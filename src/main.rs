mod game;
mod game_lib;

use game::State;
use game_lib::GContext;

fn main() {
  game_lib::run(
    State::new(),
    game::init,
    game::update,
    game::render,
    game::handle_event,
  );
}
