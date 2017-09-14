//! Inter-component communication.

use std::fmt::{self, Display};
use std::ops::Range;

use itertools::Itertools;

use bytes::ByteExt;
use cpu::Interrupts;
use graphics::{Ppu, Shade, TileMapStart, TileDataStart, SpriteSize};
use memory::{Addressable, Mmu};

/// The "wires" of the emulator.
///
/// The `Bus` contains each individual component. All memory accesses are proxied through the
/// `Bus`, which then dispatches the read or write to the correct component.
#[derive(Debug, Default)]
pub struct Bus {
    pub ppu: Ppu,
    pub mmu: Mmu,
    pub interrupts: Interrupts,
}

impl Addressable for Bus {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x8000...0x9FFF | 0xFE00...0xFE9F => self.ppu.read_byte(address),
            0xFF00...0xFF7F | 0xFFFF => self.read_io_register(address),
            _ => self.mmu.read_byte(address),
        }
    }

    fn write_byte(&mut self, address: u16, byte: u8) {
        match address {
            0x8000...0x9FFF | 0xFE00...0xFE9F => self.ppu.write_byte(address, byte),
            0xFF00...0xFF7F | 0xFFFF => self.write_io_register(address, byte),
            _ => self.mmu.write_byte(address, byte),
        }
    }
}

impl Bus {
    /// Create an iterator over the entire memory space.
    pub fn iter(&self) -> MemoryIterator {
        MemoryIterator {
            address_iter: 0x00..0x10000,
            bus: self,
        }
    }

    fn read_io_register(&self, address: u16) -> u8 {
        let Bus {
            ref ppu,
            ref interrupts,
            ..
        } = *self;

        match address {
            // P1/JOYP - Joypad
            0xFF00 => {
                warn!("input is not implemented");

                // Indicates that no buttons are pressed.
                0x0F
            }

            // IF - Interrupt Flag
            0xFF0F => {
                let mut register = 0u8;

                register.set_bit(0, interrupts.vblank.requested);
                register.set_bit(1, interrupts.lcd_status.requested);
                register.set_bit(2, interrupts.timer.requested);
                register.set_bit(3, interrupts.serial.requested);
                register.set_bit(4, interrupts.joypad.requested);

                // The higher bits are unspecified.

                register
            }

            // LCDC - LCD Control
            0xFF40 => {
                let control = &ppu.control;

                let mut register = 0u8;
                register.set_bit(7, control.display_enabled);
                register.set_bit(6, control.window_map_start != TileMapStart::default());
                register.set_bit(5, control.window_enabled);
                register.set_bit(4, control.tile_data_start != TileDataStart::default());
                register.set_bit(3, control.bg_map_start != TileMapStart::default());
                register.set_bit(2, control.sprite_size != SpriteSize::default());
                register.set_bit(1, control.sprites_enabled);
                register.set_bit(0, control.background_enabled);
                register
            }

            // STAT - LCDC Status
            0xFF41 => {
                let mut register = 0u8;

                // Set the lowest two bits to the mode.
                register |= ppu.mode;

                // Set bit 2 if LY == LYC
                register.set_bit(
                    2,
                    self.read_io_register(0xFF44) == self.read_io_register(0xFF45),
                );

                // Other bits are set if the various interrupts are enabled.
                register.set_bit(3, ppu.lcd_status_interrupts.hblank);
                register.set_bit(4, ppu.lcd_status_interrupts.vblank);
                register.set_bit(5, ppu.lcd_status_interrupts.oam);
                register.set_bit(6, ppu.lcd_status_interrupts.ly_lyc_coincidence);

                // The highest bit is unspecified.

                register
            }

            // SCY - Scroll Y
            0xFF42 => ppu.bg_scroll.y,

            // SCX - Scroll X
            0xFF43 => ppu.bg_scroll.x,

            // LCDC Y-Coordinate
            0xFF44 => ppu.line,

            // LYC - LY Compare
            0xFF45 => ppu.line_compare,

            // IE - Interrupt Enable
            0xFFFF => {
                let mut byte = 0x00;
                byte.set_bit(0, interrupts.vblank.enabled);
                byte.set_bit(1, interrupts.lcd_status.enabled);
                byte.set_bit(2, interrupts.timer.enabled);
                byte.set_bit(3, interrupts.serial.enabled);
                byte.set_bit(4, interrupts.joypad.enabled);

                // Remaining bits are unspecified.

                byte
            }

            _ => {
                error!("read unimplemented I/O register {:#04x}", address);
                0x00
            }
        }
    }

