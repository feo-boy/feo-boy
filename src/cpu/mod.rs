//! Functionality related to the CPU.
//!
//! Contains an implementation of the registers and instruction set.

mod instructions;

use std::default::Default;
use std::fmt;
use std::num::Wrapping;
use std::ops::{AddAssign, SubAssign};

use byteorder::{BigEndian, ByteOrder};

use bus::Bus;
use memory::{Addressable, Mmu};

pub use self::instructions::Instruction;

bitflags! {
    /// CPU status flags.
    #[derive(Default)]
    pub struct Flags: u8 {
        /// Set if the value of the computation is zero.
        const ZERO          = 0b10000000;

        /// Set if the last operation was a subtraction.
        const SUBTRACT      = 0b01000000;

        /// Set if there was a carry from bit 3 to bit 4.
        const HALF_CARRY    = 0b00100000;

        /// Set if the result did not fit in the register.
        const CARRY         = 0b00010000;
    }
}

/// Two mutable registers treated as a pair (`BC`, `DE`, `HL`).
///
/// Addition and subtraction may be performed on each pair.
#[derive(Debug)]
pub struct RegisterPairMut<'a> {
    hi: &'a mut u8,
    lo: &'a mut u8,
}

impl<'a> RegisterPairMut<'a> {
    /// Returns the register pair as a word.
    pub fn as_word(&self) -> u16 {
        BigEndian::read_u16(&[*self.hi, *self.lo])
    }

    /// Write a word to the register pair.
    ///
    /// # Examples
    ///
    /// ```
    /// use feo_boy::cpu::Registers;
    ///
    /// let mut registers = Registers::new();
    ///
    /// registers.bc_mut().write(0xABCD);
    ///
    /// assert_eq!(registers.bc(), 0xABCD);
    /// assert_eq!(registers.b, 0xAB);
    /// assert_eq!(registers.c, 0xCD);
    /// ```
    pub fn write(&mut self, value: u16) {
        let mut bytes = [0u8; 2];
        BigEndian::write_u16(&mut bytes, value);

        *self.hi = bytes[0];
        *self.lo = bytes[1];
    }
}

impl<'a> AddAssign<u16> for RegisterPairMut<'a> {
    fn add_assign(&mut self, rhs: u16) {
        let pair = Wrapping(BigEndian::read_u16(&[*self.hi, *self.lo])) + Wrapping(rhs);

        self.write(pair.0)
    }
}

impl<'a> SubAssign<u16> for RegisterPairMut<'a> {
    fn sub_assign(&mut self, rhs: u16) {
        let pair = Wrapping(BigEndian::read_u16(&[*self.hi, *self.lo])) - Wrapping(rhs);

        self.write(pair.0)
    }
}

/// The registers.
///
/// # Examples
///
/// Registers are often operated on in pairs. For convenience, assigning addition and subtraction
/// may be performed on each pair.
///
/// ```
/// use feo_boy::cpu::Registers;
///
/// let mut registers = Registers::new();
/// {
///     let mut de = registers.de_mut();
///     de += 1;
/// }
///
/// assert_eq!(registers.de(), 0x0001);
/// assert_eq!(registers.d, 0x00);
/// assert_eq!(registers.e, 0x01);
/// ```
///
/// ```
/// use feo_boy::cpu::Registers;
///
/// let mut registers = Registers::new();
/// {
///     let mut hl = registers.hl_mut();
///     hl.write(0xFFFF);
///     hl -= 0xF;
/// }
///
/// assert_eq!(registers.hl(), 0xFFF0);
/// assert_eq!(registers.h, 0xFF);
/// assert_eq!(registers.l, 0xF0);
/// ```
///
/// To avoid saving the pair to a local variable, you may use the `AddAssign` and `SubAssign`
/// traits directly instead of the operator.
///
/// ```
/// use std::ops::{AddAssign, SubAssign};
/// use feo_boy::cpu::Registers;
///
/// let mut registers = Registers::new();
///
/// registers.bc_mut().add_assign(0xFF);
///
/// assert_eq!(registers.bc(), 0x00FF);
/// ```
#[derive(Debug, Default)]
pub struct Registers {
    /// Accumulator
    pub a: u8,

    /// Status flags
    pub f: Flags,

    // General registers
    pub b: u8,
    pub c: u8,

    pub d: u8,
    pub e: u8,

    pub h: u8,
    pub l: u8,

    /// Program counter
    pub pc: u16,

    /// Stack pointer
    pub sp: u16,
}

impl Registers {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn af(&self) -> u16 {
        BigEndian::read_u16(&[self.a, self.f.bits])
    }

    pub fn af_mut(&mut self) -> RegisterPairMut {
        RegisterPairMut {
            hi: &mut self.a,
            lo: &mut self.f.bits,
        }
    }

    pub fn bc(&self) -> u16 {
        BigEndian::read_u16(&[self.b, self.c])
    }

    pub fn bc_mut(&mut self) -> RegisterPairMut {
        RegisterPairMut {
            hi: &mut self.b,
            lo: &mut self.c,
        }
    }

    pub fn de(&self) -> u16 {
        BigEndian::read_u16(&[self.d, self.e])
    }

    pub fn de_mut(&mut self) -> RegisterPairMut {
        RegisterPairMut {
            hi: &mut self.d,
            lo: &mut self.e,
        }
    }

    pub fn hl(&self) -> u16 {
        BigEndian::read_u16(&[self.h, self.l])
    }

    pub fn hl_mut(&mut self) -> RegisterPairMut {
        RegisterPairMut {
            hi: &mut self.h,
            lo: &mut self.l,
        }
    }
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "A {:#04x}", self.a)?;
        writeln!(f, "B {:#04x}  {:#04x} C", self.b, self.c)?;
        writeln!(f, "D {:#04x}  {:#04x} E", self.d, self.e)?;
        writeln!(f, "H {:#04x}  {:#04x} L", self.h, self.l)?;
        writeln!(f)?;
        writeln!(f, "SP {:#06x}", self.sp)?;
        writeln!(f, "PC {:#06x}", self.pc)?;
        writeln!(f)?;
        writeln!(f, "  ZNHC")?;
        writeln!(f, "F {:08b}", self.f)?;

        Ok(())
    }
}

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
    pub fn step(&mut self, bus: &mut Bus) -> u32 {
        let instruction = self.fetch(bus);
        self.execute(instruction, bus)
    }

    /// Push a value onto the stack.
    ///
    /// Uses the current value of `SP`, and decrements it.
    pub fn push(&mut self, value: u16, bus: &mut Bus) {
        self.reg.sp -= 2;
        bus.write_word(self.reg.sp, value);
    }

    /// Pop a value off the stack.
    ///
    /// Uses the current value of `SP`, and increments it.
    pub fn pop(&mut self, bus: &Bus) -> u16 {
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

#[cfg(test)]
mod tests {
    use std::ops::SubAssign;

    use bus::Bus;

    use super::{Cpu, Registers};

    #[test]
    fn wrap_pair() {
        let mut registers = Registers::default();

        registers.hl_mut().sub_assign(1);

        assert_eq!(registers.h, 0xFF);
        assert_eq!(registers.l, 0xFF);
    }

    #[test]
    fn conversion_equals_immutable() {
        let mut registers = Registers::default();

        registers.hl_mut().write(0xBEEF);

        assert_eq!(0xBEEF, registers.hl_mut().as_word());
        assert_eq!(registers.hl_mut().as_word(), registers.hl());
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
