//! Functionality related to memory.
//!
//! Contains the implementation of the memory manager unit (MMU).

use std::cell::{Ref, RefMut, RefCell};
use std::default::Default;
use std::fmt::{self, Debug, Display, Formatter};
use std::num::Wrapping;
use std::ops::Range;
use std::rc::Rc;

use byteorder::{BigEndian, LittleEndian, ByteOrder};
use itertools::Itertools;

use bytes::ByteExt;
use errors::*;
use graphics::Ppu;

const BIOS_SIZE: usize = 0x0100;

/// Operations for memory-like structs.
pub trait Addressable {
    /// Returns the byte at a given memory address.
    fn read_byte(&self, address: u16) -> u8;

    /// Writes a byte to a given memory address.
    fn write_byte(&mut self, address: u16, value: u8);

    /// Returns the word at a given memory address, read in little-endian order.
    fn read_word(&self, address: u16) -> u16 {
        LittleEndian::read_u16(&[self.read_byte(address), self.read_byte(address + 1)])
    }

    /// Writes a word to a given memory address in little-endian order.
    fn write_word(&mut self, address: u16, word: u16) {
        let mut bytes = [0u8; 2];

        LittleEndian::write_u16(&mut bytes, word);

        self.write_byte(address, bytes[0]);
        self.write_byte(address + 1, bytes[1]);
    }
}

/// Memory managed by the MMU.
///
/// VRAM and OAM are stored in the PPU.
struct Memory {
    /// BIOS memory.
    bios: Option<[u8; BIOS_SIZE]>,

    /// ROM banks 0 and 1.
    ///
    /// Bank 1 memory may be switched to other banks by the cartridge.
    rom: [u8; 0x8000],

    /// Cartridge external RAM.
    eram: [u8; 0x2000],

    /// Working RAM.
    wram: [u8; 0x2000],

    /// Zero-Page RAM.
    ///
    /// High speed.
    zram: [u8; 0x0080],
}

impl Default for Memory {
    fn default() -> Memory {
        Memory {
            bios: None,
            rom: [0; 0x8000],
            eram: [0; 0x2000],
            wram: [0; 0x2000],
            zram: [0; 0x0080],
        }
    }
}

impl Debug for Memory {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let bios: Option<&[u8]> = self.bios.as_ref().map(|b| &b[..]);
        let rom: &[u8] = &self.rom;
        let eram: &[u8] = &self.eram;
        let wram: &[u8] = &self.wram;
        let zram: &[u8] = &self.zram;

        f.debug_struct("Memory")
            .field("bios", &bios)
            .field("rom", &rom)
            .field("eram", &eram)
            .field("wram", &wram)
            .field("zram", &zram)
            .finish()
    }
}

/// The memory manager unit.
#[derive(Debug)]
pub struct Mmu {
    /// ROM and RAM.
    mem: Memory,

    /// Whether the BIOS is mapped or not.
    bios_mapped: bool,

    /// The entire ROM contained on the inserted cartridge.
    cartridge_rom: Vec<u8>,

    /// A reference to the picture processing unit.
    ///
    /// Graphics memory reads and writes are forwarded to this unit, as well as any relevant
    /// modifications of the I/O registers.
    ppu: Option<Rc<RefCell<Ppu>>>,
}

impl Mmu {
    /// Creates a new memory manager unit.
    ///
    /// The initial contents of the memory are unspecified.
    pub fn new(ppu: Rc<RefCell<Ppu>>) -> Self {
        let mut mmu = Mmu::default();
        mmu.ppu = Some(ppu);
        mmu
    }

    /// Loads a byte slice containing the BIOS into memory.
    ///
    /// Returns an error if the slice is not the correct length.
    pub fn load_bios(&mut self, bios: &[u8]) -> Result<()> {
        if bios.len() != BIOS_SIZE {
            let msg = format!("must be exactly {} bytes", BIOS_SIZE);
            bail!(ErrorKind::InvalidBios(msg));
        }

        let mut bios_memory = [0; BIOS_SIZE];
        bios_memory.copy_from_slice(bios);

        self.mem.bios = Some(bios_memory);

        Ok(())
    }

