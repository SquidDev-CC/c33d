//! Texture loading and mapping for blocks.

use anyhow::{anyhow, Result};
use tinybmp::RawBmp;

use crate::buffer::Colour;
use crate::ray::{Hit, Plane};
use crate::world::Block;

const WIDTH: usize = 8;
const HEIGHT: usize = 8;

type Image = Vec<Colour>;

/// The default "background" colour, used when no blocks are "under" that pixel
/// and so open sky should be shown instead.
///
/// > Do you love the colour of the sky?
pub const DEFAULT_COLOUR: Colour = 9;

fn get_colour(c: u32) -> Result<Colour> {
  match c {
    // White
    0xf0f0f0 => Ok(0),
    // Green
    0x73b349 => Ok(1),
    0x5f9f35 => Ok(2),
    0x509026 => Ok(3),
    // Brown
    0x966c4a => Ok(4),
    0x79553a => Ok(5),
    0x593d29 => Ok(6),
    // Blue
    0x3266cc => Ok(7),
    0x4c32cc => Ok(8),
    // Grey
    0x8f8f8f => Ok(10),
    0x747474 => Ok(11),
    0x686868 => Ok(12),

    c => Err(anyhow!("Unknown colour {}", c)),
  }
}

fn load_texture(bytes: &[u8]) -> Result<Image> {
  let bitmap = match RawBmp::from_slice(bytes) {
    Err(e) => return Err(anyhow!("Failed to parse bitmap: {:?}", e)),
    Ok(x) => x,
  };

  let size = bitmap.size();
  if size.width != (WIDTH as u32) || size.height != (HEIGHT as u32) {
    return Err(anyhow!(
      "Image should be {}x{} but is {}x{}",
      WIDTH,
      HEIGHT,
      size.width,
      size.height
    ));
  };

  let mut pixels = vec![0; WIDTH * HEIGHT];
  for pixel in bitmap.pixels() {
    let colour = get_colour(pixel.color)?;
    pixels[(pixel.position.x as usize) + (pixel.position.y as usize) * WIDTH] = colour;
  }

  Ok(pixels)
}

/// All textures loaded by the game. Typically each block has three textures, one
/// for each axis.
///
///  - Top: This is the brightest of the three textures. We assume the bottom of
///    a block is never visible, hence (right now at least) not having a separate
///    texture for the bottom.
///  - Front/Back (z-axis): Slightly dimmer than the top.
///  - Left/Right (x-axis): The darkest of the three textures.
pub struct Textures {
  water: Image,

  dirt_x: Image,
  dirt_y: Image,
  dirt_z: Image,

  grass_x: Image,
  grass_y: Image,
  grass_z: Image,

  stone_x: Image,
  stone_y: Image,
  stone_z: Image,
}

impl Textures {
  /// Load all textures.
  pub fn new() -> Result<Textures> {
    let textures = Textures {
      water: load_texture(include_bytes!("../../texture/water.bmp"))?,

      dirt_x: load_texture(include_bytes!("../../texture/dirt_x.bmp"))?,
      dirt_y: load_texture(include_bytes!("../../texture/dirt_y.bmp"))?,
      dirt_z: load_texture(include_bytes!("../../texture/dirt_z.bmp"))?,

      grass_x: load_texture(include_bytes!("../../texture/grass_x.bmp"))?,
      grass_y: load_texture(include_bytes!("../../texture/grass_top.bmp"))?,
      grass_z: load_texture(include_bytes!("../../texture/grass_z.bmp"))?,

      stone_x: load_texture(include_bytes!("../../texture/stone_x.bmp"))?,
      stone_y: load_texture(include_bytes!("../../texture/stone_y.bmp"))?,
      stone_z: load_texture(include_bytes!("../../texture/stone_z.bmp"))?,
    };
    Ok(textures)
  }

  /// Get the colour under a particular ray trace collision. This looks up the
  /// block and axis to find the texture, and then maps that to a pixel within
  /// the texture.
  pub fn get_colour(&self, hit: Hit) -> Colour {
    let (x, y) = hit.offset;
    debug_assert!((0.0..=1.0).contains(&x) && (0.0..=1.0).contains(&y));

    let x = ((x * (WIDTH as f64)).floor() as usize).clamp(0, WIDTH - 1);
    let y = ((y * (HEIGHT as f64)).floor() as usize).clamp(0, HEIGHT - 1);
    let idx = x + y * WIDTH;

    match (hit.block, hit.side) {
      (Block::Air, _) => DEFAULT_COLOUR,
      (Block::Water, _) => self.water[idx],

      (Block::Dirt, Plane::X) => self.dirt_x[idx],
      (Block::Dirt, Plane::Y) => self.dirt_y[idx],
      (Block::Dirt, Plane::Z) => self.dirt_z[idx],

      (Block::Grass, Plane::X) => self.grass_x[idx],
      (Block::Grass, Plane::Y) => self.grass_y[idx],
      (Block::Grass, Plane::Z) => self.grass_z[idx],

      (Block::Stone, Plane::X) => self.stone_x[idx],
      (Block::Stone, Plane::Y) => self.stone_y[idx],
      (Block::Stone, Plane::Z) => self.stone_z[idx],
    }
  }
}
