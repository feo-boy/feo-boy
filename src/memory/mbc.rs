
use std::fmt::{self, Debug, Formatter};

//const RAM_SIZE: usize = 32 * 0x400 * 0x400;
const RAM_SIZE: usize = 0x2000 * 4;
const RTC_SIZE: usize = 0x2000 * 5;

#[derive(Debug)]
enum RamRtcSelect {
    Ram(u8), // 0-3
    Rtc(u8), // 8-c -> 0-4
}

pub struct Mbc3 {
    rom: Vec<u8>,
    ram: [u8; RAM_SIZE],
    rtc: [u8; RTC_SIZE],
    ram_timer_enabled: bool,
    rom_select: u8,
    ram_rtc_select: RamRtcSelect,
}

impl Mbc3 {
    pub fn new(rom: Vec<u8>) -> Mbc3 {
        Mbc3 {
            rom,
            ram: [0; RAM_SIZE],
            rtc: [0; RTC_SIZE],
            ram_timer_enabled: false,
            rom_select: 1,
            ram_rtc_select: RamRtcSelect::Ram(0),
        }
    }
}

impl super::Addressable for Mbc3 {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            // ROM Bank 00 (RO)
            0x0000...0x3fff => self.rom[address as usize],

            // ROM Bank 01-7f (RO)
            0x4000...0x7fff => {
                let addr: usize = (self.rom_select as usize) * 0x4000 + (address as usize) - 0x4000;
                self.rom[addr]
            }

            // RAM Bank 00-03 (RW) && RTC Register 08-0C (RW)
            0xa000...0xbfff => {
                match self.ram_rtc_select {
                    RamRtcSelect::Ram(x) if x <= 3 => {
                        let addr: usize = (x as usize) * 0x2000 + (address as usize) - 0xa000;
                        self.ram[addr]
                    }
                    RamRtcSelect::Rtc(x) if x <= 4 => {
                        let addr: usize = (x as usize) * 0x2000 + (address as usize) - 0xa000;
                        self.ram[addr]
                    }
                    _ => panic!("Bad Ram Rtc setting"),
                }
            }

            // Error Read
            _ => {
                warn!("Bad read!");
                0x00
            }

        }
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            // RAM and Time Enable (WO)
            0x0000...0x1fff => {
                match value {
                    0x00 => self.ram_timer_enabled = false,
                    0x0a => self.ram_timer_enabled = true,
                    _ => (),//warn!("Bad RAM and Time Enable Setting"),
                }
            }

            // ROM Bank Number (WO)
            0x2000...0x3fff => {
                // only cares about lower 7-bits
                let val = value & !0x80;
                match val {
                    0x00 => self.rom_select = 0x01,
                    x => self.rom_select = x,
                }
            }

            // RAM Bank Number || RTC Register Select (WO)
            0x4000...0x5fff => {
                match value {
                    0x00...0x03 => self.ram_rtc_select = RamRtcSelect::Ram(value),
                    0x08...0x0c => self.ram_rtc_select = RamRtcSelect::Rtc(value - 0x08),
                    _ => (),//warn!("Bad RAM Bank / RTC Register"),
                }
            }

            // Latch Clock Data (WO)
            0x6000...0x7fff => {
                match value {
                    0x00 => (),//unimplemented!(),
                    0x01 => (),//unimplemented!(),
                    _ => (),//unimplemented!(),
                }
            }

            // RAM Bank 00-03 (RW) && RTC Register 08-0C (RW)
            0xa000...0xbfff => {
                match self.ram_rtc_select {
                    RamRtcSelect::Ram(x) if x <= 3 => {
                        let addr: usize = (x as usize) * 0x2000 + (address as usize) - 0xa000;
                        self.ram[addr] = value;
                    }
                    RamRtcSelect::Rtc(x) if x <= 4 => {
                        let addr: usize = (x as usize) * 0x2000 + (address as usize) - 0xa000;
                        self.ram[addr] = value;
                    }
                    _ => (),//warn!("Bad Ram Rtc setting"),
                }
            }

            _ => warn!("Bad write!"),
        }
    }
}

impl Debug for Mbc3 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let ram: &[u8] = &self.ram;
        let rtc: &[u8] = &self.rtc;

        f.debug_struct("Mbc3")
            .field("bios", &self.rom)
            .field("rom", &ram)
            .field("rtc", &rtc)
            .field("ram_timer_enabled", &self.ram_timer_enabled)
            .field("rom_select", &self.rom_select)
            .field("ram_rtc_select", &self.ram_rtc_select)
            .finish()
    }
}
