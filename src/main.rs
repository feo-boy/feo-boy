extern crate feo_boy;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate error_chain;

extern crate pretty_env_logger;

use std::io::prelude::*;
use std::io;
use std::path::PathBuf;

use clap::{App, AppSettings, Arg};

use feo_boy::Emulator;
use feo_boy::errors::*;

#[derive(Debug, Clone)]
struct Config {
    rom: PathBuf,
    bios: Option<PathBuf>,
    debug: bool,
}

fn start_emulator(config: Config) -> Result<()> {
    let mut emulator = Emulator::new();

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
        if config.debug {
            print!("feo debug [sdcq?]: ");
            io::stdout().flush()?;

            if let Some(answer) = stdin.next() {
                match answer?.as_str() {
                    "s" => emulator.step(),
                    "d" => println!("{}", emulator.dump_memory()),
                    "c" => println!("{}", emulator.dump_state()),
                    "q" => break,
                    "?" => {
                        println!("d: dump memory");
                        println!("c: cpu state");
                        println!("s: step emulator");
                        println!("q: quit");
                    }
                    _ => println!("unknown command"),
                }
            }
        } else {
            emulator.step();
        }
    }

    Ok(())
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
