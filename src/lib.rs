//! A Game Boy emulator written in Rust.

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;

pub mod cpu;
pub mod errors;
pub mod memory;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use errors::*;
use memory::Mmu;

pub struct Emulator {
    mmu: Mmu,
}

impl Emulator {
    pub fn new() -> Self {
        Emulator { mmu: Mmu::new() }
    }

    pub fn load_bios<P>(&mut self, path: P) -> Result<()>
        where P: AsRef<Path>
    {
        let mut file = File::open(path)?;

        let mut buf = vec![];
        file.read_to_end(&mut buf)?;

        self.mmu.load_bios(&buf)?;

        Ok(())
    }

    pub fn dump_memory(&self) -> String {
        self.mmu.to_string()
    }
}
