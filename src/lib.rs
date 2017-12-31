//! A Game Boy emulator written in Rust.

#![cfg_attr(feature = "cargo-clippy", allow(needless_range_loop))]
#![cfg_attr(feature = "cargo-clippy", allow(unreadable_literal))]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate derive_more;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

extern crate byteorder;
extern crate image;
extern crate itertools;
extern crate regex;
extern crate rustyline;
extern crate smallvec;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

#[cfg(test)]
extern crate rand;

pub mod bus;
pub mod bytes;
pub mod cpu;
pub mod graphics;
pub mod audio;
pub mod input;
pub mod memory;
pub mod tui;

use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process;

use failure::{Error, ResultExt};
use image::RgbaImage;
use rustyline::error::ReadlineError;
use rustyline::Editor;

use bus::Bus;
use cpu::{Cpu, Instruction, TCycles};
use graphics::Ppu;
use audio::SoundController;
use memory::Mmu;

pub use graphics::SCREEN_DIMENSIONS;
pub use input::Button;

const MICROSECONDS_PER_CYCLE: f64 = 0.2384;

pub type Result<T> = std::result::Result<T, Error>;
pub(crate) type StdResult<T, E> = std::result::Result<T, E>;

/// The emulator itself. Contains all components required to emulate the Game Boy.
#[derive(Debug)]
pub struct Emulator {
    /// The CPU.
    pub cpu: Cpu,

    /// Other components of the emulator.
    pub bus: Bus,

    /// An image buffer to be drawn to the screen.
    pub screen_buffer: RgbaImage,

    debug: Option<Debugger>,
}

impl Emulator {
    /// Create a new emulator.
    pub fn new() -> Self {
        let cpu = Cpu::new();
        let bus = Bus {
            ppu: Ppu::new(),
            audio: SoundController::new(),
            mmu: Mmu::new(),
            ..Default::default()
        };

        let (width, height) = SCREEN_DIMENSIONS;

        Emulator {
            cpu,
            bus,
            screen_buffer: RgbaImage::new(width, height),
            debug: None,
        }
    }

    /// Create a new emulator with the debugger enabled.
    pub fn new_with_debug() -> Self {
        let mut emulator = Emulator::new();
        emulator.debug = Some(Debugger::new());
        emulator
    }

    /// Reset all emulator components to their initial states.
    pub fn reset(&mut self) {
        self.bus.mmu.reset();
        self.cpu.reset(&self.bus.mmu);
    }

    /// Load a BIOS dump into the emulator from a file.
    pub fn load_bios<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        info!("loading BIOS from file '{}'", path.as_ref().display());

        let mut file = File::open(path).context("could not open BIOS file")?;

        let mut buf = vec![];
        file.read_to_end(&mut buf)?;

        self.bus.mmu.load_bios(&buf)?;

        info!("loaded BIOS successfully");

        Ok(())
    }

    /// Load a cartridge ROM into the emulator from a file.
    pub fn load_rom<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        info!("loading ROM from file '{}'", path.as_ref().display());

        let mut file = File::open(path).context("could not open ROM file")?;

        let mut buf = vec![];
        file.read_to_end(&mut buf)?;

        self.bus.mmu.load_rom(&buf)?;

        info!("loaded ROM successfully");

        Ok(())
    }

    /// Fetch and execute a single instruction. Returns the number of cycles executed.
    pub fn step(&mut self) -> TCycles {
        self.bus.timer.reset_diff();

        self.cpu.step(&mut self.bus);

        self.bus.ppu.step(
            TCycles::from(self.bus.timer.diff()),
            &mut self.bus.interrupts,
            &mut self.screen_buffer,
        );

        // FIXME: Make sure the timing is correct here
        self.cpu.handle_interrupts(&mut self.bus);

        if let Some(ref mut debugger) = self.debug {
            let pc = self.cpu.reg.pc;
            if debugger.breakpoints.contains(&pc) {
                debugger.paused = true;
            }
        }

        TCycles::from(self.bus.timer.diff())
    }

    pub fn press(&mut self, button: Button) {
        self.bus.button_state.press(button);
    }

    pub fn release(&mut self, button: Button) {
        self.bus.button_state.release(button);
    }

    /// Step the emulation state for the given time in seconds.
    ///
    /// If the debugger is enabled, debug commands will be read from stdin.
    pub fn update(&mut self, dt: f64) -> Result<()> {
        let microseconds = dt * 1_000_000.0;
        let cycles_to_execute = TCycles((microseconds / MICROSECONDS_PER_CYCLE) as u32);

        let mut cycles_executed = TCycles(0);

        while cycles_executed < cycles_to_execute {
            if self.is_paused() {
                let readline = {
                    let editor = &mut self.debug.as_mut().unwrap().editor;
                    let prompt = format!("feo debug [{}] >> ", tui::COMMANDS);
                    editor.readline(&prompt)
                };

                match readline {
                    Ok(line) => {
                        self.debug.as_mut().unwrap().editor.add_history_entry(&line);
                        // FIXME: Don't propagate this error.
                        tui::parse_command(self, line.trim())?
                    }
                    Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => process::exit(0),
                    Err(err) => panic!("{}", err),
                }
            } else {
                cycles_executed += self.step();
            }
        }

        Ok(())
    }

    /// Resume execution after pausing.
    pub fn resume(&mut self) {
        if let Some(ref mut debugger) = self.debug {
            debugger.paused = false;
        }
    }

    /// Whether the emulator is paused.
    pub fn is_paused(&self) -> bool {
        self.debug.as_ref().map_or(false, |d| d.paused)
    }

    /// Insert a breakpoint at a given memory address.
    pub fn add_breakpoint(&mut self, breakpoint: u16) {
        if let Some(ref mut debugger) = self.debug {
            debugger.breakpoints.insert(breakpoint);
        }
    }

    /// Return a list of active breakpoints.
    pub fn breakpoints(&self) -> Vec<u16> {
        self.debug
            .as_ref()
            .map_or(vec![], |d| d.breakpoints.iter().cloned().collect())
    }

    /// Returns the current value of the program counter and the instruction at that memory
    /// address.
    pub fn current_instruction(&self) -> (u16, Instruction) {
        (self.cpu.reg.pc, self.cpu.fetch(&self.bus))
    }
}

