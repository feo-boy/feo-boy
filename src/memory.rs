//! Functionality related to memory.
//!
//! Contains the implementation of the memory manager unit (MMU).

use std::default::Default;
use std::fmt::{self, Debug, Display, Formatter};
use std::ops::Range;
use std::num::Wrapping;

use byteorder::{BigEndian, LittleEndian, ByteOrder};
use itertools::Itertools;

use bytes::ByteExt;
use errors::*;

const BIOS_SIZE: usize = 0x0100;

/// The I/O Registers.
#[derive(Debug)]
struct IoRegisters {
    /// True if the BIOS is currently mapped into memory.
    bios_mapped: bool,

    /// True if the sound controller is enabled.
    sound_enabled: bool,
}

impl Default for IoRegisters {
    /// Resets the I/O registers to their initial state.
    ///
    /// - The BIOS is mapped.
    /// - The sound controller is disabled.
    fn default() -> Self {
        IoRegisters {
            bios_mapped: true,
            sound_enabled: false,
        }
    }
}

/// The memory manager unit.
pub struct Mmu {
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

    /// Video RAM.
    vram: [u8; 0x2000],

    /// Object attribute memory (OAM).
    oam: [u8; 0xA0],

    /// Zero-Page RAM.
    ///
    /// High speed.
    zram: [u8; 0x0080],

    /// The I/O registers.
    io_reg: IoRegisters,

    /// The entire ROM contained on the inserted cartridge.
    cartridge_rom: Vec<u8>,
}

impl Mmu {
    /// Creates a new memory manager unit.
    ///
    /// The initial contents of the memory are unspecified.
    pub fn new() -> Self {
        Mmu {
            bios: None,
            rom: [0; 0x8000],
            eram: [0; 0x2000],
            wram: [0; 0x2000],
            vram: [0; 0x2000],
            oam: [0; 0xA0],
            zram: [0; 0x0080],
            io_reg: IoRegisters::default(),
            cartridge_rom: Vec::default(),
        }
    }

    /// Loads a byte slice containing the BIOS into memory.
    ///
    /// Returns an error if the slice is not the correct length.
    pub fn load_bios(&mut self, bios: &[u8]) -> Result<()> {
        if bios.len() != BIOS_SIZE {
            bail!(ErrorKind::InvalidBios(format!("must be exactly {} bytes", BIOS_SIZE)));
        }

        let mut bios_memory = [0; BIOS_SIZE];
        bios_memory.copy_from_slice(bios);

        self.bios = Some(bios_memory);

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

        let initial_banks = &self.cartridge_rom[..self.rom.len()];
        self.rom.copy_from_slice(initial_banks);

        info!("title: {}",
              &rom[0x134..0x144]
                   .iter()
                   .map(|&c| c as char)
                   .collect::<String>());

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
            .map(|n| if n == 0 {
                     String::from("no banking")
                 } else {
                     format!("{} banks", n)
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
            .map(|n| if n == 0 {
                     String::from("none")
                 } else {
                     format!("{}KB", n)
                 })
            .unwrap_or_else(|| String::from("no information"));
        info!("external RAM size: {}", eram_info);

        info!("region: {}",
              if rom[0x14A] == 0 {
                  "Japanese"
              } else {
                  "Non-Japanese "
              });

        let header_sum = {
            let mut x = Wrapping(0u8);
            for byte in rom[0x134..0x14D].iter() {
                x = x - Wrapping(*byte) - Wrapping(1u8);
            }

            x.0
        };
        let header_checksum = rom[0x14D];
        if header_sum != header_checksum {
            let msg = format!("header checksum {:#02} is not equal to sum {:#02}",
                              header_checksum,
                              header_sum);
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
            info!("global checksum FAILED: {:#04x} (sum) != {:#04x} (checksum)",
                  global_sum,
                  global_checksum);
        }

        Ok(())
    }

    /// Returns `true` if the MMU has loaded the BIOS using `Mmu::load_bios`.
    pub fn has_bios(&self) -> bool {
        self.bios.is_some()
    }

    /// Resets the MMU to its initial state, including all I/O registers.
    pub fn reset(&mut self) {
        self.io_reg = Default::default();
    }

    /// Returns the byte at a given memory address.
    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            // BIOS
            0x0000...0x00FF if self.io_reg.bios_mapped && self.has_bios() => {
                self.bios.unwrap()[address as usize]
            }

            // ROM Banks
            0x0000...0x7FFF => self.rom[address as usize],

            // Graphics RAM
            0x8000...0x9FFF => {
                let index = address & 0x1FFF;
                self.vram[index as usize]
            }

            // Cartridge (External) RAM
            0xA000...0xBFFF => {
                let index = address & 0x1FFF;

                self.eram[index as usize]
            }

            // Working RAM
            0xC000...0xFDFF => {
                // Addresses E000-FDFF are known as "shadow RAM." They contain an exact copy of
                // addresses C000-DFFF, until the last 512 bytes of the map.
                let index = address & 0x1FFF;

                self.wram[index as usize]
            }

            // OAM
            0xFE00...0xFE9F => {
                let index = address & 0xFF;
                self.oam[index as usize]
            }

            // Reserved, unused
            0xFEA0...0xFEFF => 0x00,

            // I/O Registers
            0xFF00...0xFF7F => {
                error!("read unimplemented memory: I/O registers");
                0x00
            }

            // Zero-Page RAM
            0xFF80...0xFFFF => {
                let index = address & 0x7F;

                self.zram[index as usize]
            }

            _ => unreachable!(),
        }
    }