    fn write_io_register(&mut self, address: u16, byte: u8) {
        let Bus {
            ref mut ppu,
            ref mut mmu,
            ref mut interrupts,
            ..
        } = *self;

        match address {
            // IF - Interrupt Flag
            0xFF0F => {
                interrupts.vblank.requested = byte.has_bit_set(0);
                interrupts.lcd_status.requested = byte.has_bit_set(1);
                interrupts.timer.requested = byte.has_bit_set(2);
                interrupts.serial.requested = byte.has_bit_set(3);
                interrupts.joypad.requested = byte.has_bit_set(4);
            }

            // NR11 - Channel 1 Sound length/Wave pattern duty
            0xFF11 => {
                warn!("attempted to modify sound channel 1 wave (unimplemented)");
            }

            // NR12 - Channel 1 Volume Envelope
            0xFF12 => {
                warn!("attempted to modify sound channel 1 volume (unimplemented)");
            }

            // NR50 - Channel control / ON-OFF / Volume
            0xFF24 => {
                warn!("attempted to modify master volume (unimplemented)");
            }

            // NR51 - Selection of Sound output terminal
            0xFF25 => {
                warn!("attempted to modify sound output terminal (unimplemented)");
            }

            // Sound on/off
            0xFF26 => {
                // Only the high bit is writable.
                if byte.has_bit_set(7) {
                    info!("enabling sound controller");
                    warn!("sound controller not implemented");
                }
            }

            // LCDC - LCD Control
            0xFF40 => {
                let control = &mut ppu.control;

                control.display_enabled = byte.has_bit_set(7);
                control.window_map_start = if byte.has_bit_set(6) {
                    TileMapStart::Low
                } else {
                    TileMapStart::High
                };
                control.window_enabled = byte.has_bit_set(5);
                control.tile_data_start = if byte.has_bit_set(4) {
                    TileDataStart::Low
                } else {
                    TileDataStart::High
                };
                control.bg_map_start = if byte.has_bit_set(3) {
                    TileMapStart::High
                } else {
                    TileMapStart::Low
                };
                control.sprite_size = if byte.has_bit_set(2) {
                    SpriteSize::Large
                } else {
                    SpriteSize::Small
                };
                control.sprites_enabled = byte.has_bit_set(1);
                control.background_enabled = byte.has_bit_set(0);
            }

            // STAT - LCDC Status
            0xFF41 => {
                ppu.lcd_status_interrupts.hblank = byte.has_bit_set(3);
                ppu.lcd_status_interrupts.vblank = byte.has_bit_set(4);
                ppu.lcd_status_interrupts.oam = byte.has_bit_set(5);
                ppu.lcd_status_interrupts.ly_lyc_coincidence = byte.has_bit_set(6);
            }

            // SCY - Scroll Y
            0xFF42 => ppu.bg_scroll.y = byte,

            // SCX - Scroll X
            0xFF43 => ppu.bg_scroll.x = byte,

            // BGP - BG Palette Data
            0xFF47 => {
                let palette = &mut ppu.bg_palette;

                for i in 0..4 {
                    let shade = (byte >> (i * 2)) & 0x3;
                    palette[i] = shade.into();
                }
            }

            // OBP0 - Object Palette 0 Data
            0xFF48 => Self::set_sprite_palette(&mut ppu.sprite_palette[0], byte),

            // OBP1 - Object Palette 1 Data
            0xFF49 => Self::set_sprite_palette(&mut ppu.sprite_palette[1], byte),

            // WY - Window Y position
            0xFF4A => ppu.window.y = byte,

            // WB - Window X position minus 7
            0xFF4B => ppu.window.x = byte,

            // Unmap BIOS
            0xFF50 => {
                if mmu.bios_mapped {
                    mmu.unmap_bios();
                }
            }

            // IE - Interrupt Enable
            0xFFFF => {
                interrupts.vblank.enabled = byte.has_bit_set(0);
                interrupts.lcd_status.enabled = byte.has_bit_set(1);
                interrupts.timer.enabled = byte.has_bit_set(2);
                interrupts.serial.enabled = byte.has_bit_set(3);
                interrupts.joypad.enabled = byte.has_bit_set(4);
            }

            _ => error!("write to unimplemented I/O register {:#02x}", address),
        }
    }

