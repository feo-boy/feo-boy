//! Functionality related to the CPU.
//!
//! Contains an implementation of the registers and instruction set.

mod instructions;
mod registers;

use std::default::Default;
use std::fmt;

use bus::Bus;
use memory::{Addressable, Mmu};

pub use self::instructions::Instruction;
pub use self::registers::{Registers, Flags};

/// Current state of the CPU.
#[derive(Debug)]
pub enum State {
    /// The CPU is executing code.
    Running,

    /// The CPU is paused while waiting for an interrupt or reset.
    Halted,

    /// The CPU is paused while waiting for a button press or reset.
    Stopped,

    /// The CPU executed an illegal instruction and requires a reset.
    Locked,
}

impl Default for State {
    fn default() -> State {
        State::Running
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

/// Contains whether an interrupt is enabled or requested.
#[derive(Debug, Default)]
pub struct InterruptState {
    /// Whether the interrupt has been enabled via the Interrupt Enable I/O register (`0xFFFF`).
    /// Note that this is independent of the Interrupt Master Enable (IME) flag.
    pub enabled: bool,

    /// Whether the interrupt has been requested by the program (by writing to the Interrupt Flag
    /// I/O register `0xFF0F`) or triggered by a condition.
    pub requested: bool,
}

/// CPU interrupt state.
#[derive(Debug, Default)]
pub struct Interrupts {
    /// Interrupt Master Enable (IME).
    ///
    /// This state overrides whether individual interrupts are enabled. When an interrupt is
    /// triggered, it is set to `false`. A program may control this flag through the `EI`, `DI`,
    /// and `RETI` instructions.
    pub enabled: bool,

    /// The V-blank interrupt occurs when the PPU has completed scanning the LCD lines, signalling
    /// that video memory is no longer being read.
    pub vblank: InterruptState,

    /// The LCD Status interrupt may be triggered by a number of conditions. These conditions are
    /// controlled by the LCD Status I/O register (`0xFF42`).
    pub lcd_status: InterruptState,

    /// The timer interrupt is triggered when the Timer Counter I/O register (`0xFF05`) overflows.
    pub timer: InterruptState,

    /// The serial interrupt is triggered when a data transfer has completed over the serial port.
    pub serial: InterruptState,

    /// The joypad interrupt is triggered when any button is pressed.
    ///
    /// On real hardware, this interrupt may activate multiple times per button press due to
    /// fluctuating signal.
    pub joypad: InterruptState,
}

/// The CPU.
#[derive(Debug, Default)]
pub struct Cpu {
    /// Registers
    pub reg: Registers,

    /// The clock corresponding to the last instruction cycle.
    pub clock: Clock,

    /// The state of execution.
    state: State,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu::default()
    }

    /// Fetch and execute a single instruction.
    ///
    /// Returns the number of cycles the instruction takes.
    pub fn step(&mut self, bus: &mut Bus) -> u32 {
        match self.state {
            State::Running => (),
            _ => return 0,
        }

        let instruction = self.fetch(bus);
        self.execute(instruction, bus)
    }

    /// Execute any enabled interrupt requests.
    pub fn handle_interrupts(&mut self, bus: &mut Bus) {
        if !bus.interrupts.enabled {
            return;
        }

        if bus.interrupts.vblank.enabled && bus.interrupts.vblank.requested {
            bus.interrupts.enabled = false;
            bus.interrupts.vblank.requested = false;
            self.rst(0x0040, bus);
        }
    }

    /// Push a value onto the stack.
    ///
    /// Uses the current value of `SP`, and decrements it.
    pub fn push<B: Addressable>(&mut self, value: u16, bus: &mut B) {
        self.reg.sp = self.reg.sp.wrapping_sub(2);
        bus.write_word(self.reg.sp, value);
    }

    /// Pop a value off the stack.
    ///
    /// Uses the current value of `SP`, and increments it.
    pub fn pop<B: Addressable>(&mut self, bus: &B) -> u16 {
        let value = bus.read_word(self.reg.sp);
        self.reg.sp = self.reg.sp.wrapping_add(2);
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

        self.state = State::Running;
    }
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.reg.to_string())
    }
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

/// Returns `true` if the subtraction of `b` from `a` requires a borrow from the high nibble to the
/// low nibble.
pub fn is_half_carry_sub(a: u8, b: u8) -> bool {
    (a & 0xf) < (b & 0xf)
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

        assert!(super::is_half_carry_sub(0xf0, 0x01));
        assert!(!super::is_half_carry_sub(0xff, 0xf0));
        assert!(super::is_half_carry_sub(0x3e, 0x0f));
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
        let mut bus = [0u8; 0x10000];
        let mut cpu = Cpu::new();

        cpu.reg.sp = 0xFFF0;

        cpu.push(0xcafe, &mut bus);
        assert_eq!(cpu.pop(&bus), 0xcafe);

        cpu.reg.sp = 0;
        cpu.push(0xbeef, &mut bus);
        assert_eq!(cpu.pop(&bus), 0xbeef);
    }
}
