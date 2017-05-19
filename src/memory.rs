//! Functionality related to memory.
//!
//! Contains an implementation of a memory manager unit.

/// The memory manager unit.
pub struct Mmu {
    /// BIOS memory.
    bios: [u8; 0x00FF],

    /// ROM banks 0 and 1.
    rom: [u8; 0x4000],

    /// True if the BIOS is currently mapped into memory.
    in_bios: bool,
}

impl Mmu {
    pub fn new() -> Self {
        Mmu {
            bios: [0; 0x00FF],
            rom: [0; 0x4000],
            in_bios: true,
        }
    }

    pub fn reset(&mut self) {
        self.in_bios = true;
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000...0x0100 if self.in_bios => self.bios[address as usize],
            0x0000...0x8000 => self.rom[address as usize],
            _ => unimplemented!(),
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