    /// Returns the word at a given memory address, read in little-endian order.
    pub fn read_word(&self, address: u16) -> u16 {
        LittleEndian::read_u16(&[self.read_byte(address), self.read_byte(address + 1)])
    }

    /// Writes a byte to a given memory address.
    pub fn write_byte(&mut self, address: u16, byte: u8) {
        match address {
            // BIOS and ROM Banks
            0x0000...0x7FFF => {
                // While BIOS and ROM are read-only, if the cartridge has a memory bank controller,
                // writes to this region will trigger a bank switch.
                unimplemented!()
            }

            // Graphics RAM
            0x8000...0x9FFF => {
                let index = address & 0x1FFF;
                self.vram[index as usize] = byte;
            }

            // Cartridge (External) RAM
            0xA000...0xBFFF => {
                let index = address & 0x1FFF;
                self.eram[index as usize] = byte;
            }

            // Working RAM
            0xC000...0xFDFF => {
                let index = address & 0x1FFF;
                self.wram[index as usize] = byte;
            }

            // Graphics Sprite Information
            0xFE00...0xFE9F => {
                let index = address & 0xFF;
                self.oam[index as usize] = byte;
            }

            // Reserved, unused
            0xFEA0...0xFEFF => (),

            // Memory-Mapped I/O
            0xFF00...0xFF7F => {
                // I/O Registers
                match address {
                    // Sound on/off
                    0xFF26 => {
                        // Only the high bit is writable.
                        if byte.has_bit_set(7) {
                            info!("enabling sound controller");
                            self.io_reg.sound_enabled = true;
                        }
                    }
                    // Unmap BIOS
                    0xFF50 => {
                        if self.io_reg.bios_mapped {
                            self.unmap_bios()
                        }
                    }
                    _ => warn!("write to unimplemented I/O register {:#02x}", address),
                }
            }

            // Zero-Page RAM
            0xFF80...0xFFFF => {
                let index = address & 0x7F;
                self.zram[index as usize] = byte;
            }

            _ => unreachable!(),
        }
    }

    /// Writes a word to a given memory address in little-endian order.
    pub fn write_word(&mut self, address: u16, word: u16) {
        let mut bytes = [0u8; 2];

        LittleEndian::write_u16(&mut bytes, word);

        self.write_byte(address, bytes[0]);
        self.write_byte(address + 1, bytes[1]);
    }

    /// Create an iterator over the entire memory space.
    pub fn iter(&self) -> MemoryIterator {
        MemoryIterator {
            address_iter: 0x00..0x10000,
            mmu: self,
        }
    }

    fn unmap_bios(&mut self) {
        info!("unmapping BIOS");
        self.io_reg.bios_mapped = false;
    }
}

