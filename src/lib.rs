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
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::rc::Rc;

use cpu::Cpu;
use errors::*;
use graphics::Ppu;
use memory::Mmu;

pub struct Emulator {
    cpu: Rc<RefCell<Cpu>>,
    mmu: Rc<RefCell<Mmu>>,
}

impl Emulator {
    pub fn new() -> Self {
        let ppu = Rc::new(RefCell::new(Ppu::new()));
        let mmu = Rc::new(RefCell::new(Mmu::new(Rc::clone(&ppu))));
        let cpu = Rc::new(RefCell::new(Cpu::new(Rc::clone(&mmu))));

        Emulator { mmu: mmu, cpu: cpu }
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

    pub fn dump_memory(&self) -> String {
        self.mmu.borrow().to_string()
    }

    pub fn dump_state(&self) -> String {
        self.cpu.borrow_mut().to_string()
    }

    pub fn step(&mut self) {
        self.cpu.borrow_mut().step()
    }
}
