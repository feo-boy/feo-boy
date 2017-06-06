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

pub mod cpu;
pub mod graphics;
pub mod errors;
pub mod memory;

use std::cell::RefCell;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::rc::Rc;

use cpu::Cpu;
use errors::*;
use graphics::Gpu;
use memory::Mmu;

pub struct Emulator {
    cpu: Rc<RefCell<Cpu>>,
    mmu: Rc<RefCell<Mmu>>,
    gpu: Gpu,
}

impl Emulator {
    pub fn new() -> Self {
        let mmu = Rc::new(RefCell::new(Mmu::new()));
        let cpu = Rc::new(RefCell::new(Cpu::new(Rc::clone(&mmu))));
        let gpu = Gpu::new(Rc::clone(&mmu), Rc::clone(&cpu));

        Emulator {
            mmu: mmu,
            cpu: cpu,
            gpu: gpu,
        }
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

    pub fn dump_state(&self) -> String {
        self.cpu.borrow_mut().to_string()
    }

    pub fn step(&mut self) {
        self.cpu.borrow_mut().step()
    }
}
