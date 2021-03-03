use crate::types::*;
use cgmath::prelude::*;

// based on https://lodev.org/cgtutor/raycasting.html
pub fn dda<F, G>(
  tile_is_blocking: F,
  tile_step_fun: &mut G,
  pos: P2F,
  dir: V2F,
  max_dist: f32,
) -> Option<P2I>
where
  F: Fn(P2I) -> bool,
  G: FnMut(P2I) -> (),
{
  let mut cell_pos = P2I::new(pos.x as i32, pos.y as i32);

  let len = dir.magnitude();
  let delta_dist = V2F::new((len / dir.x).abs(), (len / dir.y).abs());

  let mut step = V2I::new(0, 0);
  let mut side_dist: V2F = V2F::new(0.0, 0.0);

  if dir.x < 0.0 {
    step.x = -1;
    side_dist.x = (pos.x - cell_pos.x as f32) * delta_dist.x;
  } else {
    step.x = 1;
    side_dist.x = (cell_pos.x as f32 + 1.0 - pos.x) * delta_dist.x;
  }

  if dir.y < 0.0 {
    step.y = -1;
    side_dist.y = (pos.y - cell_pos.y as f32) * delta_dist.y;
  } else {
    step.y = 1;
    side_dist.y = (cell_pos.y as f32 + 1.0 - pos.y) * delta_dist.y;
  }

  let mut done = false;
  let mut dist = 0.0;
  while !done && dist < max_dist {
    let step_x = side_dist.x < side_dist.y;
    if step_x {
      side_dist.x += delta_dist.x;
      dist = side_dist.x;
    } else {
      side_dist.y += delta_dist.y;
      dist = side_dist.y;
    }
    tile_step_fun(cell_pos);
    if tile_is_blocking(cell_pos) {
      done = true;
    } else {
      if step_x {
        cell_pos.x += step.x;
      } else {
        cell_pos.y += step.y;
      }
    }
  }

  if dist > max_dist {
    return None;
  }
  return Some(cell_pos);
}

// https://gist.github.com/badboy/6267743#using-multiplication-for-hashing
pub fn hash(a0: u32) -> u32 {
  let a1 = (a0 ^ 61) ^ (a0 >> 16);
  let a2 = a1 + (a1 << 3);
  let a3 = a2 ^ (a2 >> 4);
  let a4 = a3.wrapping_mul(0x27d4eb2d);
  let a5 = a4 ^ (a4 >> 15);
  a5
}

pub fn hash_v2(v: V2U) -> u32 {
  hash(v.x ^ (v.y << 16))
}
