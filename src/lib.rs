//! A Game Boy emulator written in Rust.

#![cfg_attr(feature = "cargo-clippy", allow(needless_range_loop))]
#![cfg_attr(feature = "cargo-clippy", allow(unreadable_literal))]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

extern crate byteorder;
extern crate image;
extern crate itertools;
extern crate piston_window;
extern crate regex;
extern crate smallvec;

#[cfg(test)]
extern crate quickcheck;

pub mod bus;
pub mod bytes;
pub mod cpu;
pub mod errors;
pub mod graphics;
pub mod memory;
pub mod tui;

use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::path::Path;

use image::RgbaImage;

use bus::Bus;
use cpu::{Cpu, Instruction};
use errors::*;
use graphics::Ppu;
use memory::Mmu;

pub use graphics::SCREEN_DIMENSIONS;

const CYCLES_PER_FRAME: u32 = 70224;

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

        let mut file = File::open(path)?;

        let mut buf = vec![];
        file.read_to_end(&mut buf)?;

        self.bus.mmu.load_bios(&buf)?;

        info!("loaded BIOS successfully");

        Ok(())
    }

    /// Load a cartridge ROM into the emulator from a file.
    pub fn load_rom<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        info!("loading ROM from file '{}'", path.as_ref().display());

        let mut file = File::open(path)?;

        let mut buf = vec![];
        file.read_to_end(&mut buf)?;

        self.bus.mmu.load_rom(&buf)?;

        info!("loaded ROM successfully");

        Ok(())
    }

    /// Fetch and execute a single instruction.
    pub fn step(&mut self) {
        let cycles = self.cpu.step(&mut self.bus);
        self.bus.ppu.step(
            cycles,
            &mut self.cpu.interrupts,
            &mut self.screen_buffer,
        );

        if let Some(ref mut debugger) = self.debug {
            let pc = self.cpu.reg.pc;
            if debugger.breakpoints.contains(&pc) {
                debugger.paused = true;
            }
        }
    }

    /// Step the emulation state for 1/60 of a second.
    ///
    /// If the debugger is enabled, debug commands will be read from stdin.
    pub fn update(&mut self) -> Result<()> {
        let frame_clock = self.cpu.clock.t + CYCLES_PER_FRAME;
        while self.cpu.clock.t < frame_clock {
            if self.is_paused() {
                print!("feo debug [{}]: ", tui::COMMANDS);
                io::stdout().flush()?;

                let mut command = String::new();
                io::stdin().read_line(&mut command)?;
                tui::parse_command(self, &command.trim())?;
            } else {
                self.step();
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
        self.debug.as_ref().map_or(vec![], |d| {
            d.breakpoints.iter().cloned().collect()
        })
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
    breakpoints: HashSet<u16>,
    paused: bool,
}

impl Debugger {
    fn new() -> Debugger {
        Debugger {
            breakpoints: Default::default(),
            paused: true,
        }
    }
}