    /// Loads a byte slice containing the cartridge ROM into memory.
    ///
    /// This function also parses and logs information contained in the [cartridge header].
    ///
    /// Returns an error if the header checksum is invalid.
    ///
    /// [cartridge header]: http://gbdev.gg8.se/wiki/articles/The_Cartridge_Header
    pub fn load_rom(&mut self, rom: &[u8]) -> Result<()> {
        self.cartridge_rom = rom.to_vec();

        let initial_banks = &self.cartridge_rom[..self.mem.rom.len()];
        self.mem.rom.copy_from_slice(initial_banks);

        let title = &rom[0x134..0x144]
            .iter()
            .map(|&c| c as char)
            .collect::<String>();
        info!("title: {}", title);

        let cartridge_type = match rom[0x147] {
            0x00 => "ROM ONLY",
            0x01 => "MBC1",
            0x02 => "MBC1+RAM",
            0x03 => "MBC1+RAM+BATTERY",
            0x05 => "MBC2",
            0x06 => "MBC2+BATTERY",
            0x08 => "ROM+RAM",
            0x09 => "ROM+RAM+BATTERY",
            0x0B => "MMM01",
            0x0C => "MMM01+RAM",
            0x0D => "MMM01+RAM+BATTERY",
            0x0F => "MBC3+TIMER+BATTERY",
            0x10 => "MBC3+TIMER+RAM+BATTERY",
            0x11 => "MBC3",
            0x12 => "MBC3+RAM",
            0x13 => "MBC3+RAM+BATTERY",
            0x19 => "MBC5",
            0x1A => "MBC5+RAM",
            0x1B => "MBC4+RAM+BATTERY",
            0x1C => "MBC5+RUMBLE",
            0x1D => "MBC5+RUMBLE+RAM",
            0x1E => "MBC5+RUMBLE+RAM+BATTERY",
            0x20 => "MBC6",
            0x22 => "MBC7+SENSOR+RUMBLE+RAM+BATTERY",
            0xFC => "POCKET CAMERA",
            0xFD => "BANDAI TAMAS",
            0xFE => "HuC3",
            0xFF => "HuC1+RAM+BATTERY",
            _ => "unknown",
        };
        info!("cartridge type: {}", cartridge_type);

        let num_banks = match rom[0x148] {
            0x00 => Some(0),
            0x01...0x08 => Some(2 << rom[0x148]),
            0x52 => Some(72),
            0x53 => Some(80),
            0x54 => Some(96),
            _ => None,
        };
        let bank_info = num_banks
            .map(|n| match n {
                0 => String::from("no banking"),
                n => format!("{} banks", n),
            })
            .unwrap_or_else(|| String::from("no bank information"));
        info!("ROM size: {}KB ({})", 32 << rom[0x148], bank_info);

        let eram_size = match rom[0x149] {
            0x00 => Some(0),
            0x01 => Some(2),
            0x02 => Some(8),
            0x03 => Some(32),
            0x04 => Some(128),
            0x05 => Some(64),
            _ => None,
        };
        let eram_info = eram_size
            .map(|n| match n {
                0 => String::from("none"),
                n => format!("{}KB", n),
            })
            .unwrap_or_else(|| String::from("no information"));
        info!("external RAM size: {}", eram_info);

        let region = match rom[0x14A] {
            0 => "Japanese",
            _ => "Non-Japanese",
        };
        info!("region: {}", region);

        let header_sum = {
            let mut x = Wrapping(0u8);
            for byte in rom[0x134..0x14D].iter() {
                x = x - Wrapping(*byte) - Wrapping(1u8);
            }

            x.0
        };
        let header_checksum = rom[0x14D];
        if header_sum != header_checksum {
            let msg = format!(
                "header checksum {:#02} is not equal to sum {:#02}",
                header_checksum,
                header_sum
            );
            bail!(ErrorKind::InvalidCartridge(msg))
        }
        info!("header checksum OK");

        let global_sum: Wrapping<u16> = rom.iter()
            .enumerate()
            .flat_map(|(i, byte)| match i {
                0x14E | 0x14F => None,
                _ => Some(Wrapping(*byte as u16)),
            })
            .sum();
        let global_checksum = BigEndian::read_u16(&rom[0x14E..0x150]);
        if global_sum.0 == global_checksum {
            info!("global checksum OK");
        } else {
            info!(
                "global checksum FAILED: {:#04x} (sum) != {:#04x} (checksum)",
                global_sum,
                global_checksum
            );
        }

        Ok(())
    }

