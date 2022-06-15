//! Traces rays through a [`World`] and renders them.

use crate::buffer::{Buffer, BUF_HEIGHT, BUF_WIDTH};
use crate::texture::{Textures, DEFAULT_COLOUR};
use crate::world::{Block, World};

use log::warn;
use rayon::prelude::*;

#[derive(Debug)]
pub enum Plane {
  X,
  Y,
  Z,
}

pub struct Vec3<T> {
  pub x: T,
  pub y: T,
  pub z: T,
}

impl<T> Vec3<T> {
  pub fn new(x: T, y: T, z: T) -> Vec3<T> {
    Vec3 { x, y, z }
  }
}

pub struct Hit {
  pub block: Block,
  pub side: Plane,
  pub offset: (f64, f64),
}

fn get_dists(start: f64, direction: f64) -> (i64, i64, f64, f64) {
  let step = match direction {
    i if i == 0.0 => 0,
    i if i < 0.0 => -1,
    _ => 1,
  };

  let delta_dist = if direction == 0.0 {
    1e30
  } else {
    1.0 / direction.abs()
  };
  let map = start.trunc() as i64;

  let side_dist = delta_dist
    * (if direction > 0.0 {
      1.0 - start.fract()
    } else {
      start.fract()
    });

  (map, step, delta_dist, side_dist)
}

fn outside(value: i64, min: i64, max: i64, step: i64) -> bool {
  if step < 0 {
    value < min
  } else {
    value > max
  }
}

/** Trace a ray through the world. */
pub fn trace(world: &World, start: Vec3<f64>, direction: Vec3<f64>) -> Option<Hit> {
  let width = world.width as i64;
  let height = world.height as i64;
  let depth = world.depth as i64;

  let (mut map_x, step_x, delta_dist_x, mut side_dist_x) = get_dists(start.x, direction.x);
  let (mut map_y, step_y, delta_dist_y, mut side_dist_y) = get_dists(start.y, direction.y);
  let (mut map_z, step_z, delta_dist_z, mut side_dist_z) = get_dists(start.z, direction.z);

  loop {
    let side: Plane;
    if side_dist_x < side_dist_y {
      if side_dist_x < side_dist_z {
        side = Plane::X;
        map_x += step_x;
        side_dist_x += delta_dist_x;
      } else {
        side = Plane::Z;
        map_z += step_z;
        side_dist_z += delta_dist_z;
      }
    } else if side_dist_y < side_dist_z {
      side = Plane::Y;
      map_y += step_y;
      side_dist_y += delta_dist_y;
    } else {
      side = Plane::Z;
      map_z += step_z;
      side_dist_z += delta_dist_z;
    }

    if (0..width).contains(&map_x) && (0..height).contains(&map_y) && (0..depth).contains(&map_z) {
      match world.get(map_x as usize, map_y as usize, map_z as usize) {
        Block::Air => (),
        // TODO: Compute offset. How??
        block => {
          // Without loss of generality, pick our side to be x and face be closest to us. We have map_x == start.x +
          // direction.x * t for some t. Solving for t gives (map_x - start.x) / direction.x
          let (t, offset) = match side {
            Plane::Z => {
              let map_z = if step_z < 0 { map_z + 1 } else { map_z };
              let t = (map_z as f64 - start.z) / direction.z;
              (
                t,
                (
                  start.x + direction.x * t - map_x as f64,
                  1.0 - (start.y + direction.y * t - map_y as f64),
                ),
              )
            }
            Plane::X => {
              let map_x = if step_x < 0 { map_x + 1 } else { map_x };
              let t = (map_x as f64 - start.x) / direction.x;
              (
                t,
                (
                  start.z + direction.z * t - map_z as f64,
                  1.0 - (start.y + direction.y * t - map_y as f64),
                ),
              )
            }
            Plane::Y => {
              let map_y = if step_y < 0 { map_y + 1 } else { map_y };
              let t = (map_y as f64 - start.y) / direction.y;
              (
                t,
                (
                  start.x + direction.x * t - map_x as f64,
                  start.z + direction.z * t - map_z as f64,
                ),
              )
            }
          };

          if offset.0 > 1.0 || offset.1 > 1.0 || offset.0 < 0.0 || offset.1 < 0.0 {
            warn!(
            "Tracing ray from {},{},{} with {},{},{}. Collides at {},{},{} (t={}, side={:?}) => {}, {}, {} {:?}",
            start.x,
            start.y,
            start.z,
            direction.x,
            direction.y,
            direction.z,
            map_x,
            map_y,
            map_z,
            t,
            side,
            start.x + direction.x * t,
            start.y + direction.y * t,
            start.z + direction.z * t,
            offset
          );
          }

          return Some(Hit { block, side, offset });
        }
      }
    } else if outside(map_x, 0, width, step_x)
      || outside(map_y, 0, height, step_y)
      || outside(map_z, 0, depth, step_z)
    {
      return None;
    }
  }
}

pub fn render(
  world: &World,
  textures: &Textures,
  offset: Vec3<f64>,
  position: Vec3<f64>,
) -> Buffer {
  let mut buffer = Buffer::new();
  buffer
    .as_mut_slice()
    .par_chunks_exact_mut(BUF_WIDTH as usize)
    .enumerate()
    .for_each(|(y, out)| {
      for x in 0..BUF_WIDTH {
        let ox = (1.0 - ((x as f64) / (BUF_WIDTH as f64))) * 8.0;
        let oy = (1.0 - ((y as f64) / (BUF_HEIGHT as f64))) * 6.0;

        out[x as usize] = match trace(
          world,
          Vec3::new(ox + offset.x, oy + offset.y, offset.z),
          Vec3::new(ox - position.x, oy - position.y, -position.z),
        ) {
          None => DEFAULT_COLOUR,
          Some(hit) => textures.get_colour(hit),
        }
      }
    });
  buffer
}
