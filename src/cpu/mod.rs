//! Functionality related to the CPU.
//!
//! Contains an implementation of the registers and instruction set.

mod instructions;

use std::cell::RefCell;
use std::default::Default;
use std::fmt;
use std::rc::Rc;

use memory::Mmu;

bitflags! {
    /// CPU status flags.
    #[derive(Default)]
    struct Flags: u8 {
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

/// The registers.
#[derive(Debug, Default)]
pub struct Registers {
    /// Accumulator
    a: u8,

    /// Status flags
    f: Flags,

    // General registers
    b: u8,
    c: u8,

    d: u8,
    e: u8,

    h: u8,
    l: u8,

    /// Program counter
    pc: u16,

    /// Stack pointer
    sp: u16,
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

    pub fn dec_hl(&mut self) {
        let x = self.read_hl() - 1;
        self.write_hl(x);
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
#[derive(Debug)]
pub struct Cpu {
    /// Registers
    reg: Registers,

    /// The clock corresponding to the last instruction cycle.
    clock: Clock,

    /// Memory unit
    mmu: Rc<RefCell<Mmu>>,

    /// The operands for the current instruction.
    operands: [u8; 2],
}

impl Cpu {
    pub fn new(mmu: Rc<RefCell<Mmu>>) -> Cpu {
        Cpu {
            reg: Registers::new(),
            clock: Clock::new(),
            mmu: mmu,
            operands: Default::default(),
        }
    }

    /// Fetch and execute a single instruction.
    pub fn step(&mut self) {
        let instruction = self.fetch();

        self.execute(&instruction);
    }
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.reg.to_string())
    }
}
