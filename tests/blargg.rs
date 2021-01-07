//! Tests the emulator against Blargg's test ROMs.
//!
//! Expected rom duration taken from <https://github.com/c-sp/gameboy-test-roms>.

use std::io::Read;
use std::thread;
use std::time::Duration;

use anyhow::Result;
use indoc::indoc;

use feo_boy::Emulator;

/// Creates a new emulator, runs it for a simulated duration, and then asserts the serial output
/// against the provided output.
fn assert_rom_output(rom: &'static [u8], duration: Duration, output: &str) -> Result<()> {
    let (mut read, write) = pipe::pipe();

    let thread = thread::spawn(move || -> Result<()> {
        let mut emulator = Emulator::builder().with_serial_out(write).build();

        emulator.load_rom(rom)?;
        emulator.reset();
        emulator.update(duration)?;

        Ok(())
    });

    let mut out = String::new();
    read.read_to_string(&mut out).unwrap();

    thread.join().unwrap()?;

    assert_eq!(out, output);

    Ok(())
}

#[test]
fn cpu_instrs() -> Result<()> {
    assert_rom_output(
        include_bytes!("./gb-test-roms/cpu_instrs/cpu_instrs.gb"),
        Duration::from_secs(55),
        indoc! {"
            cpu_instrs

            01:ok  02:ok  03:ok  04:ok  05:ok  06:ok  07:ok  08:ok  09:ok  10:ok  11:ok  

            Passed all tests
        "},
    )?;

    Ok(())
}

#[test]
fn instr_timing() -> Result<()> {
    assert_rom_output(
        include_bytes!("./gb-test-roms/instr_timing/instr_timing.gb"),
        Duration::from_secs(1),
        indoc! {"
            instr_timing


            Passed
        "},
    )?;

    Ok(())
}

#[test]
fn mem_timing() -> Result<()> {
    assert_rom_output(
        include_bytes!("./gb-test-roms/mem_timing/mem_timing.gb"),
        Duration::from_secs(4),
        indoc! {"
            mem_timing

            01:ok  02:ok  03:ok  

            Passed all tests
        "},
    )?;

    Ok(())
}
