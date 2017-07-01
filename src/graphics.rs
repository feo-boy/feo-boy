//! Graphics-related functionality.
//!
//! Contains an implementation of a PPU.

use std::fmt;

use memory::Addressable;

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
    /// Video RAM.
    vram: [u8; 0x2000],

    /// Object attribute memory (OAM).
    oam: [u8; 0xA0],
}

impl Default for Memory {
    fn default() -> Memory {
        Memory {
            vram: [0; 0x2000],
            oam: [0; 0xA0],
        }
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

                    debug!("set graphics mode to {}", self.mode);
                }
            }

            _ => panic!("unimplemented PPU mode: {:?}", self.mode),
        }
    }
}

impl Addressable for Ppu {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x8000...0x9FFF => {
                let index = address & 0x1FFF;
                self.mem.vram[index as usize]
            }

            0xFE00...0xFE9F => {
                let index = address & 0xFF;
                self.mem.oam[index as usize]
            }

            _ => panic!("read out-of-range address in PPU"),
        }
    }

    fn write_byte(&mut self, address: u16, byte: u8) {
        match address {
            0x8000...0x9FFF => {
                let index = address & 0x1FFF;
                self.mem.vram[index as usize] = byte;
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
        let vram: &[u8] = &self.vram;
        let oam: &[u8] = &self.oam;

        f.debug_struct("Memory")
            .field("vram", &vram)
            .field("oam", &oam)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::Ppu;

    use memory::Addressable;

    #[test]
    fn vram() {
        let mut ppu = Ppu::new();

        ppu.mem.vram[0] = 1;
        assert_eq!(ppu.read_byte(0x8000), 1);

        ppu.mem.vram[0x1FFF] = 2;
        assert_eq!(ppu.read_byte(0x9FFF), 2);
    }

    #[test]
    fn oam() {
        let mut ppu = Ppu::new();

        ppu.mem.oam[0] = 1;
        assert_eq!(ppu.read_byte(0xFE00), 1);

        ppu.mem.oam[0x9F] = 2;
        assert_eq!(ppu.read_byte(0xFE9F), 2);
    }
}
