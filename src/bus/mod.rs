//! Inter-component communication.

mod timer;

use std::fmt::{self, Display};
use std::io::Write;
use std::ops::Range;

use byteorder::{ByteOrder, LittleEndian};
use derivative::Derivative;
use itertools::Itertools;
use log::*;

use crate::audio::SoundController;
use crate::bytes::ByteExt;
use crate::cpu::{Interrupts, MCycles, TCycles};
use crate::graphics::Ppu;
use crate::input::ButtonState;
use crate::memory::{Addressable, Mmu};

use self::timer::Timer;

/// The "wires" of the emulator.
///
/// The `Bus` contains each individual component. All memory accesses are proxied through the
/// `Bus`, which then dispatches the read or write to the correct component.
#[derive(Default, Derivative)]
#[derivative(Debug)]
pub struct Bus {
    pub ppu: Ppu,
    pub audio: SoundController,
    pub mmu: Mmu,
    pub interrupts: Interrupts,
    pub timer: Timer,
    pub button_state: ButtonState,
    pub serial_transfer_data: u8,
    #[derivative(Debug = "ignore")]
    pub serial_out: Option<Box<dyn Write>>,
}

impl Bus {
    /// Returns the word at a given memory address, read in little-endian order. Each component is
    /// ticked two cycles.
    pub fn read_word(&mut self, address: u16) -> u16 {
        LittleEndian::read_u16(&[self.read_byte(address), self.read_byte(address + 1)])
    }

    /// Writes a word to a given memory address in little-endian order. Each component is ticked
    /// two cycles.
    pub fn write_word(&mut self, address: u16, word: u16) {
        let mut bytes = [0u8; 2];

        LittleEndian::write_u16(&mut bytes, word);

        self.write_byte(address, bytes[0]);
        self.write_byte(address + 1, bytes[1]);
    }

    /// Reads a single byte from memory. Ticks each component a cycle.
    pub fn read_byte(&mut self, address: u16) -> u8 {
        let byte = self.read_byte_no_tick(address);
        self.tick(MCycles(1));
        byte
    }

    /// Writes a single byte to memory. Ticks each component a cycle.
    pub fn write_byte(&mut self, address: u16, byte: u8) {
        self.write_byte_no_tick(address, byte);
        self.tick(MCycles(1));
    }

    /// Reads a single byte from memory. This read happens instantaneously: no components are
    /// ticked.
    pub fn read_byte_no_tick(&self, address: u16) -> u8 {
        match address {
            0x8000..=0x9FFF | 0xFE00..=0xFE9F => self.ppu.read_byte(address),
            0xFF00..=0xFF7F | 0xFFFF => self.read_io_register(address),
            _ => self.mmu.read_byte(address),
        }
    }

    /// Writes a single byte to memory. This write happens instantaneously: no components are
    /// ticked.
    pub fn write_byte_no_tick(&mut self, address: u16, byte: u8) {
        match address {
            0x8000..=0x9FFF | 0xFE00..=0xFE9F => self.ppu.write_byte(address, byte),
            0xFF00..=0xFF7F | 0xFFFF => self.write_io_register(address, byte),
            _ => self.mmu.write_byte(address, byte),
        }
    }

    /// Tick each component individually.
    pub fn tick(&mut self, cycles: MCycles) {
        let t_cycles = TCycles::from(cycles);

        for _ in 0..t_cycles.0 {
            self.ppu.step(&mut self.interrupts);
        }

        self.timer
            .tick(cycles, &mut self.interrupts.timer.requested);
    }

