use super::Addressable;
use std::fmt::{self, Debug, Formatter};
use std::rc::Rc;

const RAM_SIZE: usize = 0x2000 * 4;
const RTC_SIZE: usize = 0x2000 * 5;
const ROM_BANK_SIZE: usize = 0x4000;
const RAM_BANK_RTC_REG_SIZE: usize = 0x2000;

pub trait Mbc: Addressable + Debug {}

impl<M: Addressable + Debug> Mbc for M {}

pub struct Mbc1 {
    rom: Rc<Vec<u8>>,
    rom_num: u8,
    ram: [u8; RAM_SIZE],
    ram_num: u8,
    ram_enabled: bool,
    rom_ram_select: bool, // TODO rename?
}

impl Mbc1 {
    pub fn new(rom: Rc<Vec<u8>>) -> Mbc1 {
        Mbc1 {
            rom,
            rom_num: 1,
            ram: [0; RAM_SIZE],
            ram_num: 0,
            ram_enabled: false,
            rom_ram_select: false,
        }
    }
}

impl Debug for Mbc1 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let ram: &[u8] = &self.ram;

        f.debug_struct("Mbc1")
            .field("rom", &self.rom)
            .field("rom_num", &self.rom_num)
            .field("ram", &ram)
            .field("ram_num", &self.ram_num)
            .field("ram_enabled", &self.ram_enabled)
            .field("rom_ram_select", &self.rom_ram_select)
            .finish()
    }
}

impl super::Addressable for Mbc1 {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000...0x3FFF => self.rom[address as usize],
            0x4000...0x7FFF => {
                let bank_start = u32::from(self.rom_num) * ROM_BANK_SIZE as u32;
                let address_offset = u32::from(address) - 0x4000;
                self.rom[(bank_start + address_offset) as usize]
            }
            0xA000...0xBFFF => {
                let bank_start = u32::from(self.ram_num) * RAM_BANK_RTC_REG_SIZE as u32;
                let address_offset = u32::from(address) - 0xA000;
                self.ram[(bank_start + address_offset) as usize]
            }
            _ => unreachable!(),
        }
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            // RAM Enabled
            0x0000...0x1FFF => {
                match value {
                    0x00 => self.ram_enabled = false,
                    0x0A => self.ram_enabled = true,
                    _ => (), //unreachable!(),
                }
            }

            // ROM Bank Num (Lower)
            0x2000...0x3FFF => {
                let lower = value & 0x1F; // TODO should I enforce this?
                let upper = self.rom_num & 0x60;
                self.rom_num = lower | upper;
                if self.rom_num % 0x20 == 0 {
                    // cannot select 0x00, 0x20, 0x40, 0x60
                    self.rom_num += 1
                }
            }
            // TODO question about how upper bits are preserved between switches

            // RAM Bank Num or ROM Bank # (Upper)
            0x4000...0x5FFF => {
                if self.rom_ram_select {
                    // rom selected
                    let lower = self.rom_num & 0x1F;
                    let upper = value & 0x03; // TODO should I enforce this?
                    self.rom_num = lower | upper;
                    if self.rom_num % 0x20 == 0 {
                        // cannot select 0x00, 0x20, 0x40, 0x60
                        self.rom_num += 1
                    }
                } else {
                    // ram select
                    self.ram_num = value & 0x03; // TODO should I enforce this?
                }
            }

            // ROM/RAM Mode Select
            0x6000...0x7FFF => match value {
                0x00 => self.rom_ram_select = false,
                0x01 => self.rom_ram_select = true,
                _ => unreachable!(),
            },

            _ => (), //unimplemented!(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum RamRtcSelect {
    Ram(u8), // 0-3
    Rtc(u8), // 8-c -> 0-4
}

pub struct Mbc3 {
    rom: Rc<Vec<u8>>,
    ram: [u8; RAM_SIZE],
    rtc: [u8; RTC_SIZE],
    ram_timer_enabled: bool,
    rom_select: u8,
    ram_rtc_select: RamRtcSelect,
}

impl Mbc3 {
    pub fn new(rom: Rc<Vec<u8>>) -> Mbc3 {
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
                let addr: usize =
                    (self.rom_select as usize) * ROM_BANK_SIZE + (address as usize) - 0x4000;
                self.rom[addr]
            }

            // RAM Bank 00-03 (RW) && RTC Register 08-0C (RW)
            0xa000...0xbfff => match self.ram_rtc_select {
                RamRtcSelect::Ram(rom_num) => {
                    debug_assert!(rom_num <= 3);
                    let addr: usize =
                        (rom_num as usize) * RAM_BANK_RTC_REG_SIZE + (address as usize) - 0xa000;
                    self.ram[addr]
                }
                RamRtcSelect::Rtc(rtc_num) => {
                    debug_assert!(rtc_num <= 4);
                    let addr: usize =
                        (rtc_num as usize) * RAM_BANK_RTC_REG_SIZE + (address as usize) - 0xa000;
                    self.rtc[addr]
                }
            },

            // Error Read
            _ => unreachable!(),
        }
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            // RAM and Time Enable (WO)
            0x0000...0x1fff => {
                self.ram_timer_enabled = match value {
                    0x00 => false,
                    0x0a => true,
                    _ => self.ram_timer_enabled,
                }
            }

            // ROM Bank Number (WO)
            0x2000...0x3fff => {
                // only cares about lower 7-bits
                self.rom_select = value & !0x80;
                if self.rom_select == 0x00 {
                    self.rom_select = 0x01;
                }
            }

            // RAM Bank Number || RTC Register Select (WO)
            0x4000...0x5fff => {
                self.ram_rtc_select = match value {
                    0x00...0x03 => RamRtcSelect::Ram(value),
                    0x08...0x0c => RamRtcSelect::Rtc(value - 0x08),
                    _ => self.ram_rtc_select,
                }
            }

            // Latch Clock Data (WO)
            0x6000...0x7fff => match value {
                0x00 => unimplemented!(), // TODO fix?
                0x01 => unimplemented!(),
                _ => unimplemented!(),
            },

            // RAM Bank 00-03 (RW) && RTC Register 08-0C (RW)
            0xa000...0xbfff => match self.ram_rtc_select {
                RamRtcSelect::Ram(bank_num) => {
                    debug_assert!(bank_num <= 3);
                    let addr: usize =
                        (bank_num as usize) * RAM_BANK_RTC_REG_SIZE + (address as usize) - 0xa000;
                    self.ram[addr] = value;
                }
                RamRtcSelect::Rtc(rtc_num) => {
                    debug_assert!(rtc_num <= 4);
                    let addr: usize =
                        (rtc_num as usize) * RAM_BANK_RTC_REG_SIZE + (address as usize) - 0xa000;
                    self.rtc[addr] = value;
                }
            },

            _ => unreachable!(),
        }
    }
}

impl Debug for Mbc3 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let ram: &[u8] = &self.ram;
        let rtc: &[u8] = &self.rtc;

        f.debug_struct("Mbc3")
            .field("rom", &self.rom)
            .field("ram", &ram)
            .field("rtc", &rtc)
            .field("ram_timer_enabled", &self.ram_timer_enabled)
            .field("rom_select", &self.rom_select)
            .field("ram_rtc_select", &self.ram_rtc_select)
            .finish()
    }
}
