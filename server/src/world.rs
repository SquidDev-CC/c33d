//! Defines the available blocks and a "world" containing those blocks.

use serde::de::Error;
use serde::{Deserialize, Deserializer};

/// A block in the world.
#[derive(Copy, Clone)]
pub enum Block {
  Air,
  Dirt,
  Grass,
  Stone,
  Water,
}

impl Block {
  /// Parse a block from a character. Returns [`None`] when an invalid character
  /// is given.
  ///
  /// This is used when deserialising a world.
  fn parse(c: char) -> Option<Block> {
    match c {
      ' ' => Some(Block::Air),
      'd' => Some(Block::Dirt),
      'g' => Some(Block::Grass),
      's' => Some(Block::Stone),
      'w' => Some(Block::Water),
      _ => None,
    }
  }
}

/// A world, containing a 3D grid of blocks.
pub struct World {
  pub width: usize,
  pub height: usize,
  pub depth: usize,
  blocks: Vec<Block>,
}

impl World {
  /// Construct a new world with the given dimensions. Blocks can then be
  /// modified with [`World::set`].
  pub fn new(width: usize, height: usize, depth: usize) -> World {
    World { width, height, depth, blocks: vec![Block::Air; width * height * depth] }
  }

  /// Get the block at the given position. Panics if the block is outside this
  /// world.
  pub fn get(&self, x: usize, y: usize, z: usize) -> Block {
    debug_assert!(x < self.width && y < self.height && z < self.depth);
    self.blocks[x + y * self.width + z * self.height * self.width]
  }

  /// Set the block at the given position. Panics if the block is outside this
  /// world.
  pub fn set(&mut self, x: usize, y: usize, z: usize, block: Block) {
    debug_assert!(x < self.width && y < self.height && z < self.depth);
    self.blocks[x + y * self.width + z * self.height * self.width] = block;
  }
}

impl<'de> Deserialize<'de> for World {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let contents: Vec<Vec<String>> = Deserialize::deserialize(deserializer)?;

    let width = contents[0][0].len();
    let height = contents.len();
    let depth = contents[0].len();

    let mut world = World::new(width, height, depth);

    for (y, plane) in contents.iter().enumerate() {
      for (z, row) in plane.iter().enumerate() {
        for (x, cell) in row.chars().enumerate() {
          match Block::parse(cell) {
            None => return Err(D::Error::custom(format!("Unknown block {}", cell))),
            Some(block) => world.set(x, y, z, block),
          }
        }
      }
    }

    Ok(world)
  }
}
