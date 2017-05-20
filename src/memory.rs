//! Functionality related to memory.
//!
//! Contains an implementation of a memory manager unit.

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
                unimplemented!();
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
            0xFE00...0xFE9F => unimplemented!(),
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

        lo + hi << 8
    }

    pub fn write_byte(&mut self, address: u16, byte: u8) {
        // Register to unmap BIOS
        if address == 0xFF50 && byte != 0 {
            self.unmap_bios();
        }

        unimplemented!();
    }

    fn unmap_bios(&mut self) {
        self.in_bios = false;
    }
}

#[cfg(test)]
mod tests {
    use super::Mmu;

    #[test]
    fn bios() {
        let mut mmu = Mmu::new();
        assert!(mmu.in_bios);

        mmu.bios[0] = 1;
        assert_eq!(mmu.read_byte(0x0000), 1);

        mmu.bios[0xFF] = 2;
        assert_eq!(mmu.read_byte(0x00FF), 2);
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
}
