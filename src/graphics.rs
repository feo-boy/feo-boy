//! Graphics-related functionality.
//!
//! Contains an implementation of a PPU.

use std::fmt::{self, Debug, Formatter};

use byteorder::{ByteOrder, LittleEndian};
use image::{Rgba, RgbaImage};

use bytes::ByteExt;
use cpu::Interrupts;
use memory::Addressable;

/// The width and height of the Game Boy screen.
pub const SCREEN_DIMENSIONS: (u32, u32) = (SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32);
pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

/// The colors that can be displayed by the DMG.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Shade {
    White,
    LightGray,
    DarkGray,
    Black,

    /// A shade that is only used by sprites.
    // FIXME: Should this be represented by `None`?
    Transparent,
}

impl Shade {
    /// Returns a pixel that represents the color of a `Shade`.
    pub fn to_rgba(&self) -> Rgba<u8> {
        use self::Shade::*;

        // This uses the GameBoy Pocket palette.
        // TODO: Support more palettes.
        match *self {
            White => Rgba([0xFF, 0xFF, 0xFF, 0xFF]),
            LightGray => Rgba([0xA9, 0xA9, 0xA9, 0xFF]),
            DarkGray => Rgba([0x54, 0x54, 0x54, 0xFF]),
            Black => Rgba([0x00, 0x00, 0x00, 0xFF]),

            Transparent => panic!("transparent pixels cannot be displayed"),
        }
    }
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

/// Determines under which conditions the LCDC Status register (0xFF41) will fire.
#[derive(Debug, Default)]
pub struct LcdcStatusInterrupts {
    /// Fires during H-Blank.
    pub hblank: bool,

    /// Fires during V-Blank.
    pub vblank: bool,

    /// Fires when OAM is being transferred.
    pub oam: bool,

    /// Fires when LYC = LY (i.e., 0xFF45 = 0xFF44).
    pub ly_lyc_coincidence: bool,
}

/// The location of the window or background tile map.
#[derive(Debug, PartialEq, Eq)]
pub enum TileMapStart {
    /// The low tile map (0x9800).
    Low,

    /// The high tile map (0x9C00).
    High,
}

impl TileMapStart {
    fn address(&self) -> u16 {
        match *self {
            TileMapStart::Low => 0x9800,
            TileMapStart::High => 0x9C00,
        }
    }
}

impl Default for TileMapStart {
    fn default() -> Self {
        TileMapStart::Low
    }
}

/// The location of the window or background tile data.
#[derive(Debug, PartialEq, Eq)]
pub enum TileDataStart {
    /// The low address (0x8000). Offsets are unsigned.
    Low,

    /// The high address (0x8800). Offsets are signed.
    High,
}

impl TileDataStart {
    fn address(&self) -> u16 {
        match *self {
            TileDataStart::Low => 0x8000,
            TileDataStart::High => 0x8800,
        }
    }
}

impl Default for TileDataStart {
    fn default() -> Self {
        TileDataStart::High
    }
}

/// The available sizes of sprites.
#[derive(Debug, PartialEq, Eq)]
pub enum SpriteSize {
    /// 8x8
    Small,

    /// 8x16
    Large,
}

impl Default for SpriteSize {
    fn default() -> Self {
        SpriteSize::Small
    }
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
    pub window_map_start: TileMapStart,

    /// The address of the start of the background and window tile data.
    pub tile_data_start: TileDataStart,

    /// The address of the start of the background tile map.
    pub bg_map_start: TileMapStart,

    /// The size of the sprites being used.
    pub sprite_size: SpriteSize,
}

/// An X/Y coordinate pair.
#[derive(Debug, Default)]
pub struct Position {
    pub x: u8,
    pub y: u8,
}

pub struct ScreenBuffer(pub [[Shade; SCREEN_WIDTH]; SCREEN_HEIGHT]);

impl Debug for ScreenBuffer {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("FrameBuffer").finish()
    }
}

impl Default for ScreenBuffer {
    fn default() -> ScreenBuffer {
        ScreenBuffer([[Shade::default(); SCREEN_WIDTH]; SCREEN_HEIGHT])
    }
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

    /// Contains conditions under which the LCDC Status register will fire.
    pub lcd_status_interrupts: LcdcStatusInterrupts,

    /// The pixels to be rendered on the screen.
    pub pixels: ScreenBuffer,
}

impl Ppu {
    /// Creates a new picture processing unit.
    ///
    /// The initial contents of the memory are unspecified.
    pub fn new() -> Ppu {
        Ppu::default()
    }