    /// Returns `true` if the MMU has loaded the BIOS using `Mmu::load_bios`.
    pub fn has_bios(&self) -> bool {
        self.mem.bios.is_some()
    }

    /// Resets the MMU to its initial state, including all I/O registers.
    pub fn reset(&mut self) {
        self.bios_mapped = true;
    }

    /// Create an iterator over the entire memory space.
    pub fn iter(&self) -> MemoryIterator {
        MemoryIterator {
            address_iter: 0x00..0x10000,
            mmu: self,
        }
    }

    /// Returns an immutable reference to the PPU.
    ///
    /// # Panics
    ///
    /// Panics if there is no `Ppu` attached.
    fn ppu(&self) -> Ref<Ppu> {
        self.ppu.as_ref().expect("no PPU attached").borrow()
    }

    /// Returns a mutable reference to the PPU.
    ///
    /// # Panics
    ///
    /// Panics if there is no `Ppu` attached.
    fn ppu_mut(&mut self) -> RefMut<Ppu> {
        self.ppu.as_ref().expect("no PPU attached").borrow_mut()
    }

    fn unmap_bios(&mut self) {
        info!("unmapping BIOS");
        self.bios_mapped = false;
    }
}

impl Addressable for Mmu {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            // BIOS
            0x0000...0x00FF if self.bios_mapped && self.has_bios() => {
                self.mem.bios.unwrap()[address as usize]
            }

            // ROM Banks
            0x0000...0x7FFF => self.mem.rom[address as usize],

            // Graphics RAM
            0x8000...0x9FFF => self.ppu().read_byte(address),

            // Cartridge (External) RAM
            0xA000...0xBFFF => {
                let index = address & 0x1FFF;
                self.mem.eram[index as usize]
            }

            // Working RAM
            0xC000...0xFDFF => {
                // Addresses E000-FDFF are known as "shadow RAM." They contain an exact copy of
                // addresses C000-DFFF, until the last 512 bytes of the map.
                let index = address & 0x1FFF;
                self.mem.wram[index as usize]
            }

            // Graphics Sprite Information
            0xFE00...0xFE9F => self.ppu().read_byte(address),

            // Reserved, unused
            0xFEA0...0xFEFF => 0x00,

            // I/O Registers
            0xFF00...0xFF7F => {
                match address {
                    // STAT - LCDC Status
                    0xFF41 => {
                        let mut register = 0u8;

                        let ppu = self.ppu();

                        // Set the lowest two bits to the mode.
                        register |= ppu.mode;

                        // Set bit 2 if LY == LYC
                        register.set_bit(2, self.read_byte(0xFF44) == self.read_byte(0xFF45));

                        // Other bits are set if the various interrupts are enabled.
                        register.set_bit(3, ppu.interrupts.hblank);
                        register.set_bit(4, ppu.interrupts.vblank);
                        register.set_bit(5, ppu.interrupts.oam);
                        register.set_bit(6, ppu.interrupts.ly_lyc);

                        // The highest bit is unspecified.

                        register
                    }

                    // LCDC Y-Coordinate
                    0xFF44 => self.ppu().line,

                    // LYC - LY Compare
                    0xFF45 => self.ppu().line_compare,

                    _ => {
                        error!("read unimplemented I/O register {:#04x}", address);
                        0x00
                    }
                }
            }

