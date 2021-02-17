use std::collections::HashMap;

pub struct IncMap<T> {
  next_id: u32,
  data: HashMap<u32, T>,
}

impl<T> IncMap<T> {
  pub fn new() -> IncMap<T> {
    IncMap {
      next_id: 0,
      data: HashMap::new(),
    }
  }

  pub fn insert(&mut self, e: T) {
    self.data.insert(self.next_id, e);
    self.next_id += 1;
  }

  pub fn get(&self, id: u32) -> Option<&T> {
    self.data.get(&id)
  }

  pub fn get_mut(&mut self, id: u32) -> Option<&mut T> {
    self.data.get_mut(&id)
  }

  pub fn remove(&mut self, id: u32) {
    self.data.remove(&id);
  }
}

impl<'a, T> IntoIterator for &'a IncMap<T> {
  type Item = (&'a u32, &'a T);
  type IntoIter = std::collections::hash_map::Iter<'a, u32, T>;

  fn into_iter(self) -> Self::IntoIter {
    self.data.iter()
  }
}