    /// Create an iterator over the entire memory space.
    pub fn iter(&self) -> MemoryIterator<'_> {
        MemoryIterator {
            address_iter: 0x00..0x10000,
            bus: self,
        }
    }

    fn read_io_register(&self, address: u16) -> u8 {
        let Bus {
            ref ppu,
            ref audio,
            ref interrupts,
            ref button_state,
            ref timer,
            ..
        } = *self;

        match address {
            // P1/JOYP - Joypad
            0xFF00 => button_state.as_byte(),

            // DIV - Divider Register
            0xFF04 => timer.divider(),

            // TIMA - Timer Counter
            0xFF05 => timer.reg.counter,

            // TMA - Timer Modulo
            0xFF06 => timer.reg.modulo,

            // TAC - Timer Control
            0xFF07 => timer.reg.control,

            // IF - Interrupt Flag
            0xFF0F => {
                let mut register = 0xFFu8;

                register.set_bit(0, interrupts.vblank.requested);
                register.set_bit(1, interrupts.lcd_status.requested);
                register.set_bit(2, interrupts.timer.requested);
                register.set_bit(3, interrupts.serial.requested);
                register.set_bit(4, interrupts.joypad.requested);

                // The higher bits are unspecified.

                register
            }

            // Sound memory
            0xFF10..=0xFF3F => audio.read_byte(address),

            // LCD registers
            0xFF40..=0xFF4B => ppu.read_byte(address),

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
                let mut byte = 0xFF;

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
            // P!/JOYP - Joypad
            0xFF00 => self.button_state.select(byte),

            // SB - Serial transfer data
            0xFF01 => self.serial_transfer_data = byte,

            // SC - Serial Transfer Control
            0xFF02 => {
                warn!("serial transfer is unfinished");

                if let Some(out) = &mut self.serial_out {
                    out.write_all(&[self.serial_transfer_data])
                        .expect("failed to write to serial port");
                }
            }

            // DIV - Divider Register
            0xFF04 => self.timer.reset_divider(),

            // TIMA - Timer Counter
            0xFF05 => self.timer.reg.counter = byte,

            // TMA - Timer Modulo
            0xFF06 => self.timer.reg.modulo = byte,

            // TAC - Timer Control
            0xFF07 => self.timer.reg.control = byte & 0x7,

            // IF - Interrupt Flag
            0xFF0F => {
                let interrupts = &mut self.interrupts;

                interrupts.vblank.requested = byte.has_bit_set(0);
                interrupts.lcd_status.requested = byte.has_bit_set(1);
                interrupts.timer.requested = byte.has_bit_set(2);
                interrupts.serial.requested = byte.has_bit_set(3);
                interrupts.joypad.requested = byte.has_bit_set(4);
            }

            // Sound control registers
            0xFF10..=0xFF3F => self.audio.write_byte(address, byte),

            // LCD registers
            0xFF40..=0xFF4B => {
                // DMA Transfer
                if address == 0xFF46 {
                    // The actual address is 0x100 * the written value, that is, transfer_address
                    // fills the XX in 0xXXNN, where 00 <= NN < A0
                    let transfer_address = u16::from(byte) << 8;

                    // FIXME: The timing is more subtle than this.
                    for i in 0..0xA0 {
                        let transfer_byte = self.read_byte_no_tick(transfer_address + (i as u16));
                        self.write_byte_no_tick(0xFE00 + (i as u16), transfer_byte);
                    }
                } else {
                    self.ppu.write_byte(address, byte);
                }
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

            _ => unimplemented!("write to unimplemented I/O register {:#02x}", address),
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
        self.address_iter
            .next()
            .map(|addr| self.bus.read_byte_no_tick(addr as u16))
    }
}

impl Display for Bus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

    use std::{u16, u8};

    use quickcheck::{QuickCheck, Gen, TestResult};

    use crate::graphics::{BackgroundPalette, Shade, SpriteSize};
    use crate::input::Button;
    use crate::memory::BIOS_SIZE;

    #[test]
    fn read_write() {
        fn read_write(address: u16, value: u8) -> TestResult {
            match address {
                0x0000..=0x7FFF | 0xA000..=0xBFFF | 0xFEA0..=0xFEFF | 0xFF00..=0xFFFF => {
                    TestResult::discard()
                }
                address => {
                    let mut bus = Bus::default();
                    bus.write_byte(address, value);
                    TestResult::from_bool(bus.read_byte(address) == value)
                }
            }
        }

        QuickCheck::new()
            .gen(Gen::new(u16::MAX as usize))
            .quickcheck(read_write as fn(u16, u8) -> TestResult);
    }

    #[ignore]
    #[test]
    fn read_write_io_registers() {
        fn read_write(offset: u8, value: u8) -> TestResult {
            let address = 0xFF00u16 + &offset.into();

            match address {
                0xFF00..=0xFF39 | 0xFF41..=0xFF4A | 0xFF4C..=0xFF7F => TestResult::discard(),
                address => {
                    let mut bus = Bus::default();
                    bus.write_byte(address, value);
                    TestResult::from_bool(bus.read_byte(address) == value)
                }
            }
        }

        QuickCheck::new()
            .gen(Gen::new(u8::MAX as usize))
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
            BackgroundPalette::new([
                Shade::Black,
                Shade::White,
                Shade::LightGray,
                Shade::DarkGray,
            ])
        );
    }

    #[test]
    fn stat_register() {
        let mut bus = Bus::default();
        bus.ppu.control.display_enabled = true;
        bus.ppu.line = 40;
        bus.ppu.line_compare = 40;
        bus.ppu.lcd_status_interrupts.vblank = true;

        assert_eq!(bus.read_byte(0xFF41), 0b00010110);
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

    #[test]
    fn lcd_control() {
        let mut bus = Bus::default();

        bus.write_byte(0xFF40, 0b0000_0000);
        assert!(!bus.ppu.control.display_enabled);
        bus.write_byte(0xFF40, 0b1000_0000);
        assert!(bus.ppu.control.display_enabled);

        bus.write_byte(0xFF40, 0b0000_0000);
        assert_eq!(u16::from(bus.ppu.control.window_map_start), 0x9800);
        bus.write_byte(0xFF40, 0b0100_0000);
        assert_eq!(u16::from(bus.ppu.control.window_map_start), 0x9C00);

        bus.write_byte(0xFF40, 0b0000_0000);
        assert!(!bus.ppu.control.window_enabled);
        bus.write_byte(0xFF40, 0b0010_0000);
        assert!(bus.ppu.control.window_enabled);

        bus.write_byte(0xFF40, 0b0000_0000);
        assert_eq!(u16::from(bus.ppu.control.tile_data_start), 0x8800);
        bus.write_byte(0xFF40, 0b0001_0000);
        assert_eq!(u16::from(bus.ppu.control.tile_data_start), 0x8000);

        bus.write_byte(0xFF40, 0b0000_0000);
        assert_eq!(u16::from(bus.ppu.control.bg_map_start), 0x9800);
        bus.write_byte(0xFF40, 0b0000_1000);
        assert_eq!(u16::from(bus.ppu.control.bg_map_start), 0x9C00);

        bus.write_byte(0xFF40, 0b0000_0000);
        assert_eq!(bus.ppu.control.sprite_size, SpriteSize::Small);
        bus.write_byte(0xFF40, 0b0000_0100);
        assert_eq!(bus.ppu.control.sprite_size, SpriteSize::Large);

        bus.write_byte(0xFF40, 0b0000_0000);
        assert!(!bus.ppu.control.sprites_enabled);
        bus.write_byte(0xFF40, 0b0000_0010);
        assert!(bus.ppu.control.sprites_enabled);

        bus.write_byte(0xFF40, 0b0000_0000);
        assert!(!bus.ppu.control.background_enabled);
        bus.write_byte(0xFF40, 0b0000_0001);
        assert!(bus.ppu.control.background_enabled);
    }
}
