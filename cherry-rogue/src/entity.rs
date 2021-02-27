use game_lib::types::*;

#[derive(Copy, Clone, PartialEq)]
pub struct Item {
  pub pos: P2I,
  pub item_type: ItemType,
}

#[derive(Copy, Clone, PartialEq)]
pub struct Enemy {
  pub pos: P2I,
  pub enemy_type: EnemyType,
}

#[derive(Copy, Clone, PartialEq)]
pub enum ItemType {
  Cherry,
  Coin,
  DeadGin,
}

#[derive(Copy, Clone, PartialEq)]
pub enum EnemyType {
  Gin,
}

impl std::fmt::Display for ItemType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        ItemType::Cherry => "cherry",
        ItemType::Coin => "coin",
        ItemType::DeadGin => "dead gin",
      }
    )
  }
}

impl std::fmt::Display for EnemyType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        EnemyType::Gin => "gin",
      }
    )
  }
}

pub type ItemId = u32;
pub type EnemyId = u32;
