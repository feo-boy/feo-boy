extern crate feo_boy;

#[macro_use]
extern crate clap;

extern crate env_logger;

use std::io::prelude::*;
use std::io;
use std::path::Path;
use std::process;

use clap::{App, Arg};

use feo_boy::Emulator;
use feo_boy::errors::*;

fn run<P>(rom: P, bios: Option<P>) -> Result<()>
    where P: AsRef<Path>
{
    let mut emulator = Emulator::new();

    if let Some(bios) = bios {
        emulator
            .load_bios(bios)
            .chain_err(|| "could not load BIOS")?;
    }

    emulator.load_rom(rom).chain_err(|| "could not load ROM")?;

    loop {
        emulator.step();
    }
}

fn main() {
    env_logger::init().unwrap();

    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("rom")
                 .required(true)
                 .help("a file containing a ROM to load into the emulator"))
        .arg(Arg::with_name("bios-file")
                 .long("bios")
                 .takes_value(true)
                 .help("a file containing a binary dump of the Game Boy BIOS"))
        .get_matches();

    let bios = matches.value_of("bios-file");
    let rom = matches.value_of("rom").unwrap();

    if let Err(ref e) = run(rom, bios) {
        let stderr = &mut io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "error: {}", e).expect(errmsg);

        for e in e.iter().skip(1) {
            writeln!(stderr, "caused by: {}", e).expect(errmsg);
        }

        if let Some(backtrace) = e.backtrace() {
            writeln!(stderr, "backtrace: {:?}", backtrace).expect(errmsg);
        }

        process::exit(1);
    }
}
