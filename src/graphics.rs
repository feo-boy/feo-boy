//! Graphics-related functionality.
//!
//! Contains an implementation of a PPU.

use std::fmt;

use memory::Addressable;

/// Memory managed by the PPU.
struct Memory {
    /// Video RAM.
    vram: [u8; 0x2000],

    /// Object attribute memory (OAM).
    oam: [u8; 0xA0],
}

impl Default for Memory {
    fn default() -> Memory {
        Memory {
            vram: [0; 0x2000],
            oam: [0; 0xA0],
        }
    }
}

/// The picture processing unit.
#[derive(Debug)]
pub struct Ppu {
    mem: Memory,
}


impl Ppu {
    /// Creates a new picture processing unit.
    ///
    /// The initial contents of the memory are unspecified.
    pub fn new() -> Ppu {
        Ppu { mem: Default::default() }
    }
}

impl Addressable for Ppu {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x8000...0x9FFF => {
                let index = address & 0x1FFF;
                self.mem.vram[index as usize]
            }

            0xFE00...0xFE9F => {
                let index = address & 0xFF;
                self.mem.oam[index as usize]
            }

            _ => panic!("read out-of-range address in PPU"),
        }
    }

    fn write_byte(&mut self, address: u16, byte: u8) {
        match address {
            0x8000...0x9FFF => {
                let index = address & 0x1FFF;
                self.mem.vram[index as usize] = byte;
            }

            0xFE00...0xFE9F => {
                let index = address & 0xFF;
                self.mem.oam[index as usize] = byte;
            }

            _ => panic!("write out-of-range address in PPU"),
        }
    }
}

impl fmt::Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let vram: &[u8] = &self.vram;
        let oam: &[u8] = &self.oam;

        f.debug_struct("Memory")
            .field("vram", &vram)
            .field("oam", &oam)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::Ppu;

    use memory::Addressable;

    #[test]
    fn vram() {
        let mut ppu = Ppu::new();

        ppu.mem.vram[0] = 1;
        assert_eq!(ppu.read_byte(0x8000), 1);

        ppu.mem.vram[0x1FFF] = 2;
        assert_eq!(ppu.read_byte(0x9FFF), 2);
    }

    #[test]
    fn oam() {
        let mut ppu = Ppu::new();

        ppu.mem.oam[0] = 1;
        assert_eq!(ppu.read_byte(0xFE00), 1);

        ppu.mem.oam[0x9F] = 2;
        assert_eq!(ppu.read_byte(0xFE9F), 2);
    }
}
