//! Graphics-related functionality.
//!
//! Contains an implementation of a PPU.

use std::fmt;

/// The colors that can be displayed by the DMG.
#[derive(Debug, PartialEq, Eq)]
pub enum Shade {
    White,
    LightGray,
    DarkGray,
    Black,

    /// A shade that is only used by sprites.
    Transparent,
}

impl Default for Shade {
    fn default() -> Shade {
        Shade::White
    }
}

impl From<u8> for Shade {
    fn from(val: u8) -> Shade {
        use self::Shade::*;

        match val {
            0 => White,
            1 => LightGray,
            2 => DarkGray,
            3 => Black,
            _ => panic!("only 0-3 correspond to valid shades"),
        }
    }
}

/// Memory managed by the PPU.
struct Memory {
    /// Background data, split into two overlapping 1024 byte maps.
    ///
    /// Each byte in the map represents an 8x8 pixel space on the display, referring to tile data
    /// stored in the Character RAM. Each total map is 32x32 tiles.
    bg_map: [u8; 0x800],

    /// Character RAM, storing 8x8 pixel tile data.
    ///
    /// Each pixel has two bits of color data, so each tile is 16 bytes long. This area is
    /// divided into signed and unsigned tiles: unsigned are numbered 0-255 at $8000-$9000.
    /// Signed tiles are numbered in two's complement from -127-128 at $87FF-$97FF.
    chram: [u8; 0x1800],

    /// Object attribute memory (OAM).
    oam: [u8; 0xA0],
}

impl Default for Memory {
    fn default() -> Memory {
        Memory {
            bg_map: [0; 0x800],
            chram: [0; 0x1800],
            oam: [0; 0xA0],
        }
    }
}

impl Memory {
    /// Return the first set of background map data from VRAM.
    fn bg1_mut(&mut self) -> &mut [u8] {
        &mut self.bg_map[0..0x3FF]
    }

    /// Return the second set of background map data from VRAM.
    fn bg2_mut(&mut self) -> &mut [u8] {
        &mut self.bg_map[0x9C00..0x9FFF]
    }
}

/// Groups information that determines if various interrupts are enabled.
#[derive(Debug, Default)]
pub struct Interrupts {
    pub hblank: bool,
    pub vblank: bool,
    pub oam: bool,
    pub ly_lyc: bool,
}

/// Core LCD settings.
#[derive(Debug, Default)]
pub struct LcdControl {
    /// Whether the LCD is operating.
    pub display_enabled: bool,

    /// True if window memory should be displayed.
    pub window_enabled: bool,

    /// True if sprites should be displayed.
    pub sprites_enabled: bool,

    /// True if the background should be displayed.
    pub background_enabled: bool,

    /// The address of the start of the window tile map.
    pub window_map_start: u16,

    /// The address of the start of the background and window tile data.
    pub window_data_start: u16,

    /// The address of the start of the background tile map.
    pub bg_map_start: u16,

    /// The size of the sprites being used. Valid values are 8x8 and 8x16.
    pub sprite_size: (u8, u8),
}

/// An X/Y coordinate pair.
#[derive(Debug, Default)]
pub struct Position {
    pub x: u8,
    pub y: u8,
}

/// The picture processing unit.
#[derive(Debug, Default)]
pub struct Ppu {
    mem: Memory,

    pub control: LcdControl,

    /// The current mode number of the PPU operation.
    ///
    /// | Mode      | Description               |
    /// | --------- | ------------------------- |
    /// | 0         | Horizontal blank          |
    /// | 1         | Vertical blank            |
    /// | 2         | Scanline (accessing OAM)  |
    /// | 3         | Scanline (accessing VRAM) |
    pub mode: u8,

    /// The number of PPU clock cycles that have been executed for the current
    /// PPU operation.
    modeclock: u32,

    /// The background palette.
    ///
    /// This array can be thought of as a map from color number to shade, where the color numbers
    /// are those used by the Background and Window tiles.
    pub bg_palette: [Shade; 4],

    /// The two object palettes.
    ///
    /// Each array can be thought of as a map from color number to shade, where the color numbers
    /// are those used by the sprite tiles. Note that color 0 is always transparent for sprites.
    pub sprite_palette: [[Shade; 4]; 2],

    /// The current line position of the PPU. The last line is 143.
    pub line: u8,

    /// The position in the 256x256 background tile map that should be displayed from the upper
    /// left.
    pub bg_scroll: Position,

    /// A value that is compared against the current line.
    ///
    /// Used by the LCDC status and LYC I/O registers.
    pub line_compare: u8,

    /// Contains whether PPU-related interrupts are enabled or disabled.
    pub interrupts: Interrupts,

    /// Indicates which background map is in use. If `false`, the first map is in use, and if
    /// `true`, the second map is in use.
    pub use_second_bg_map: bool,

    /// Indicates whether we are using the signed or unsigned tile mode.
    pub signed_tile_mode: bool,
}

impl Ppu {
    /// Creates a new picture processing unit.
    ///
    /// The initial contents of the memory are unspecified.
    pub fn new() -> Ppu {
        Ppu::default()
    }

