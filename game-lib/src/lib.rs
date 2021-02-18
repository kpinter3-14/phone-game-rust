pub mod run;
pub use run::*;
pub mod gcontext;
pub use gcontext::*;
pub mod config;
pub use config::*;

pub mod types;
pub use types::*;
pub mod key_status;
pub use key_status::*;
pub mod incmap;
pub use incmap::*;

pub fn get_arg(arg_name: &str) -> Option<String> {
  let prefix = String::new() + "--" + arg_name + "=";
  std::env::args()
    .find(|s| s.starts_with(&prefix))
    .map(|s| s.strip_prefix(&prefix).map(|a| a.to_string()))
    .flatten()
}