    /// Performs one clock step of the PPU.
    pub fn step(&mut self, cycles: u32, interrupts: &mut Interrupts, buffer: &mut RgbaImage) {
        // TODO: Set LCD status interrupt request here

        self.modeclock += cycles;

        match self.mode {
            // Horizontal blank
            0 => {
                if self.modeclock >= 204 {
                    self.modeclock = 0;
                    self.line += 1;

                    if self.line == 143 {
                        // Enter vertical blank mode
                        self.mode = 1;

                        debug!("set graphics mode to {}", self.mode);

                        // Draw the pixels to the screen.
                        for (x, y, pixel) in buffer.enumerate_pixels_mut() {
                            *pixel = self.pixels.0[y as usize][x as usize].to_rgba();
                        }

                        interrupts.vblank.requested = true;
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
        if !self.control.display_enabled {
            return;
        }

        if self.control.background_enabled {
            self.render_tiles();
        }
    }

    pub fn render_tiles(&mut self) {
        const TILE_HEIGHT: u16 = 8;
        const TILE_MAP_HEIGHT: u16 = 32;

        // Calculate the absolute y-position of the pixel in the background map.
        let y_position: u16 = (self.bg_scroll.y + self.line).into();

        // Find which row of the 32x32 tile map the tile is in.
        let tile_row_offset: u16 = (y_position / TILE_HEIGHT) * TILE_MAP_HEIGHT;

        // Draw the line.
        for x in 0..160 {
            let x_position = x + self.bg_scroll.x;

            // Find x-position of the tile in the row of tiles.
            let tile_offset = x_position / 8;

            // Get the address of the tile in memory.
            let tile_id_address = self.control.bg_map_start.address() + &tile_row_offset.into() +
                &tile_offset.into();

            let tile_id = self.read_byte(tile_id_address);
            let tile_address = self.tile_data_address(tile_id);

            // Find the correct vertical position within the tile. Multiply by two because each
            // row of the tile takes two bytes.
            let tile_line = (y_position % TILE_HEIGHT) * 2;

            let shade = self.shade(
                self.read_word(tile_address + tile_line as u16),
                x_position % 8,
            );

            self.pixels.0[self.line as usize][x as usize] = shade;
        }
    }

    /// Given a tile identifier, returns the starting address of the tile.
    ///
    /// The tile identifier may be interpreted as signed or unsigned depending on the tile map
    /// being used.
    fn tile_data_address(&self, tile_id: u8) -> u16 {
        const SIGNED_TILE_OFFSET: i16 = 128;
        const TILE_DATA_ROW_SIZE: u16 = 16;

        let start = &self.control.tile_data_start;

        // Depending on which tile map we're using, the offset can be signed or unsigned.
        let offset = match *start {
            TileDataStart::Low => tile_id.into(),
            TileDataStart::High => (i16::from(tile_id as i8) + SIGNED_TILE_OFFSET) as u16,
        };

        start.address() + offset * TILE_DATA_ROW_SIZE
    }

    /// Gets the shade for rendering a particular pixel of the screen.
    fn shade(&self, tile_row: u16, tile_x: u8) -> Shade {
        // Every two bytes represents one row of 8 pixels. The bits of each byte correspond to one
        // pixel. The first byte contains the lower order bit of the color number, while the second
        // byte contains the higher order bit.
        let mut bytes = [0; 2];
        LittleEndian::write_u16(&mut bytes, tile_row);

        // Convert x-position into bit position (bit 7 is leftmost bit).
        let color_bit = 7 - tile_x;

        let mut color_num = 0;
        color_num.set_bit(0, bytes[0].has_bit_set(color_bit));
        color_num.set_bit(1, bytes[1].has_bit_set(color_bit));

        // Map the color number to the shade to display on the screen
        self.bg_palette[color_num as usize]
    }
}

impl Addressable for Ppu {
    /// Reads a byte of graphics memory.
    ///
    /// # Panics
    ///
    /// Panics if reading memory that is not managed by the PPU.
    fn read_byte(&self, address: u16) -> u8 {
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

            _ => panic!("read out-of-range address in PPU: {:#0x}", address),
        }
    }

    /// Writes a byte of graphics memory.
    ///
    /// # Panics
    ///
    /// Panics if writing memory that is not managed by the PPU.
    fn write_byte(&mut self, address: u16, byte: u8) {
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

#[cfg(test)]
mod tests {
    use std::u8;

    use byteorder::{ByteOrder, LittleEndian};

    use memory::Addressable;

    use super::{Ppu, Shade, TileDataStart};

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
    fn shade() {
        let mut ppu = Ppu::new();

        let tile_row = LittleEndian::read_u16(&[0x4E, 0x8B]);

        ppu.bg_palette = [
            Shade::White,
            Shade::LightGray,
            Shade::DarkGray,
            Shade::Black,
        ];

        assert_eq!(ppu.shade(tile_row, 0), Shade::DarkGray);
        assert_eq!(ppu.shade(tile_row, 1), Shade::LightGray);
        assert_eq!(ppu.shade(tile_row, 2), Shade::White);
        assert_eq!(ppu.shade(tile_row, 3), Shade::White);
        assert_eq!(ppu.shade(tile_row, 4), Shade::Black);
        assert_eq!(ppu.shade(tile_row, 5), Shade::LightGray);
        assert_eq!(ppu.shade(tile_row, 6), Shade::Black);
        assert_eq!(ppu.shade(tile_row, 7), Shade::DarkGray);
    }

    #[test]
    fn tile_data_address() {
        let mut ppu = Ppu::new();
        ppu.control.tile_data_start = TileDataStart::Low;
        assert_eq!(ppu.tile_data_address(0), 0x8000);
        assert_eq!(ppu.tile_data_address(37), 0x8250);
        assert_eq!(ppu.tile_data_address(u8::MAX), 0x8FF0);

        let mut ppu = Ppu::new();
        ppu.control.tile_data_start = TileDataStart::High;
        assert_eq!(ppu.tile_data_address(0), 0x9000);
        assert_eq!(ppu.tile_data_address(37), 0x9250);
        assert_eq!(ppu.tile_data_address(u8::MAX), 0x8FF0);
    }
}