    /// Performs one clock step of the PPU.
    pub fn step(&mut self, cycles: u32) {
        self.modeclock += cycles;

        match self.mode {
            // Horizontal blank
            0 => {
                if self.modeclock >= 204 {
                    self.modeclock = 0;
                    self.line += 1;

                    if self.line == 143 {
                        // FIXME: show the image data on the screen here

                        // Enter vertical blank mode
                        self.mode = 1;

                        debug!("set graphics mode to {}", self.mode);
                    } else {
                        // Enter scanline mode
                        self.mode = 2;

                        debug!("set graphics mode to {}", self.mode);
                    }
                }
            }

            // Vertical blank
            1 => {
                if self.modeclock >= 456 {
                    self.modeclock = 0;
                    self.line += 1;

                    // FIXME: Should this be 143?
                    if self.line > 153 {
                        // Enter scanline mode
                        self.mode = 2;
                        self.line = 0;

                        debug!("set graphics mode to {}", self.mode);
                    }
                }
            }

            // Scanline mode reading OAM
            2 => {
                if self.modeclock >= 80 {
                    // Enter scanline mode reading VRAM
                    self.modeclock = 0;
                    self.mode = 3;

                    debug!("set graphics mode to {}", self.mode);
                }
            }

            // Scanline mode reading VRAM
            3 => {
                if self.modeclock >= 172 {
                    // Enter horizontal blank mode
                    self.modeclock = 0;
                    self.mode = 0;

                    // Write a scanline to the framebuffer
                    self.renderscan();

                    debug!("set graphics mode to {}", self.mode);
                }
            }

            _ => panic!("unimplemented PPU mode: {:?}", self.mode),
        }
    }

    /// Renders the screen one line at a time. Move tile-by-tile through the line until it is
    /// complete.
    pub fn renderscan(&mut self) {
        // Figure out which background map to use
        let mut bg_map = if self.use_second_bg_map {
            self.mem.bg2_mut()
        } else {
            self.mem.bg1_mut()
        };

        // Determine the index of the start of the tile line
        let tile_line_index = (self.line + self.bg_scroll.y) / 8;
        // Determine the index of the first tile within the line
        let tile_line_offset = self.bg_scroll.x / 8;

        // Get position in tile
        let tile_y = (self.line + self.bg_scroll.y) % 8;
        let tile_x = self.bg_scroll.x % 8;

        // Finally, get the tile position value from the Background RAM
        let tile = bg_map[(tile_line_index + tile_line_offset) as usize];

        // Calculate the real index of the tile
        let tile_index = tile_index(tile, self.signed_tile_mode);

        for i in 0..160 {
            // FIXME: Get lifetimes or whatever working for bg1 and bg2 above. I think the logic
            // here is correct though.

            // // Get the two bits used to represent the color number
            // let color_lo_byte = self.mem.chram[tile_index * 16 + tile_y as usize * 2];
            // let color_hi_byte = self.mem.chram[tile_index * 16 + tile_y as usize * 2 + 1];

            // let color_lo_bit = color_lo_byte >> tile_x & 0x1;
            // let color_hi_bit = color_hi_byte >> tile_x & 0x1;

            // let color_num = (color_hi_bit << 1) | color_lo_bit;

            // // Map the color number to the shade to display on the screen
            // let shade = self.bg_palette[color_num as usize];
        }
    }

    /// Reads a byte of graphics memory.
    ///
    /// # Panics
    ///
    /// Panics if reading memory that is not managed by the PPU.
    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x8000...0x97FF => {
                let index = address - 0x8000;
                self.mem.chram[index as usize]
            }

            0x9800...0x9FFF => {
                let index = address - 0x9800;
                self.mem.bg_map[index as usize]
            }

            0xFE00...0xFE9F => {
                let index = address - 0xFE00;
                self.mem.oam[index as usize]
            }

            _ => panic!("read out-of-range address in PPU"),
        }
    }

    /// Writes a byte of graphics memory.
    ///
    /// # Panics
    ///
    /// Panics if writing memory that is not managed by the PPU.
    pub fn write_byte(&mut self, address: u16, byte: u8) {
        match address {
            0x8000...0x97FF => {
                let index = address - 0x8000;
                self.mem.chram[index as usize] = byte;
            }

            0x9800...0x9FFF => {
                let index = address - 0x9800;
                self.mem.bg_map[index as usize] = byte;
            }

            0xFE00...0xFE9F => {
                let index = address & 0xFF;
                self.mem.oam[index as usize] = byte;
            }

            _ => panic!("write out-of-range address in PPU"),
        }
    }
}

impl fmt::Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let chram: &[u8] = &self.chram;
        let bg_map: &[u8] = &self.bg_map;
        let oam: &[u8] = &self.oam;

        f.debug_struct("Memory")
            .field("chram", &chram)
            .field("bg_map", &bg_map)
            .field("oam", &oam)
            .finish()
    }
}

/// Finds the index of a tile in the Character RAM.
pub fn tile_index(tile: u8, signed_tile_mode: bool) -> usize {
    if signed_tile_mode {
        ((tile as i8) as i16 + 256) as usize
    } else {
        tile as usize
    }
}

#[cfg(test)]
mod tests {
    use super::Ppu;

    #[test]
    fn chram() {
        let mut ppu = Ppu::new();

        ppu.mem.chram[0] = 1;
        assert_eq!(ppu.read_byte(0x8000), 1);

        ppu.mem.chram[0x17FF] = 2;
        assert_eq!(ppu.read_byte(0x97FF), 2);
    }

    #[test]
    fn oam() {
        let mut ppu = Ppu::new();

        ppu.mem.oam[0] = 1;
        assert_eq!(ppu.read_byte(0xFE00), 1);

        ppu.mem.oam[0x9F] = 2;
        assert_eq!(ppu.read_byte(0xFE9F), 2);
    }

    #[test]
    fn tile_index_test() {
        let mut j = -128;

        let mut i_u8;
        let mut j_u8;

        for i in 0..256 {
            // Annoying hack because rust doesn't support inclusive ranges yet
            i_u8 = i as u8;
            j_u8 = j as u8;

            assert_eq!(super::tile_index(i_u8, false), i_u8 as usize);
            assert_eq!(super::tile_index(j_u8, true), i_u8 as usize + 128);

            j += 1;
        }
    }
}
