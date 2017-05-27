//! Functionality related to memory.
//!
//! Contains an implementation of a memory manager unit.

use std::default::Default;
use std::fmt::{self, Debug, Display, Formatter};
use std::ops::Range;

use byteorder::{LittleEndian, ByteOrder};
use itertools::Itertools;

use errors::*;

const BIOS_SIZE: usize = 0x0100;

/// The memory manager unit.
pub struct Mmu {
    /// BIOS memory.
    bios: [u8; BIOS_SIZE],

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

    /// True if the BIOS is currently mapped into memory.
    in_bios: bool,
}

impl Mmu {
    pub fn new() -> Self {
        Mmu {
            bios: [0; BIOS_SIZE],
            rom: [0; 0x8000],
            eram: [0; 0x2000],
            wram: [0; 0x2000],
            zram: [0; 0x0080],
            in_bios: true,
        }
    }

    pub fn load_bios(&mut self, bios: &[u8]) -> Result<()> {
        if bios.len() != BIOS_SIZE {
            bail!(ErrorKind::InvalidBios(format!("must be exactly {} bytes", BIOS_SIZE)));
        }

        self.bios.copy_from_slice(bios);

        Ok(())
    }

    pub fn load_rom(&mut self, rom: &[u8]) -> Result<()> {
        for (address, byte) in rom.iter().enumerate() {
            self.rom[address as usize] = *byte;
        }

        Ok(())
    }

    pub fn reset(&mut self) {
        self.in_bios = true;
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            // BIOS
            0x0000...0x00FF if self.in_bios => self.bios[address as usize],

            // BIOS and ROM Banks
            0x0000...0x7FFF => self.rom[address as usize],

            // Graphics RAM
            0x8000...0x9FFF => {
                warn!("read unimplemented memory: VRAM");
                0x00
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

            // Graphics Sprite Information
            0xFE00...0xFE9F => {
                warn!("read unimplemented memory: OAM");
                0x00
            }

            // 
            0xFEA0...0xFF7F => {
                warn!("read unimplemented memory: I/O registers");
                0x00
            }

            // Zero-Page RAM
            0xFF80...0xFFFF => {
                let index = address & 0x7F;

                self.zram[index as usize]
            }

            // Bad Memory Address
            _ => unreachable!(),
        }
    }

    pub fn read_word(&self, address: u16) -> u16 {
        LittleEndian::read_u16(&[self.read_byte(address), self.read_byte(address + 1)])
    }

    pub fn write_byte(&mut self, address: u16, byte: u8) {
        match address {
            // BIOS and ROM Banks
            0x0000...0x7FFF => {
                // BIOS and ROM are read-only.
                return;
            }

            // Graphics RAM
            0x8000...0x9FFF => unimplemented!(),

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
            0xFE00...0xFE9F => unimplemented!(),

            // Memory-Mapped I/O
            0xFF00...0xFF7F => {
                // I/O Registers
                match address {
                    0xFF50 if address != 0 => self.unmap_bios(),
                    _ => unimplemented!(),
                }
            }

            // Zeroid-Page RAM
            0xFF80...0xFFFF => {
                let index = address & 0x7F;
                self.zram[index as usize] = byte;
            }

            // Bad Memory Address
            _ => unreachable!(),
        }
    }

    pub fn write_word(&mut self, address: u16, word: u16) {
        let mut bytes = [0u8; 2];

        LittleEndian::write_u16(&mut bytes, word);

        self.write_byte(address, bytes[0]);
        self.write_byte(address + 1, bytes[1]);
    }

    pub fn iter<'a>(&'a self) -> MemoryIterator<'a> {
        MemoryIterator {
            address_iter: 0x00..0x10000,
            mmu: self,
        }
    }

    fn unmap_bios(&mut self) {
        self.in_bios = false;
    }
}

impl Debug for Mmu {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let bios = self.bios[..].fmt(f);
        let rom = self.rom[..].fmt(f);
        let eram = self.eram[..].fmt(f);
        let wram = self.wram[..].fmt(f);
        let zram = self.zram[..].fmt(f);

        f.debug_struct("Mmu")
            .field("in_bios", &self.in_bios)
            .field("bios", &bios)
            .field("rom", &rom)
            .field("eram", &eram)
            .field("wram", &wram)
            .field("zram", &zram)
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
        assert!(mmu.in_bios);

        mmu.bios[0] = 1;
        assert_eq!(mmu.read_byte(0x0000), 1);

        mmu.bios[0xFF] = 2;
        assert_eq!(mmu.read_byte(0x00FF), 2);

        mmu.write_byte(0xFF50, 1);
        assert!(!mmu.in_bios);
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
