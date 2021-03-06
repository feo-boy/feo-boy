//! Functionality related to memory.
//!
//! Contains the implementation of the memory manager unit (MMU).

mod mbc;

use std::default::Default;
use std::fmt::{self, Debug, Formatter};
use std::num::Wrapping;
use std::rc::Rc;

use byteorder::{BigEndian, ByteOrder, LittleEndian};
use log::*;
use thiserror::Error;

use self::mbc::{Mbc, Mbc1, Mbc3};

/// The size (in bytes) of the DMG BIOS.
pub const BIOS_SIZE: usize = 0x0100;

#[derive(Debug, Error)]
pub enum BiosError {
    #[error("the BIOS must be exactly 256 bytes")]
    InvalidSize,
}

#[derive(Debug, Error)]
pub enum CartridgeError {
    #[error("the size of the ROM must be at least 32KB")]
    InvalidSize,

    #[error("the header checksum {checksum:#02} is not equal to sum {sum:#02}")]
    BadChecksum { checksum: u8, sum: u8 },

    #[error("cartridge type `{0}` is unimplemented")]
    Unimplemented(String),
}

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

#[cfg(test)]
impl Addressable for [u8; 0x10000] {
    fn read_byte(&self, address: u16) -> u8 {
        self[address as usize]
    }

    fn write_byte(&mut self, address: u16, byte: u8) {
        self[address as usize] = byte;
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
            wram: [0; 0x2000],
            zram: [0; 0x0080],
        }
    }
}

impl Debug for Memory {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let bios: Option<&[u8]> = self.bios.as_ref().map(|b| &b[..]);
        let rom: &[u8] = &self.rom;
        let wram: &[u8] = &self.wram;
        let zram: &[u8] = &self.zram;

        f.debug_struct("Memory")
            .field("bios", &bios)
            .field("rom", &rom)
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
    pub bios_mapped: bool,

    /// The entire ROM contained on the inserted cartridge.
    cartridge_rom: Rc<Vec<u8>>,

    /// Memory bank controller.
    mbc: Option<Box<dyn Mbc>>,
}

impl Mmu {
    /// Creates a new memory manager unit.
    ///
    /// The initial contents of the memory are unspecified.
    pub fn new() -> Self {
        Mmu {
            mem: Memory::default(),
            bios_mapped: true,
            cartridge_rom: Rc::new(vec![]),
            mbc: None,
        }
    }

