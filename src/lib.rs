//! A Game Boy emulator written in Rust.

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

extern crate byteorder;
extern crate itertools;

pub mod cpu;
pub mod errors;
pub mod memory;

use std::cell::RefCell;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::rc::Rc;

use errors::*;
use memory::Mmu;
use cpu::Cpu;

pub struct Emulator {
    cpu: Cpu,
    mmu: Rc<RefCell<Mmu>>,
}

impl Emulator {
    pub fn new() -> Self {
        let mmu = Rc::new(RefCell::new(Mmu::new()));
        let cpu = Cpu::new(Rc::clone(&mmu));

        Emulator { mmu: mmu, cpu: cpu }
    }

    pub fn load_bios<P>(&mut self, path: P) -> Result<()>
        where P: AsRef<Path>
    {
        let mut file = File::open(path)?;

        let mut buf = vec![];
        file.read_to_end(&mut buf)?;

        self.mmu.borrow_mut().load_bios(&buf)?;

        Ok(())
    }

    pub fn load_rom<P>(&mut self, path: P) -> Result<()>
        where P: AsRef<Path>
    {
        let mut file = File::open(path)?;

        let mut buf = vec![];
        file.read_to_end(&mut buf)?;

        self.mmu.borrow_mut().load_rom(&buf)?;

        Ok(())
    }

    pub fn dump_memory(&self) -> String {
        self.mmu.borrow().to_string()
    }

    pub fn step(&mut self) {
        self.cpu.step()
    }
}
