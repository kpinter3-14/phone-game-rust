use game_lib::types::*;

#[derive(Copy, Clone, PartialEq)]
pub struct Entity {
  pub pos: V2I,
  pub entity_type: EntityType,
}

#[derive(Copy, Clone, PartialEq)]
pub enum EntityType {
  Item { item_type: ItemType },
  Enemy { enemy_type: EnemyType },
}

#[derive(Copy, Clone, PartialEq)]
pub enum ItemType {
  Cherry,
  Coin,
}

#[derive(Copy, Clone, PartialEq)]
pub enum EnemyType {
  Gin,
}

impl std::fmt::Display for EntityType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      EntityType::Item { item_type } => write!(f, "{}", item_type),
      EntityType::Enemy { enemy_type } => write!(f, "{}", enemy_type),
    }
  }
}

impl std::fmt::Display for ItemType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        ItemType::Cherry => "cherry",
        ItemType::Coin => "coin",
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

pub type EntityId = u32;
