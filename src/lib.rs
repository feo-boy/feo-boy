//! A Game Boy emulator written in Rust.

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

extern crate byteorder;
extern crate itertools;
extern crate regex;
extern crate smallvec;

pub mod bytes;
pub mod cpu;
pub mod errors;
pub mod graphics;
pub mod memory;

use std::cell::RefCell;
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::rc::Rc;

use cpu::Cpu;
use errors::*;
use graphics::Ppu;
use memory::Mmu;

pub struct Emulator {
    pub cpu: Rc<RefCell<Cpu>>,
    pub mmu: Rc<RefCell<Mmu>>,
    debug: Option<Debugger>,
}

impl Emulator {
    pub fn new() -> Self {
        let ppu = Rc::new(RefCell::new(Ppu::new()));
        let mmu = Rc::new(RefCell::new(Mmu::new(Rc::clone(&ppu))));
        let cpu = Rc::new(RefCell::new(Cpu::new(Rc::clone(&mmu))));

        Emulator {
            mmu,
            cpu,
            debug: None,
        }
    }

    pub fn new_with_debug() -> Self {
        let mut emulator = Emulator::new();
        emulator.debug = Some(Debugger::new());
        emulator
    }

    /// Reset all emulator components to their initial states.
    pub fn reset(&mut self) {
        self.mmu.borrow_mut().reset();
        self.cpu.borrow_mut().reset();
    }

    pub fn load_bios<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        info!("loading BIOS from file '{}'", path.as_ref().display());

        let mut file = File::open(path)?;

        let mut buf = vec![];
        file.read_to_end(&mut buf)?;

        self.mmu.borrow_mut().load_bios(&buf)?;

        info!("loaded BIOS successfully");

        Ok(())
    }

    pub fn load_rom<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        info!("loading ROM from file '{}'", path.as_ref().display());

        let mut file = File::open(path)?;

        let mut buf = vec![];
        file.read_to_end(&mut buf)?;

        self.mmu.borrow_mut().load_rom(&buf)?;

        info!("loaded ROM successfully");

        Ok(())
    }

    /// Fetch and execute a single instruction.
    pub fn step(&mut self) {
        self.cpu.borrow_mut().step();

        if let Some(ref mut debugger) = self.debug {
            let pc = self.cpu.borrow().reg.pc;
            if debugger.breakpoints.contains(&pc) {
                debugger.paused = true;
            }
        }
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
}

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
