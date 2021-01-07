use std::fs;
use std::path::PathBuf;
use std::process;

use anyhow::{Context, Result};
use log::*;
use structopt::clap::AppSettings::*;
use structopt::StructOpt;

use feo_boy::Emulator;

#[derive(Debug, StructOpt)]
#[structopt(setting(ColorAuto), setting(ColoredHelp))]
#[structopt(author, about)]
struct Opt {
    /// A file containing a ROM to load into the emulator.
    rom: PathBuf,

    /// A file containing a binary dump of the Game Boy BIOS.
    ///
    /// If not supplied, the emulator will begin executing the ROM as if the BIOS had succeeded.
    #[structopt(long)]
    bios: Option<PathBuf>,

    /// Pixel scaling factor.
    ///
    /// Each pixel on the emulator screen is scaled by this amount to map to the host screen.
    #[structopt(long, default_value = "1")]
    scaling: u8,

    /// Enable debug mode.
    #[structopt(short, long)]
    debug: bool,
}

fn run(opt: Opt) -> Result<()> {
    let mut builder = Emulator::builder();

    if opt.debug {
        builder = builder.with_debug();
    }

    let mut emulator = builder.build();

    if let Some(bios) = &opt.bios {
        info!("loading BIOS from file '{}'", bios.display());
        let bios = fs::read(&bios).context("could not read BIOS")?;
        emulator.load_bios(&bios).context("could not load BIOS")?;
    }

    info!("loading ROM from file '{}'", opt.rom.display());
    let rom = fs::read(&opt.rom).context("could not read ROM")?;
    emulator.load_rom(&rom).context("could not load ROM")?;

    emulator.run()
}

fn main() {
    pretty_env_logger::init();
    let opt = Opt::from_args();

    if let Err(e) = run(opt) {
        eprintln!("fatal error: {:?}", e);

        if let Some(pixels::Error::AdapterNotFound) = e.downcast_ref() {
            eprintln!("help: ensure your graphics adapter supports Vulkan");
        }

        process::exit(1);
    }
}