    /// Loads a byte slice containing the BIOS into memory.
    ///
    /// Returns an error if the slice is not the correct length.
    pub fn load_bios(&mut self, bios: &[u8]) -> Result<(), BiosError> {
        if bios.len() != BIOS_SIZE {
            return Err(BiosError::InvalidSize);
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
    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), CartridgeError> {
        if rom.len() < self.mem.rom.len() {
            return Err(CartridgeError::InvalidSize);
        }

        self.cartridge_rom = Rc::new(rom.to_vec());

        let initial_banks = &self.cartridge_rom[..self.mem.rom.len()];
        self.mem.rom.copy_from_slice(initial_banks);

        let title = &rom[0x134..0x144]
            .iter()
            .take_while(|&&b| b != 0)
            .map(|&c| c as char)
            .collect::<String>();
        info!("title: {}", title);

        let header_sum = {
            let mut x = Wrapping(0u8);
            for byte in rom[0x134..0x14D].iter() {
                x = x - Wrapping(*byte) - Wrapping(1u8);
            }

            x.0
        };
        let header_checksum = rom[0x14D];
        if header_sum != header_checksum {
            return Err(CartridgeError::BadChecksum {
                checksum: header_checksum,
                sum: header_sum,
            });
        }
        info!("header checksum OK");

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

        self.mbc = if cartridge_type.contains("ROM") {
            None
        } else if cartridge_type.contains("MBC1") {
            Some(Box::new(Mbc1::new(Rc::clone(&self.cartridge_rom))))
        } else if cartridge_type.contains("MBC3") {
            Some(Box::new(Mbc3::new(Rc::clone(&self.cartridge_rom))))
        } else {
            return Err(CartridgeError::Unimplemented(cartridge_type.to_owned()));
        };

        let num_banks = match rom[0x148] {
            0x00 => Some(0),
            0x01..=0x08 => Some(2 << rom[0x148]),
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

        let global_sum: Wrapping<u16> = rom
            .iter()
            .enumerate()
            .flat_map(|(i, byte)| match i {
                0x14E | 0x14F => None,
                _ => Some(Wrapping(u16::from(*byte))),
            })
            .sum();
        let global_checksum = BigEndian::read_u16(&rom[0x14E..0x150]);
        if global_sum.0 == global_checksum {
            info!("global checksum OK");
        } else {
            info!(
                "global checksum FAILED: {:#04x} (sum) != {:#04x} (checksum)",
                global_sum, global_checksum
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
        for byte in &mut self.mem.wram {
            *byte = 0;
        }

        if self.mem.bios.is_some() {
            self.bios_mapped = true;
        }
    }

    /// Unmaps the BIOS from the memory map, meaning that bytes `0x00`-`0x100` will read as
    /// cartridge ROM.
    pub fn unmap_bios(&mut self) {
        info!("unmapping BIOS");
        self.bios_mapped = false;
    }

    /// Reads a byte from memory.
    ///
    /// # Panics
    ///
    /// Panics if attempting to read memory managed by a different component, such as the `PPU`.
    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            // BIOS
            0x0000..=0x00FF if self.bios_mapped && self.has_bios() => {
                self.mem.bios.unwrap()[address as usize]
            }

            // ROM Banks
            0x0000..=0x7FFF => match self.mbc {
                Some(ref mbc) => mbc.read_byte(address),
                None => self.mem.rom[address as usize],
            },

            // Graphics RAM
            0x8000..=0x9FFF => panic!("graphics RAM is present on the PPU"),

            // Cartridge (External) RAM
            0xA000..=0xBFFF => match self.mbc {
                Some(ref mbc) => mbc.read_byte(address),
                None => 0xFF,
            },

            // Working RAM
            0xC000..=0xFDFF => {
                // Addresses E000-FDFF are known as "shadow RAM." They contain an exact copy of
                // addresses C000-DFFF, until the last 512 bytes of the map.
                let index = address & 0x1FFF;
                self.mem.wram[index as usize]
            }

            // Graphics Sprite Information
            0xFE00..=0xFE9F => panic!("sprite RAM is present on the PPU"),

            // Reserved, unused
            0xFEA0..=0xFEFF => 0x00,

            // I/O Registers
            0xFF00..=0xFF7F | 0xFFFF => panic!("I/O registers are not stored in memory"),

            // Zero-Page RAM
            0xFF80..=0xFFFE => {
                let index = address & 0x7F;
                self.mem.zram[index as usize]
            }
        }
    }

    /// Writes a byte to memory.
    ///
    /// # Panics
    ///
    /// Panics if attempting to write memory managed by a different component, such as the `PPU`.
    pub fn write_byte(&mut self, address: u16, byte: u8) {
        match address {
            // BIOS and ROM Banks
            0x0000..=0x7FFF => {
                // While BIOS and ROM are read-only, if the cartridge has a memory bank controller,
                // writes to this region will trigger a bank switch.

                match self.mbc {
                    Some(ref mut mbc) => mbc.write_byte(address, byte),
                    None => warn!(
                        "attempted to write {:#04x} to read-only memory at {:#06x}",
                        byte, address
                    ),
                }
            }

            // Graphics RAM
            0x8000..=0x9FFF => panic!("graphics RAM is present on the PPU"),

            // Cartridge (External) RAM
            0xA000..=0xBFFF => {
                if let Some(mbc) = &mut self.mbc {
                    mbc.write_byte(address, byte)
                }
            }

            // Working RAM
            0xC000..=0xFDFF => {
                let index = address & 0x1FFF;
                self.mem.wram[index as usize] = byte;
            }

            // Graphics Sprite Information
            0xFE00..=0xFE9F => panic!("sprite RAM is present on the PPU"),

            // Reserved, unused
            0xFEA0..=0xFEFF => (),

            // I/O Registers
            0xFF00..=0xFF7F | 0xFFFF => panic!("I/O registers are not stored in memory"),

            // Zero-Page RAM
            0xFF80..=0xFFFE => {
                let index = address & 0x7F;
                self.mem.zram[index as usize] = byte;
            }
        }
    }
}

impl Default for Mmu {
    fn default() -> Mmu {
        Mmu::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Mmu;

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

        mmu.mem.zram[0x7E] = 2;
        assert_eq!(mmu.read_byte(0xFFFE), 2);
    }

    #[test]
    fn words() {
        use super::Addressable;

        let mut bus = [0u8; 0x10000];

        bus[0xC000] = 0xAB;
        bus[0xC001] = 0xCD;
        assert_eq!(bus.read_word(0xC000), 0xCDAB);

        bus.write_word(0x1234, 0xABCD);
        assert_eq!(bus[0x1234], 0xCD);
        assert_eq!(bus[0x1235], 0xAB);
    }
}
