//! Functionality related to memory.
//!
//! Contains an implementation of a memory manager unit.

use std::default::Default;
use std::fmt::{self, Debug, Display, Formatter};

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

    /// Zero-page RAM.
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

    pub fn reset(&mut self) {
        self.in_bios = true;
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000...0x00FF if self.in_bios => self.bios[address as usize],
            0x0000...0x7FFF => self.rom[address as usize],
            0x8000...0x9FFF => {
                error!("read unimplemented memory: VRAM");
                0x00
            }
            0xA000...0xBFFF => {
                let index = address & 0x1FFF;

                self.eram[index as usize]
            }
            0xC000...0xFDFF => {
                // Addresses E000-FDFF are known as "shadow RAM." They contain an exact copy of
                // addresses C000-DFFF, until the last 512 bytes of the map.
                let index = address & 0x1FFF;

                self.wram[index as usize]
            }
            0xFE00...0xFE9F => {
                error!("read unimplemented memory: OAM");
                0x00
            }
            0xFEA0...0xFF7F => {
                error!("read unimplemented memory: I/O registers");
                0x00
            }
            0xFF80...0xFFFF => {
                let index = address & 0x7F;

                self.zram[index as usize]
            }
            _ => unreachable!(),
        }
    }

    pub fn read_word(&self, address: u16) -> u16 {
        let lo: u16 = self.read_byte(address).into();
        let hi: u16 = self.read_byte(address + 1).into();

        lo + (hi << 8)
    }

    pub fn write_byte(&mut self, address: u16, byte: u8) {
        match address {
            0x0000...0x7FFF => {
                // BIOS and ROM are read-only.
                return;
            }
            0x8000...0x9FFF => unimplemented!(),
            0xA000...0xBFFF => {
                let index = address & 0x1FFF;
                self.eram[index as usize] = byte;
            }
            0xC000...0xFDFF => {
                let index = address & 0x1FFF;
                self.wram[index as usize] = byte;
            }
            0xFE00...0xFE9F => unimplemented!(),
            0xFF00...0xFF7F => {
                // I/O Registers
                match address {
                    0xFF50 if address != 0 => self.unmap_bios(),
                    _ => unimplemented!(),
                }
            }
            0xFF80...0xFFFF => {
                let index = address & 0x7F;
                self.zram[index as usize] = byte;
            }
            _ => unreachable!(),
        }
    }

    pub fn write_word(&mut self, address: u16, word: u16) {
        self.write_byte(address, (word & 0xFF) as u8);
        self.write_byte(address + 1, (word >> 8) as u8);
    }

    pub fn iter<'a>(&'a self) -> MemoryIterator<'a> {
        MemoryIterator {
            pos: Some(0),
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
        let mut address = 0;

        let mut iter = self.iter().peekable();
        while iter.peek().is_some() {
            write!(f, "{:04x}", address)?;

            for _ in 0..16 {
                address += 1;
                let byte = iter.next().expect("expected memory to be a multiple of 16");
                write!(f, " {:02x}", byte)?;
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

pub struct MemoryIterator<'a> {
    pos: Option<u16>,
    mmu: &'a Mmu,
}

impl<'a> Iterator for MemoryIterator<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        match self.pos {
            Some(0xFFFF) => {
                self.pos = None;
                Some(self.mmu.read_byte(0xFFFF))
            }
            Some(pos) => {
                self.pos = Some(pos + 1);
                Some(self.mmu.read_byte(pos))
            }
            None => None,
        }
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