impl Debug for Mmu {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let bios: Option<&[u8]> = self.bios.as_ref().map(|b| &b[..]);
        let rom: &[u8] = &self.rom;
        let eram: &[u8] = &self.eram;
        let wram: &[u8] = &self.wram;
        let vram: &[u8] = &self.vram;
        let oam: &[u8] = &self.oam;
        let zram: &[u8] = &self.zram;

        f.debug_struct("Mmu")
            .field("bios", &bios)
            .field("rom", &rom)
            .field("eram", &eram)
            .field("wram", &wram)
            .field("vram", &vram)
            .field("oam", &oam)
            .field("zram", &zram)
            .field("io_reg", &self.io_reg)
            .field("cartridge_rom", &self.cartridge_rom)
            .finish()
    }
}

impl Default for Mmu {
    fn default() -> Self {
        Mmu::new()
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
        self.address_iter
            .next()
            .map(|addr| self.mmu.read_byte(addr as u16))
    }
}

#[cfg(test)]
mod tests {
    use super::Mmu;

    #[test]
    fn dump() {
        Mmu::new().to_string();
    }

    #[test]
    fn bios() {
        let mut mmu = Mmu::new();
        assert!(mmu.io_reg.bios_mapped);

        let mut bios_memory = [0; super::BIOS_SIZE];
        bios_memory[0] = 1;
        bios_memory[0xFF] = 2;
        mmu.bios = Some(bios_memory);

        assert_eq!(mmu.read_byte(0x0000), 1);

        assert_eq!(mmu.read_byte(0x00FF), 2);

        mmu.write_byte(0xFF50, 1);
        assert!(!mmu.io_reg.bios_mapped);
    }

    #[test]
    fn rom() {
        let mut mmu = Mmu::new();
        mmu.unmap_bios();

        mmu.rom[0] = 1;
        assert_eq!(mmu.read_byte(0x0000), 1);

        mmu.rom[0x100] = 2;
        assert_eq!(mmu.read_byte(0x0100), 2);

        mmu.rom[0x7FFF] = 3;
        assert_eq!(mmu.read_byte(0x7FFF), 3);
    }

    #[test]
    fn vram() {
        let mut mmu = Mmu::new();

        mmu.vram[0] = 1;
        assert_eq!(mmu.read_byte(0x8000), 1);

        mmu.vram[0x1FFF] = 2;
        assert_eq!(mmu.read_byte(0x9FFF), 2);
    }

    #[test]
    fn eram() {
        let mut mmu = Mmu::new();

        mmu.eram[0] = 1;
        assert_eq!(mmu.read_byte(0xA000), 1);

        mmu.eram[0x1FFF] = 2;
        assert_eq!(mmu.read_byte(0xBFFF), 2);
    }

    #[test]
    fn wram() {
        let mut mmu = Mmu::new();

        mmu.wram[0] = 1;
        assert_eq!(mmu.read_byte(0xC000), 1);
        assert_eq!(mmu.read_byte(0xE000), 1);

        mmu.wram[0x1FFF] = 2;
        assert_eq!(mmu.read_byte(0xDFFF), 2);

        mmu.wram[0x1FFF - 512] = 3;
        assert_eq!(mmu.read_byte(0xFDFF), 3);
    }

    #[test]
    fn oam() {
        let mut mmu = Mmu::new();

        mmu.oam[0] = 1;
        assert_eq!(mmu.read_byte(0xFE00), 1);

        mmu.oam[0x9F] = 2;
        assert_eq!(mmu.read_byte(0xFE9F), 2);
    }

    #[test]
    fn zram() {
        let mut mmu = Mmu::new();

        mmu.zram[0] = 1;
        assert_eq!(mmu.read_byte(0xFF80), 1);

        mmu.zram[0x7F] = 2;
        assert_eq!(mmu.read_byte(0xFFFF), 2);
    }

    #[test]
    fn words() {
        let mut mmu = Mmu::new();

        mmu.wram[0] = 0xAB;
        mmu.wram[1] = 0xCD;
        assert_eq!(mmu.read_word(0xC000), 0xCDAB);

        mmu.write_word(0xFF80, 0xABCD);
        assert_eq!(mmu.zram[0], 0xCD);
        assert_eq!(mmu.zram[1], 0xAB);
    }
}