            // Zero-Page RAM
            0xFF80...0xFFFF => {
                let index = address & 0x7F;
                self.mem.zram[index as usize]
            }

            _ => unreachable!("exhaustive match was not exhaustive: {}", address),
        }
    }

    fn write_byte(&mut self, address: u16, byte: u8) {
        match address {
            // BIOS and ROM Banks
            0x0000...0x7FFF => {
                // While BIOS and ROM are read-only, if the cartridge has a memory bank controller,
                // writes to this region will trigger a bank switch.
                unimplemented!()
            }

            // Graphics RAM
            0x8000...0x9FFF => self.ppu_mut().write_byte(address, byte),

            // Cartridge (External) RAM
            0xA000...0xBFFF => {
                let index = address & 0x1FFF;
                self.mem.eram[index as usize] = byte;
            }

            // Working RAM
            0xC000...0xFDFF => {
                let index = address & 0x1FFF;
                self.mem.wram[index as usize] = byte;
            }

            // Graphics Sprite Information
            0xFE00...0xFE9F => self.ppu_mut().write_byte(address, byte),

            // Reserved, unused
            0xFEA0...0xFEFF => (),

            // Memory-Mapped I/O
            0xFF00...0xFF7F => {
                // I/O Registers
                match address {
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
                        let control = &mut self.ppu_mut().control;

                        control.display_enabled = byte.has_bit_set(7);
                        control.window_map_start =
                            if byte.has_bit_set(6) { 0x9C00 } else { 0x9800 };
                        control.window_enabled = byte.has_bit_set(5);
                        control.window_data_start =
                            if byte.has_bit_set(4) { 0x8000 } else { 0x8800 };
                        control.bg_map_start = if byte.has_bit_set(3) { 0x9C00 } else { 0x9800 };
                        control.sprite_size = if byte.has_bit_set(2) { (8, 8) } else { (8, 16) };
                        control.sprites_enabled = byte.has_bit_set(1);
                        control.background_enabled = byte.has_bit_set(0);
                    }

                    // STAT - LCDC Status
                    0xFF41 => {
                        let mut ppu = self.ppu_mut();

                        ppu.interrupts.hblank = byte.has_bit_set(3);
                        ppu.interrupts.vblank = byte.has_bit_set(4);
                        ppu.interrupts.oam = byte.has_bit_set(5);
                        ppu.interrupts.ly_lyc = byte.has_bit_set(6);
                    }

                    // SCY - Scroll Y
                    0xFF42 => self.ppu_mut().bg_scroll.y = byte,

                    // SCX - Scroll X
                    0xFF43 => self.ppu_mut().bg_scroll.x = byte,

                    // BGP - BG Palette Data
                    0xFF47 => {
                        let mut palette = &mut self.ppu_mut().bg_palette;

                        for i in 0..4 {
                            let shade = (byte >> (i * 2)) & 0x3;
                            palette[i] = shade.into();
                        }
                    }

                    // Unmap BIOS
                    0xFF50 => {
                        if self.bios_mapped {
                            self.unmap_bios()
                        }
                    }
                    _ => error!("write to unimplemented I/O register {:#02x}", address),
                }
            }

            // Zero-Page RAM
            0xFF80...0xFFFF => {
                let index = address & 0x7F;
                self.mem.zram[index as usize] = byte;
            }

            _ => unreachable!("exhaustive match was not exhaustive: {}", address),
        }
    }
}

impl Default for Mmu {
    /// Returns an MMU with no components attached.
    fn default() -> Mmu {
        Mmu {
            mem: Default::default(),
            bios_mapped: true,
            ppu: None,
            cartridge_rom: Default::default(),
        }
    }
}

impl Display for Mmu {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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

/// An iterator over the MMU's memory.
///
/// Returns each byte in little-endian order.
pub struct MemoryIterator<'a> {
    mmu: &'a Mmu,
    address_iter: Range<u32>,
}

