//! Functionality related to the CPU.
//!
//! Contains an implementation of the registers and instruction set.

pub mod arithmetic;
mod clock;
mod instructions;
mod registers;
mod timer;

use std::default::Default;
use std::fmt;

use bus::Bus;
use memory::{Addressable, Mmu};

pub use self::clock::Clock;
pub use self::instructions::Instruction;
pub use self::registers::{Registers, Flags};
pub use self::timer::Timer;

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

/// The CPU.
#[derive(Debug, Default)]
pub struct Cpu {
    /// Registers
    pub reg: Registers,

    /// The clock corresponding to the last instruction cycle.
    pub clock: Clock,

    /// The state of execution.
    pub state: State,
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
                self.execute(instruction, bus)
            }
            State::Halted => self.clock.tick(4),    // Tick the duration of a NOP.
            _ => unimplemented!(),
        }
    }

    /// Execute any enabled interrupt requests.
    pub fn handle_interrupts(&mut self, bus: &mut Bus) {
        if !bus.interrupts.enabled {
            match self.state {
                State::Running => return,
                State::Halted => {
                    let interrupts = [
                        &bus.interrupts.vblank,
                        &bus.interrupts.lcd_status,
                        &bus.interrupts.timer,
                        &bus.interrupts.serial,
                        &bus.interrupts.joypad,
                    ];

                    if interrupts.iter().any(|int| int.requested) {
                        self.state = State::Running;

                        // Handle "HALT bug"
                        self.reg.pc += 1;
                    }

                    return;
                }
                _ => unimplemented!(),
            }
        }

        macro_rules! handle_interrupts {
            ( $bus:expr; $( $interrupt:ident, $vector:expr ; )* ) => {
                $(
                    if $bus.interrupts.$interrupt.enabled && $bus.interrupts.$interrupt.requested {
                        debug!(concat!("handling ", stringify!($interrupt), " interrupt"));

                        if let State::Halted = self.state {
                            self.state = State::Running;
                        }

                        $bus.interrupts.enabled = false;
                        $bus.interrupts.$interrupt.requested = false;

                        self.rst($vector, $bus);

                        // FIXME: The timing for interrupts might be more subtle than this.
                        self.clock.tick(12);
                        $bus.timer.tick(3);

                        return;
                    }
                )*
            }
        }

        handle_interrupts! {
            bus;
            vblank, 0x0040;
            lcd_status, 0x0048;
            timer, 0x0050;
            serial, 0x0058;
            joypad, 0x0060;
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

#[cfg(test)]
mod tests {
    use bus::Bus;

    use super::Cpu;

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