    fn set_sprite_palette(palette: &mut [Shade], shades: u8) {
        palette[0] = Shade::Transparent;
        for i in 1..4 {
            let shade = (shades >> (i * 2)) & 0x3;
            palette[i] = shade.into();
        }
    }
}

/// An iterator over the memory space.
pub struct MemoryIterator<'a> {
    bus: &'a Bus,
    address_iter: Range<u32>,
}

impl<'a> Iterator for MemoryIterator<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.address_iter.next().map(|addr| {
            self.bus.read_byte(addr as u16)
        })
    }
}

impl Display for Bus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        const LINE_LENGTH: usize = 32;

        let mut address = 0;

        for chunk in &self.iter().chunks(LINE_LENGTH) {
            for (i, byte) in chunk.enumerate() {
                if i == 0 || i == LINE_LENGTH / 2 {
                    write!(f, "{:04x} ", address + i)?;
                }

                write!(f, "{:02x} ", byte)?;
            }

            writeln!(f)?;

            address += LINE_LENGTH;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Bus;

    use quickcheck::{TestResult, quickcheck};

    use graphics::Shade;
    use memory::{Addressable, BIOS_SIZE};

    #[test]
    fn read_write() {
        fn prop(address: u16, value: u8) -> TestResult {
            // Make sure the address is writable. Also, ignore certain registers for now since they
            // aren't implemented fully.
            match address {
                0x0000...0x7FFF | 0xFEA0...0xFEFF | 0xFF00...0xFF39 | 0xFF41...0xFF7F => {
                    return TestResult::discard();
                }
                _ => (),
            }

            let mut bus = Bus::default();
            bus.write_byte(address, value);
            TestResult::from_bool(bus.read_byte(address) == value)
        }

        quickcheck(prop as fn(u16, u8) -> TestResult);
    }

    #[test]
    fn memory_dump() {
        let bus = Bus::default();
        bus.to_string();
    }

    #[test]
    fn bios() {
        let mut bus = Bus::default();
        assert!(bus.mmu.bios_mapped);

        let mut bios = [0; BIOS_SIZE];
        bios[0] = 1;
        bios[0xff] = 2;
        bus.mmu.load_bios(&bios).unwrap();

        assert_eq!(bus.mmu.read_byte(0x0000), 1);
        assert_eq!(bus.mmu.read_byte(0x00FF), 2);

        bus.write_byte(0xFF50, 1);
        assert!(!bus.mmu.bios_mapped);
    }

    #[test]
    fn background_palette_register() {
        let mut bus = Bus::default();
        bus.write_byte(0xFF47, 0b10010011);

        assert_eq!(
            bus.ppu.bg_palette,
            [
                Shade::Black,
                Shade::White,
                Shade::LightGray,
                Shade::DarkGray,
            ]
        );
    }

    #[test]
    fn stat_register() {
        let mut bus = Bus::default();
        bus.ppu.line = 40;
        bus.ppu.line_compare = 40;
        bus.ppu.lcd_status_interrupts.vblank = true;

        assert_eq!(bus.read_byte(0xFF41), 0b00010100);
    }
}