impl<'a> Iterator for MemoryIterator<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.address_iter.next().map(|addr| {
            self.mmu.read_byte(addr as u16)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Mmu, Addressable};

    use std::cell::RefCell;
    use std::rc::Rc;

    use graphics::Ppu;

    #[test]
    fn dump() {
        let ppu = Ppu::new();
        let mmu = Mmu::new(Rc::new(RefCell::new(ppu)));

        mmu.to_string();
    }

    #[test]
    fn bios() {
        let mut mmu = Mmu::default();
        assert!(mmu.bios_mapped);

        let mut bios = [0; super::BIOS_SIZE];
        bios[0] = 1;
        bios[0xff] = 2;
        mmu.load_bios(&bios).unwrap();

        assert_eq!(mmu.read_byte(0x0000), 1);
        assert_eq!(mmu.read_byte(0x00FF), 2);

        mmu.write_byte(0xFF50, 1);
        assert!(!mmu.bios_mapped);
    }

    #[test]
    fn rom() {
        let mut mmu = Mmu::default();
        mmu.unmap_bios();

        mmu.mem.rom[0] = 1;
        assert_eq!(mmu.read_byte(0x0000), 1);

        mmu.mem.rom[0x100] = 2;
        assert_eq!(mmu.read_byte(0x0100), 2);

        mmu.mem.rom[0x7FFF] = 3;
        assert_eq!(mmu.read_byte(0x7FFF), 3);
    }

    #[test]
    fn eram() {
        let mut mmu = Mmu::default();

        mmu.mem.eram[0] = 1;
        assert_eq!(mmu.read_byte(0xA000), 1);

        mmu.mem.eram[0x1FFF] = 2;
        assert_eq!(mmu.read_byte(0xBFFF), 2);
    }

    #[test]
    fn wram() {
        let mut mmu = Mmu::default();

        mmu.mem.wram[0] = 1;
        assert_eq!(mmu.read_byte(0xC000), 1);
        assert_eq!(mmu.read_byte(0xE000), 1);

        mmu.mem.wram[0x1FFF] = 2;
        assert_eq!(mmu.read_byte(0xDFFF), 2);

        mmu.mem.wram[0x1FFF - 512] = 3;
        assert_eq!(mmu.read_byte(0xFDFF), 3);
    }

    #[test]
    fn zram() {
        let mut mmu = Mmu::default();

        mmu.mem.zram[0] = 1;
        assert_eq!(mmu.read_byte(0xFF80), 1);

        mmu.mem.zram[0x7F] = 2;
        assert_eq!(mmu.read_byte(0xFFFF), 2);
    }

    #[test]
    fn words() {
        let mut mmu = Mmu::default();

        mmu.mem.wram[0] = 0xAB;
        mmu.mem.wram[1] = 0xCD;
        assert_eq!(mmu.read_word(0xC000), 0xCDAB);

        mmu.write_word(0xFF80, 0xABCD);
        assert_eq!(mmu.mem.zram[0], 0xCD);
        assert_eq!(mmu.mem.zram[1], 0xAB);
    }

    #[test]
    fn stat_register() {
        let mut ppu = Ppu::new();
        ppu.line = 40;
        ppu.line_compare = 40;
        ppu.interrupts.vblank = true;

        let mmu = Mmu::new(Rc::new(RefCell::new(ppu)));

        assert_eq!(mmu.read_byte(0xFF41), 0b00010100);
    }

    #[test]
    fn background_palette_register() {
        use graphics::Shade;

        let ppu = Rc::new(RefCell::new(Ppu::new()));
        let mut mmu = Mmu::new(Rc::clone(&ppu));

        mmu.write_byte(0xFF47, 0b10010011);

        assert_eq!(
            mmu.ppu().bg_palette,
            [
                Shade::Black,
                Shade::White,
                Shade::LightGray,
                Shade::DarkGray,
            ]
        );
    }
}