impl Default for Emulator {
    fn default() -> Self {
        Emulator::new()
    }
}

#[derive(Debug, Default)]
struct Debugger {
    editor: Editor<()>,
    breakpoints: HashSet<u16>,
    paused: bool,
}

impl Debugger {
    fn new() -> Debugger {
        Debugger {
            breakpoints: Default::default(),
            paused: true,
            editor: Editor::<()>::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Emulator;
    use super::cpu::State;
    use super::memory::Addressable;

    #[test]
    fn tick_while_halted() {
        // TODO: There should be a way to load multiple instructions into ROM for testing purposes.

        let mut emulator = Emulator::new();
        emulator.cpu.state = State::Halted;

        assert_eq!(emulator.bus.timer.reg.divider(), 0);

        // Step at least enough times for the divider to increase.
        for _ in 0..64 {
            emulator.step();
        }

        assert!(emulator.bus.timer.reg.divider() > 0);
    }

    #[test]
    fn wake_from_halt() {
        let mut emulator = Emulator::new();

        // Load a test program into RAM
        emulator.cpu.reg.pc = 0xC000;
        assert!(!emulator.bus.interrupts.enabled);

        let test_program = [
            0x76,   // HALT
            0x00,   // NOP
            0x00,   // NOP
        ];

        for (offset, byte) in test_program.into_iter().enumerate() {
            emulator
                .bus
                .write_byte(emulator.cpu.reg.pc + offset as u16, *byte);
        }

        emulator.step();

        assert_eq!(emulator.cpu.state, State::Halted);
        assert_eq!(emulator.cpu.reg.pc, 0xC001);

        emulator.step();

        assert_eq!(emulator.cpu.state, State::Halted);
        assert_eq!(emulator.cpu.reg.pc, 0xC001);

        // Request an interrupt
        emulator.bus.interrupts.timer.enabled = true;
        emulator.bus.interrupts.timer.requested = true;

        emulator.step();
        assert_eq!(emulator.cpu.reg.pc, 0xC002);
        assert!(emulator.bus.interrupts.timer.requested);
    }

    #[test]
    fn halt_bug() {
        // The notorious "HALT bug". If interrupts are disabled and there is a pending interrupt
        // when a HALT instruction is encountered, then the HALT state is skipped, and the PC fails
        // to increase for the next instruction.
        let mut emulator = Emulator::new();

        // Load a test program into RAM
        emulator.cpu.reg.pc = 0xC000;

        let test_program = [
            0xAF,   // XOR A
            0x76,   // HALT
            0x3C,   // INC A    (this instruction will be executed twice)
            0x22,   // LD (HL+),A
        ];

        emulator.cpu.reg.hl_mut().write(0xD000);
        emulator.bus.interrupts.enabled = false;
        emulator.bus.interrupts.timer.enabled = true;
        emulator.bus.interrupts.timer.requested = true;

        for (offset, byte) in test_program.into_iter().enumerate() {
            emulator
                .bus
                .write_byte(emulator.cpu.reg.pc + offset as u16, *byte);
        }

        for _ in 0..5 {
            emulator.step();
        }

        assert_eq!(emulator.cpu.reg.a, 2);
        assert_eq!(emulator.bus.read_byte(0xD000), 2);
    }
}
