//! Functionality related to the CPU.
//!
//! Contains an implementation of the registers and instruction set.

mod instructions;
mod registers;

use std::default::Default;
use std::fmt;

use memory::{Addressable, Mmu};

pub use self::instructions::Instruction;
pub use self::registers::{Registers, Flags, ZERO, SUBTRACT, HALF_CARRY, CARRY};

/// The clock.
#[derive(Debug, Default)]
pub struct Clock {
    /// Machine cycle state. One machine cycle = 4 clock cycles.
    pub m: u32,
    /// Clock cycle state.
    pub t: u32,
}

impl Clock {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn reset(&mut self) {
        self.m = 0;
        self.t = 0;
    }
}

/// Whether various CPU interrupts are enabled.
#[derive(Debug, Default)]
pub struct Interrupts {
    pub lcd_stat: bool,
    pub timer: bool,
    pub serial: bool,
    pub joypad: bool,
}

/// The CPU.
#[derive(Debug, Default)]
pub struct Cpu {
    /// Registers
    pub reg: Registers,

    /// The clock corresponding to the last instruction cycle.
    pub clock: Clock,

    /// True if interrupts are enabled.
    interrupts: bool,

    /// True if the CPU is halted.
    halted: bool,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu::default()
    }

    /// Fetch and execute a single instruction.
    ///
    /// Returns the number of cycles the instruction takes.
    pub fn step<B: Addressable>(&mut self, bus: &mut B) -> u32 {
        let instruction = self.fetch(bus);
        self.execute(instruction, bus)
    }

    /// Push a value onto the stack.
    ///
    /// Uses the current value of `SP`, and decrements it.
    pub fn push<B: Addressable>(&mut self, value: u16, bus: &mut B) {
        self.reg.sp -= 2;
        bus.write_word(self.reg.sp, value);
    }

    /// Pop a value off the stack.
    ///
    /// Uses the current value of `SP`, and increments it.
    pub fn pop<B: Addressable>(&mut self, bus: &B) -> u16 {
        let value = bus.read_word(self.reg.sp);
        self.reg.sp += 2;
        value
    }

    /// Reset registers to their initial values.
    pub fn reset(&mut self, mmu: &Mmu) {
        // Skip the BIOS if we didn't load it.
        self.reg.pc = if !mmu.has_bios() {
            info!("skipping BIOS: none loaded");
            0x100
        } else {
            0x00
        };
    }
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.reg.to_string())
    }
}

/// Returns `true` if the addition of two bytes requires a carry.
pub fn is_carry_add(a: u8, b: u8) -> bool {
    a.wrapping_add(b) < a
}

/// Returns `true` if the addition of two 16-bit numbers requires a carry.
pub fn is_carry_add_16(a: u16, b: u16) -> bool {
    a.wrapping_add(b) < a
}

/// Returns `true` if the addition of two bytes would require a half carry (a carry from the low
/// nibble to the high nibble).
pub fn is_half_carry_add(a: u8, b: u8) -> bool {
    (((a & 0xf).wrapping_add(b & 0xf)) & 0x10) == 0x10
}

/// Returns `true` if the addition of two 16-bit numbers would require a half carry (a carry from
/// bit 11 to 12, zero-indexed).
pub fn is_half_carry_add_16(a: u16, b: u16) -> bool {
    (((a & 0xfff).wrapping_add(b & 0xfff)) & 0x1000) == 0x1000
}

/// Returns `true` if the subtraction of two bytes would not require a carry from the most
/// significant bit.
pub fn is_carry_sub(a: u8, b: u8) -> bool {
    a > b
}

/// Returns `true` if the subtraction of two bytes would not require a half carry (a borrow from
/// the high nibble to the low nibble).
pub fn is_half_carry_sub(a: u8, b: u8) -> bool {
    (a & 0xf) > (b & 0xf)
}

#[cfg(test)]
mod tests {
    use bus::Bus;

    use super::Cpu;

    #[test]
    fn half_carry() {
        assert!(super::is_half_carry_add(0x0f, 0x01));
        assert!(!super::is_half_carry_add(0x37, 0x44));

        assert!(super::is_half_carry_add_16(0x0fff, 0x0fff));
        assert!(super::is_half_carry_add_16(0x0fff, 0x0001));
        assert!(!super::is_half_carry_add_16(0x0000, 0x0001));

        assert!(!super::is_half_carry_sub(0xf0, 0x01));
        assert!(super::is_half_carry_sub(0xff, 0xf0));
    }

    #[test]
    fn carry() {
        assert!(super::is_carry_add(0xff, 0xff));
        assert!(super::is_carry_add(0xff, 0x01));
        assert!(!super::is_carry_add(0x00, 0x01));

        assert!(super::is_carry_add_16(0xffff, 0xffff));
        assert!(super::is_carry_add_16(0xffff, 0x0001));
        assert!(!super::is_carry_add_16(0x0000, 0x0001));

        assert!(!super::is_carry_sub(0x00, 0x01));
        assert!(super::is_carry_sub(0xff, 0x0f));
    }


    #[test]
    fn skip_bios() {
        let bus = Bus::default();
        let mut cpu = Cpu::new();
        cpu.reset(&bus.mmu);

        assert_eq!(cpu.reg.pc, 0x100);

        let mut bus = Bus::default();
        let mut cpu = Cpu::new();

        // Load dummy BIOS
        bus.mmu.load_bios(&[0; 256]).unwrap();
        cpu.reset(&bus.mmu);

        assert_eq!(cpu.reg.pc, 0x00);
    }

    #[test]
    fn push_pop() {
        let mut bus = Bus::default();
        let mut cpu = Cpu::new();

        cpu.reg.sp = 0xFFF0;

        cpu.push(0xcafe, &mut bus);
        assert_eq!(cpu.pop(&bus), 0xcafe);
    }
}
