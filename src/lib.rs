//! A Game Boy emulator written in Rust.

#[macro_use]
extern crate error_chain;

pub mod errors;
pub mod memory;
pub mod cpu;
