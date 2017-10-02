//! Inter-component communication.

#![cfg_attr(feature = "cargo-clippy", allow(match_same_arms))]

use std::fmt::{self, Display};
use std::ops::Range;

use itertools::Itertools;

use bytes::ByteExt;
use cpu::Interrupts;
use graphics::{Ppu, Shade, TileMapStart, TileDataStart, SpriteSize};
use input::{Button, ButtonState, SelectFlags};
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
    pub button_state: ButtonState,
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
            ref button_state,
            ..
        } = *self;

        match address {
            // P1/JOYP - Joypad
            0xFF00 => {
                let mut register = 0u8;

                register.set_bit(
                    0,
                    !(button_state.is_pressed(Button::Right) ||
                          button_state.is_pressed(Button::A)),
                );

                register.set_bit(
                    1,
                    !(button_state.is_pressed(Button::Left) ||
                          button_state.is_pressed(Button::B)),
                );

                register.set_bit(
                    2,
                    !(button_state.is_pressed(Button::Up) ||
                          button_state.is_pressed(Button::Select)),
                );

                register.set_bit(
                    3,
                    !(button_state.is_pressed(Button::Down) ||
                          button_state.is_pressed(Button::Start)),
                );

                // Sets bits 4 and 5.
                register |= button_state.select.bits();

                trace!("read {:#06x}", register);

                register
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

            // WX - Window X Position minus 7
            0xFF4B => ppu.window.x.wrapping_add(7),

            // Undocumented
            0xFF4C => 0xFF,

            // KEY1 - Prepare Speed Switch - (CGB Only)
            0xFF4D => 0xFF,

            // Undocumented
            0xFF4E => 0xFF,

            // VBK - VRAM Bank (CGB Only)
            0xFF4F => 0xFF,

            // Unmap BIOS Register
            0xFF50 => 0xFF,

            // HDMA1 - New DMA Source, High (CGB Only)
            0xFF51 => 0xFF,

            // HDMA2 - New DMA Source, Low (CGB Only)
            0xFF52 => 0xFF,

            // HDMA3 - New DMA Destination, High (CGB Only)
            0xFF53 => 0xFF,

            // HDMA4 - New DMA Destination, Low (CGB Only)
            0xFF54 => 0xFF,

            // HDMA5 - New DMA Length/Mode/Start (CGB Only)
            0xFF55 => 0xFF,

            // RP - Infrared Communications Port (CGB Only)
            0xFF56 => 0xFF,

            // Undocumented
            0xFF57 => 0xFF,

            // Undocumented
            0xFF58 => 0xFF,

            // Undocumented
            0xFF59 => 0xFF,

            // Undocumented
            0xFF5A => 0xFF,

            // Undocumented
            0xFF5B => 0xFF,

            // Undocumented
            0xFF5C => 0xFF,

            // Undocumented
            0xFF5D => 0xFF,

            // Undocumented
            0xFF5E => 0xFF,

            // Undocumented
            0xFF5F => 0xFF,

            // Undocumented
            0xFF60 => 0xFF,

            // Undocumented
            0xFF61 => 0xFF,

            // Undocumented
            0xFF62 => 0xFF,

            // Undocumented
            0xFF63 => 0xFF,

            // Undocumented
            0xFF64 => 0xFF,

            // Undocumented
            0xFF65 => 0xFF,

            // Undocumented
            0xFF66 => 0xFF,

            // Undocumented
            0xFF67 => 0xFF,

            // BCPS/BGPI - Background Palette Index (CGB Only)
            0xFF68 => 0xFF,

            // BCPD/BGPD - Background Palette Data (CGB Only)
            0xFF69 => 0xFF,

            // OCPS/OBPI - Sprite Palette Index (CGB Only)
            0xFF6A => 0xFF,

            // OCPD/OBPD - Sprite Palette Data (CGB Only)
            0xFF6B => 0xFF,

            // Undocumented (CGB)
            0xFF6C => 0xFF,

            // Undocumented
            0xFF6D => 0xFF,

            // Undocumented
            0xFF6E => 0xFF,

            // Undocumented
            0xFF6F => 0xFF,

            // SVBK - WRAM Bank (CGB Only)
            0xFF70 => 0xFF,

            // Undocumented
            0xFF71 => 0xFF,

            // Undocumented (CGB)
            0xFF72 => 0xFF,

            // Undocumented (CGB)
            0xFF73 => 0xFF,

            // Undocumented (CGB)
            0xFF74 => 0xFF,

            // Undocumented (CGB)
            0xFF75 => 0xFF,

            // Undocumented (CGB)
            0xFF76 => 0xFF,

            // Undocumented (CGB)
            0xFF77 => 0xFF,

            // Undocumented
            0xFF78 => 0xFF,

            // Undocumented
            0xFF79 => 0xFF,

            // Undocumented
            0xFF7A => 0xFF,

            // Undocumented
            0xFF7B => 0xFF,

            // Undocumented
            0xFF7C => 0xFF,

            // Undocumented
            0xFF7D => 0xFF,

            // Undocumented
            0xFF7E => 0xFF,

            // Undocumented
            0xFF7F => 0xFF,

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
        match address {
            0xFF00 => {
                trace!("writing {:#06x}", byte);
                let button_state = &mut self.button_state;
                button_state.select = SelectFlags::from_bits_truncate(byte);
            }

            // SB - Serial transfer data
            0xFF01 => {
                warn!("serial transfer is unimplemented");
            }

            // SC - Serial Transfer Control
            0xFF02 => {
                warn!("serial transfer is unimplemented");
            }

            // IF - Interrupt Flag
            0xFF0F => {
                let interrupts = &mut self.interrupts;

                interrupts.vblank.requested = byte.has_bit_set(0);
                interrupts.lcd_status.requested = byte.has_bit_set(1);
                interrupts.timer.requested = byte.has_bit_set(2);
                interrupts.serial.requested = byte.has_bit_set(3);
                interrupts.joypad.requested = byte.has_bit_set(4);
            }

            // NR10 - Channel 1 Sweep Register
            0xFF10 => {
                warn!("attempted to modify channel 1 sweep (unimplemented)");
            }

            // NR11 - Channel 1 Sound length/Wave pattern duty
            0xFF11 => {
                warn!("attempted to modify sound channel 1 wave (unimplemented)");
            }

            // NR12 - Channel 1 Volume Envelope
            0xFF12 => {
                warn!("attempted to modify sound channel 1 volume (unimplemented)");
            }

            // NR13 - Channel 1 Frequency lo data
            0xFF13 => {
                warn!("attempted to modify sound channel 1 frequency lo data (unimplemented)");
            }

            // NR14 - Channel 1 Frequency hi data
            0xFF14 => {
                warn!("attempted to modify sound channel 1 frequency hi data (unimplemented)");
            }

            // NR21 - Channel 2 Sound Length/Wave Pattery Duty
            0xFF16 => {
                warn!("attempted to modify sound channel 2 wave (unimplemented)");
            }

            // NR22 - Channel 2 Volume Envelope
            0xFF17 => {
                warn!("attempted to modify sound channel 2 volume (unimplemented)");
            }

            // NR23 - Channel 2 Frequency lo data
            0xFF18 => {
                warn!("attempted to modify sound channel 2 frequency lo data (unimplemented)");
            }

            // NR23 - Channel 2 Frequency hi data
            0xFF19 => {
                warn!("attempted to modify sound channel 2 frequency hi data (unimplemented)");
            }

            // NR30 - Channel 3 Sound on/off
            0xFF1A => {
                warn!("attempted to modify channel 3 on/off state (unimplemented)");
            }

            // NR31 - Channel 3 Sound Length
            0xFF1B => {
                warn!("attempted to modify channel 3 sound length (unimplemented)");
            }

            // NR32 - Channel 3 Select output level
            0xFF1C => {
                warn!("attempted to modify channel 3 output level (unimplemented)");
            }

            // NR33 - Channel 3 Frequency lo data
            0xFF1D => {
                warn!("attempted to modify channel 3 frequency lo data (unimplemented)");
            }

            // NR34 - Channel 3 Frequency hi data
            0xFF1E => {
                warn!("attempted to modify channel 3 frequency hi data (unimplemented)");
            }

            // NR41 - Channel 4 Sound Length
            0xFF20 => {
                warn!("attempted to modify channel 4 sound length (unimplemented)");
            }

            // NR42 - Channel 4 Volume Envelope
            0xFF21 => {
                warn!("attempted to modify channel 4 volume envelope (unimplemented)");
            }

            // NR43 - Channel 4 Polynomial Counter
            0xFF22 => {
                warn!("attempted to modify channel 4 polynomial counter (unimplemented)");
            }

            // NR44 - Channel 4 Counter/consecutive; Initial
            0xFF23 => {
                warn!("attempted to modify channel 4 consecutive/initial state (unimplemented)");
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

            // Wave Pattern RAM
            0xFF30...0xFF3F => {
                warn!("attempted to modify wave pattern RAM (unimplemented)");
            }

            // LCDC - LCD Control
            0xFF40 => {
                let control = &mut self.ppu.control;

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
                let ppu = &mut self.ppu;

                ppu.lcd_status_interrupts.hblank = byte.has_bit_set(3);
                ppu.lcd_status_interrupts.vblank = byte.has_bit_set(4);
                ppu.lcd_status_interrupts.oam = byte.has_bit_set(5);
                ppu.lcd_status_interrupts.ly_lyc_coincidence = byte.has_bit_set(6);
            }

            // SCY - Scroll Y
            0xFF42 => {
                let ppu = &mut self.ppu;
                ppu.bg_scroll.y = byte;
            }

            // SCX - Scroll X
            0xFF43 => {
                let ppu = &mut self.ppu;
                ppu.bg_scroll.x = byte;
            }

            // DMA Transfer
            0xFF46 => {
                // The actual address is 0x100 * the written value, that is, transfer_address
                // fills the XX in 0xXXNN, where 00 <= NN < A0
                let transfer_address = u16::from(byte) << 8;

                for i in 0..0xA0 {
                    let transfer_byte = self.read_byte(transfer_address + (i as u16));
                    self.write_byte(0xFE00 + (i as u16), transfer_byte);
                }
            }

            // BGP - BG Palette Data
            0xFF47 => {
                let palette = &mut self.ppu.bg_palette;

                for i in 0..4 {
                    let shade = (byte >> (i * 2)) & 0x3;
                    palette[i] = shade.into();
                }
            }

            // OBP0 - Object Palette 0 Data
            0xFF48 => {
                let ppu = &mut self.ppu;
                Self::set_sprite_palette(&mut ppu.sprite_palette[0], byte);
            }

            // OBP1 - Object Palette 1 Data
            0xFF49 => {
                let ppu = &mut self.ppu;
                Self::set_sprite_palette(&mut ppu.sprite_palette[1], byte);
            }

            // WY - Window Y position
            0xFF4A => {
                let ppu = &mut self.ppu;
                ppu.window.y = byte;
            }

            // WB - Window X position minus 7
            0xFF4B => {
                let ppu = &mut self.ppu;
                ppu.window.x = byte.wrapping_sub(7);
            }

            // Unmap BIOS
            0xFF50 => {
                let mmu = &mut self.mmu;

                if mmu.bios_mapped {
                    mmu.unmap_bios();
                }
            }

            // Undocumented
            0xFF7F => (),

            // IE - Interrupt Enable
            0xFFFF => {
                let interrupts = &mut self.interrupts;

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

    use std::{u8, u16};

    use quickcheck::{QuickCheck, StdGen, TestResult};
    use rand;

    use graphics::Shade;
    use input::Button;
    use memory::{Addressable, BIOS_SIZE};

    #[test]
    fn read_write() {
        fn read_write(address: u16, value: u8) -> TestResult {
            match address {
                0x0000...0x7FFF | 0xFEA0...0xFEFF | 0xFF00...0xFFFF => TestResult::discard(),
                address => {
                    let mut bus = Bus::default();
                    bus.write_byte(address, value);
                    TestResult::from_bool(bus.read_byte(address) == value)
                }
            }
        }

        QuickCheck::new()
            .gen(StdGen::new(rand::thread_rng(), u16::MAX as usize))
            .quickcheck(read_write as fn(u16, u8) -> TestResult);
    }

    #[ignore]
    #[test]
    fn read_write_io_registers() {
        fn read_write(offset: u8, value: u8) -> TestResult {
            let address = 0xFF00u16 + &offset.into();

            match address {
                0xFF00...0xFF39 | 0xFF41...0xFF4A | 0xFF4C...0xFF7F => TestResult::discard(),
                address => {
                    let mut bus = Bus::default();
                    bus.write_byte(address, value);
                    TestResult::from_bool(bus.read_byte(address) == value)
                }
            }
        }

        QuickCheck::new()
            .gen(StdGen::new(rand::thread_rng(), u8::MAX as usize))
            .quickcheck(read_write as fn(u8, u8) -> TestResult);
    }

    #[ignore]
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

    #[test]
    fn window_position() {
        let mut bus = Bus::default();
        bus.write_byte(0xFF4B, 7);
        assert_eq!(bus.ppu.window.x, 0);
    }

    #[test]
    fn dma_transfer() {
        let mut bus = Bus::default();

        for i in 0..0xA0 {
            bus.write_byte(0x8000 + (i as u16), i as u8);
        }

        bus.write_byte(0xFF46, 0x80);

        for i in 0..0xA0 {
            assert_eq!(bus.read_byte(0xFE00 + (i as u16)), i as u8);
        }
    }

    #[test]
    fn button_press() {
        let mut bus = Bus::default();
        assert_eq!(bus.read_byte(0xFF00) & 0x3F, 0x0F);

        bus.button_state.press(Button::Right);
        assert_eq!(bus.read_byte(0xFF00) & 0x3F, 0x0F);

        bus.write_byte(0xFF00, 0x20);
        assert_eq!(bus.read_byte(0xFF00) & 0x3F, 0x2E);

        bus.button_state.release(Button::Right);
        assert_eq!(bus.read_byte(0xFF00) & 0x3F, 0x2F);
    }

    #[test]
    fn multi_press() {
        let mut bus = Bus::default();

        bus.button_state.press(Button::Left);
        bus.button_state.press(Button::B);
        assert_eq!(bus.read_byte(0xFF00) & 0x3F, 0x0F);

        bus.write_byte(0xFF00, 0x30);
        assert_eq!(bus.read_byte(0xFF00) & 0x3F, 0x3D);

        bus.button_state.press(Button::A);
        assert_eq!(bus.read_byte(0xFF00) & 0x3F, 0x3C);
    }
}
