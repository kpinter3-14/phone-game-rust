mod game;
mod game_lib;

use game::State;

fn get_arg(arg_name: &str) -> Option<String> {
  let prefix = String::new() + "--" + arg_name + "=";
  std::env::args()
    .find(|s| s.starts_with(&prefix))
    .map(|s| s.strip_prefix(&prefix).map(|a| a.to_string()))
    .flatten()
}

fn main() {
  let control_mode = get_arg("control")
    .map(|s| match s.as_str() {
      "toggle" => Some(game::ControlScheme::Toggle),
      "hold" => Some(game::ControlScheme::Hold),
      _ => None,
    })
    .flatten()
    .unwrap_or(game::ControlScheme::Toggle);

  let scale = get_arg("scale")
    .map(|s| s.parse().ok())
    .flatten()
    .unwrap_or(16);

  game_lib::run(
    scale,
    State::new(control_mode),
    game::init,
    game::update,
    game::render,
    game::handle_event,
  );
}
