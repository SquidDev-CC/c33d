// Monitors are 164 wide and 81 high. We want to avoid rendering on the edge as
// teletext characters are weird, so minus two on either side.
pub const MON_WIDTH: u32 = 164 - 2;
pub const MON_HEIGHT: u32 = 81 - 2;

pub const BUF_WIDTH: u32 = MON_WIDTH * 2;
pub const BUF_HEIGHT: u32 = MON_HEIGHT * 3;

const SIZE: usize = (BUF_WIDTH * BUF_HEIGHT) as usize;

const HEX_COLOURS: &[u8] = "0123456789abcdef".as_bytes();

pub type Colour = u8;

fn to_hex(colour: Colour) -> u8 {
  debug_assert!(colour < 16);
  HEX_COLOURS[colour as usize]
}

/// A mutable grid of pixels (each pixel being one of CC's 16 colours), which
/// can be 'drawn' to a terminal or monitor.
///
/// Buffers are (at least currently) a constant size (see [`BUF_WIDTH`],
/// [`BUF_HEIGHT`]), suitable for rendering to a max-size monitor with 1px
/// margin.
pub struct Buffer {
  colours: [Colour; SIZE],
}

impl Buffer {
  pub fn new() -> Buffer {
    Buffer { colours: [0; SIZE] }
  }

  fn get(&self, x: u32, y: u32) -> Colour {
    self.colours[(x + y * BUF_WIDTH) as usize]
  }

  pub fn as_mut_slice(&mut self) -> &mut [Colour] {
    self.colours.as_mut_slice()
  }

  /// 'Draw' the buffer.
  ///
  /// This converts it to a single string containing the whole terminal contents
  /// (text||fg||bg) concatenated for each line). This allows for fast decoding
  /// and rendering on the ComputerCraft side (it's just string.sub and blit
  /// calls).
  ///
  /// This uses CC's teletext characters to approximate the actual buffer's
  /// contents. If there are more than 2 colours in each 2x3 region, only the
  /// two most common will be used.
  pub fn draw(&self) -> Vec<u8> {
    let mut vec = vec![0; (MON_WIDTH * MON_HEIGHT * 3) as usize];

    for mon_y in 0..MON_HEIGHT {
      let y = mon_y * 3;

      for mon_x in 0..MON_WIDTH {
        let x = mon_x * 2;

        // I wish we had dependent types (or at least Ada-style arrays). Alas.
        let mut totals = [0; 16];
        let mut unique = 0;
        for dx in 0..2 {
          for dy in 0..3 {
            let colour = self.get(x + dx, y + dy) as usize;
            if totals[colour] == 0 {
              unique += 1;
            }
            totals[colour] += 1;
          }
        }

        let (text, fg, bg) = if unique == 1 {
          let (mut colour, mut best) = (0, 0);
          for (i, x) in totals.into_iter().enumerate() {
            if x > best {
              colour = i;
              best = x;
            }
          }

          (b' ', 0_u8, colour as u8)
        } else {
          let mut colours: [Colour; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
          colours.sort_by_key(|k| -totals[*k as usize]);

          // TODO: blittle-like colour similarity?
          let bg = colours[0];
          let fg = colours[1];
          let last = if self.get(x + 1, y + 2) == fg { fg } else { bg };

          let mut code: u8 = 128;
          for dx in 0..2 {
            for dy in 0..3 {
              if dx == 1 && dy == 2 {
                continue;
              }

              if self.get(x + dx, y + dy) != last {
                code |= 1 << (2 * dy + dx);
              }
            }
          }

          if last == bg {
            (code, fg, bg)
          } else {
            (code, bg, fg)
          }
        };

        vec[(mon_y * MON_WIDTH * 3 + mon_x) as usize] = text;
        vec[(mon_y * MON_WIDTH * 3 + MON_WIDTH + mon_x) as usize] = to_hex(fg);
        vec[(mon_y * MON_WIDTH * 3 + 2 * MON_WIDTH + mon_x) as usize] = to_hex(bg);
      }
    }

    vec
  }
}
