//! Functionality related to the CPU.
//!
//! Contains an implementation of the registers and instruction set.

pub mod arithmetic;
mod instructions;
mod registers;

use std::default::Default;
use std::fmt::{self, Display};

use crate::bus::Bus;
use derive_more::{Add, AddAssign, Sub, SubAssign};
use log::*;

pub use self::instructions::Instruction;
pub use self::registers::{Flags, Registers};

/// CPU frequency in Hz.
pub const FREQUENCY: u32 = 4_194_304;

/// Machine cycles. The minimum number of cycles that must occur before another instruction can be
/// decoded.
///
/// All instructions take 1-6 whole M-cycles to complete. An M-cycle is equivalent to 4 T-cycles.
#[derive(
    Debug, Default, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Add, AddAssign, Sub, SubAssign,
)]
pub struct MCycles(pub u32);

impl Display for MCycles {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} M-cycles", self.0)
    }
}

impl From<TCycles> for MCycles {
    fn from(t_cycles: TCycles) -> Self {
        MCycles(t_cycles.0 / 4)
    }
}

/// Time cycles. An individual clock of the CPU.
#[derive(
    Debug, Default, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Add, AddAssign, Sub, SubAssign,
)]
pub struct TCycles(pub u32);

impl Display for TCycles {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} T-cycles", self.0)
    }
}

impl From<MCycles> for TCycles {
    fn from(m_cycles: MCycles) -> Self {
        TCycles(m_cycles.0 * 4)
    }
}

/// Current state of the CPU.
#[derive(Debug, PartialEq, Eq)]
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

impl Interrupts {
    /// Returns true if there is a requested and enabled interrupt.
    pub fn pending(&self) -> bool {
        [
            &self.vblank,
            &self.lcd_status,
            &self.timer,
            &self.serial,
            &self.joypad,
        ]
        .iter()
        .any(|int| int.requested && int.enabled)
    }
}

/// The CPU.
#[derive(Debug, Default)]
pub struct Cpu {
    /// Registers
    pub reg: Registers,

    /// The state of execution.
    pub state: State,

    halt_bug: bool,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu::default()
    }

    /// Fetch and execute a single instruction.
    pub fn step(&mut self, bus: &mut Bus) {
        match self.state {
            State::Running => {
                let instruction = self.fetch(bus);
                self.execute(&instruction, bus);
            }
            State::Halted => {
                // Tick the duration of a NOP.
                bus.tick(MCycles(1));
            }
            _ => unimplemented!(),
        }
    }

    /// Execute any enabled interrupt requests.
    pub fn handle_interrupts(&mut self, bus: &mut Bus) {
        macro_rules! handle_interrupts {
            ( $bus:expr; $( $interrupt:ident, $vector:expr ; )* ) => {
                $(
                    if $bus.interrupts.$interrupt.enabled && $bus.interrupts.$interrupt.requested {
                        debug!(concat!("handling ", stringify!($interrupt), " interrupt"));

                        if let State::Halted = self.state {
                            self.state = State::Running;
                            bus.tick(MCycles(1));
                        }

                        $bus.interrupts.enabled = false;
                        $bus.interrupts.$interrupt.requested = false;

                        // Internal delay
                        $bus.tick(MCycles(3));

                        self.rst($vector, $bus);

                        return;
                    }
                )*
            }
        }

        if bus.interrupts.enabled {
            handle_interrupts! {
                bus;
                vblank, 0x0040;
                lcd_status, 0x0048;
                timer, 0x0050;
                serial, 0x0058;
                joypad, 0x0060;
            }
        } else {
            match self.state {
                State::Running => (),
                State::Halted => {
                    let should_wake = {
                        let interrupts = [
                            &bus.interrupts.vblank,
                            &bus.interrupts.lcd_status,
                            &bus.interrupts.timer,
                            &bus.interrupts.serial,
                            &bus.interrupts.joypad,
                        ];

                        interrupts.iter().any(|int| int.enabled && int.requested)
                    };

                    if should_wake {
                        self.state = State::Running;
                        self.reg.pc += 1;
                        bus.tick(MCycles(1));
                    }
                }
                _ => unimplemented!(),
            }
        }
    }

    /// Push a value onto the stack.
    ///
    /// Uses the current value of `SP`, and decrements it.
    pub fn push(&mut self, value: u16, bus: &mut Bus) {
        self.reg.sp = self.reg.sp.wrapping_sub(2);
        bus.write_word(self.reg.sp, value);
    }

    /// Pop a value off the stack.
    ///
    /// Uses the current value of `SP`, and increments it.
    pub fn pop(&mut self, bus: &mut Bus) -> u16 {
        let value = bus.read_word(self.reg.sp);
        self.reg.sp = self.reg.sp.wrapping_add(2);
        value
    }

    /// Reset registers to their initial values.
    pub fn reset(&mut self, bios_loaded: bool) {
        if bios_loaded {
            self.reg.pc = 0x00;
        } else {
            info!("skipping BIOS: none loaded");

            // https://gbdev.io/pandocs/#power-up-sequence
            //
            // At the time of this writing, the flags value differs from the value given in the Pan
            // Docs. However, it matches the value after execution of the real BIOS in this
            // emulator, as well as the value in BGB.
            self.reg.a = 0x01;
            self.reg.f = Flags::from_bits_truncate(0x90);
            self.reg.bc_mut().write(0x0013);
            self.reg.de_mut().write(0x00d8);
            self.reg.hl_mut().write(0x014d);
            self.reg.sp = 0xfffe;
            self.reg.pc = 0x100;
        }

        self.state = State::Running;
    }
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.reg.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::bus::Bus;

    use super::Cpu;

    #[test]
    fn skip_bios() {
        let mut cpu = Cpu::new();
        cpu.reset(false);

        assert_eq!(cpu.reg.pc, 0x100);

        let mut bus = Bus::default();
        let mut cpu = Cpu::new();

        // Load dummy BIOS
        bus.mmu.load_bios(&[0; 256]).unwrap();
        cpu.reset(true);

        assert_eq!(cpu.reg.pc, 0x00);
    }

    #[test]
    fn push_pop() {
        let mut bus = Bus::default();
        let mut cpu = Cpu::new();

        cpu.reg.sp = 0xE000;

        cpu.push(0xcafe, &mut bus);
        assert_eq!(cpu.pop(&mut bus), 0xcafe);

        cpu.reg.sp = 0xD000;
        cpu.push(0xbeef, &mut bus);
        assert_eq!(cpu.pop(&mut bus), 0xbeef);
    }
}
