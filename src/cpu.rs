//! Functionality related to the CPU
//!
//! Contains an implementation of the registers and instruction set.

use std::default::Default;

/// The registers.
#[derive(Debug, Default)]
pub struct Registers {
    /// Accumulator
    pub a: u8,

    /// Flags
    pub f: u8,

    // General registers
    pub b: u8,
    pub c: u8,

    pub d: u8,
    pub e: u8,

    pub h: u8,
    pub l: u8,
}

impl Registers {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn read_bc(&self) -> u16 {
        self.c as u16 + ((self.b as u16) << 8)
    }

    pub fn read_de(&self) -> u16 {
        self.e as u16 + ((self.d as u16) << 8)
    }

    pub fn read_hl(&self) -> u16 {
        self.l as u16 + ((self.h as u16) << 8)
    }

    pub fn write_bc(&mut self, value: u16) {
        self.c = value as u8;
        self.b = (value >> 8) as u8;
    }

    pub fn write_de(&mut self, value: u16) {
        self.e = value as u8;
        self.d = (value >> 8) as u8;
    }

    pub fn write_hl(&mut self, value: u16) {
        self.l = value as u8;
        self.h = (value >> 8) as u8;
    }
}
