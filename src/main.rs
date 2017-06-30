extern crate feo_boy;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate error_chain;

extern crate pretty_env_logger;

use std::io::prelude::*;
use std::io;
use std::path::PathBuf;
use std::process;

use clap::{App, AppSettings, Arg};

use feo_boy::Emulator;
use feo_boy::errors::*;

#[derive(Debug, Clone)]
struct Config {
    rom: PathBuf,
    bios: Option<PathBuf>,
    debug: bool,
}

fn parse_step(command: &str) -> Result<Option<i32>> {
    let components = command.split(" ").collect::<Vec<_>>();

    match components.len() {
        1 => return Ok(None),
        2 => (),
        _ => bail!("`s` takes a single optional argument"),
    }

    let step = components[1].parse().chain_err(
        || "could not parse step count",
    )?;

    Ok(Some(step))
}

fn parse_breakpoint(command: &str) -> Result<u16> {
    let components = command.split(" ").collect::<Vec<_>>();

    if components.len() != 2 {
        bail!("`b` takes a single argument");
    }

    let breakpoint = &components[1];
    if !breakpoint.starts_with("0x") {
        bail!("breakpoint must start with '0x'");
    }

    let breakpoint = u16::from_str_radix(&breakpoint[2..], 16).chain_err(
        || "could not parse hexadecimal number",
    )?;
    Ok(breakpoint)
}

fn parse_command(emulator: &mut Emulator, command: &str) -> Result<()> {
    match &command[..1] {
        "s" => {
            let step = parse_step(&command)?.unwrap_or_else(|| 1);

            for _ in 0..step {
                emulator.step();
            }
        }
        "b" => {
            let breakpoint = parse_breakpoint(&command)?;
            emulator.add_breakpoint(breakpoint);
        }
        "l" => {
            let breakpoints = emulator.breakpoints();
            if breakpoints.is_empty() {
                println!("no breakpoints");
            } else {
                println!("breakpoints:");
                for breakpoint in emulator.breakpoints() {
                    println!("{:#06x}", breakpoint);
                }
            }
        }
        "r" => emulator.resume(),
        "p" => {
            let cpu = &emulator.cpu;
            println!("{:#06x}: {}", cpu.reg.pc, cpu.fetch(&emulator.mmu))
        }
        "d" => println!("{}", emulator.mmu.to_string()),
        "c" => println!("{}", emulator.cpu.to_string()),
        "q" => process::exit(0),
        "?" => {
            println!("s: step emulator");
            println!("b: add breakpoint");
            println!("l: list breakpoints");
            println!("r: resume execution");
            println!("p: print current instruction");
            println!("d: dump memory");
            println!("c: cpu state");
            println!("q: quit");
        }
        _ => println!("unknown command"),
    }

    Ok(())
}

fn start_emulator(config: Config) -> Result<()> {
    let mut emulator = if config.debug {
        Emulator::new_with_debug()
    } else {
        Emulator::new()
    };

    if let Some(bios) = config.bios {
        emulator.load_bios(bios).chain_err(|| "could not load BIOS")?;
    }

    emulator.load_rom(config.rom).chain_err(
        || "could not load ROM",
    )?;

    emulator.reset();

    let stdin = io::stdin();
    let mut stdin = stdin.lock().lines();

    loop {
        if emulator.is_paused() {
            print!("feo debug [sblrpdcq?]: ");
            io::stdout().flush()?;

            if let Some(command) = stdin.next() {
                let command = command?.as_str().to_owned();
                parse_command(&mut emulator, &command)?;
            }
        } else {
            emulator.step();
        }
    }
}

fn run() -> Result<()> {
    pretty_env_logger::init().unwrap();

    let matches = App::new(crate_name!())
        .setting(AppSettings::ColoredHelp)
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .arg(Arg::with_name("rom").required(true).help(
            "a file containing a ROM to load into the emulator",
        ))
        .arg(Arg::with_name("bios").long("bios").takes_value(true).help(
            "a file containing a binary dump of the Game Boy BIOS. If not supplied, the emulator \
            will begin executing the ROM as if the BIOS had succeeded",
        ))
        .arg(Arg::with_name("debug").long("debug").short("d").help(
            "Enable debug mode",
        ))
        .get_matches();

    let bios = matches.value_of("bios").map(PathBuf::from);
    let rom = matches.value_of("rom").unwrap();

    let config = Config {
        bios: bios,
        rom: PathBuf::from(rom),
        debug: matches.is_present("debug"),
    };

    start_emulator(config)
}

quick_main!(run);
