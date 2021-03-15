pub struct Arr2d<T> {
  width: u32,
  height: u32,
  data_vec: Vec<T>,
}

impl<T> Arr2d<T>
where
  T: Copy,
{
  pub fn new(width: u32, height: u32, default: T) -> Arr2d<T> {
    Arr2d {
      width,
      height,
      data_vec: vec![default; (width * height) as usize],
    }
  }

  fn valid_index(&self, x: i32, y: i32) -> Option<usize> {
    if 0 <= x && x < self.width as i32 && 0 <= y && y < self.height as i32 {
      Some(self.index(x as u32, y as u32))
    } else {
      None
    }
  }

  fn index(&self, x: u32, y: u32) -> usize {
    (x + y * self.width) as usize
  }

  pub fn get(&self, x: i32, y: i32) -> Option<&T> {
    self.valid_index(x, y).map(|index| &self.data_vec[index])
  }

  pub fn get_unsafe(&self, x: u32, y: u32) -> &T {
    &self.data_vec[self.index(x, y)]
  }

  pub fn get_mut(&mut self, x: i32, y: i32) -> Option<&mut T> {
    self
      .valid_index(x, y)
      .map(move |index| &mut self.data_vec[index])
  }

  pub fn get_mut_unsafe(&mut self, x: u32, y: u32) -> &mut T {
    let index = self.index(x, y);
    &mut self.data_vec[index]
  }

  pub fn set(&mut self, x: i32, y: i32, e: T) {
    let ix = self.index(x as u32, y as u32);
    self.data_vec[ix] = e;
  }

  pub fn set_from_function<F>(&mut self, f: F)
  where
    F: Fn(i32, i32) -> T,
  {
    for x in 0..self.width as i32 {
      for y in 0..self.height as i32 {
        self.set(x, y, f(x, y));
      }
    }
  }

  pub fn update<F>(&mut self, f: F)
  where
    F: Fn(T) -> T,
  {
    for x in 0..self.width {
      for y in 0..self.height {
        let x = self.get_mut_unsafe(x, y);
        *x = f(*x);
      }
    }
  }

  pub fn data(&self) -> &[T] {
    &self.data_vec
  }

  pub fn width(&self) -> u32 {
    self.width
  }

  pub fn height(&self) -> u32 {
    self.height
  }
}
