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
