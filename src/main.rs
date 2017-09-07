extern crate feo_boy;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate error_chain;

extern crate image;
extern crate piston_window;
extern crate pretty_env_logger;

use std::path::PathBuf;

use clap::{App, AppSettings, Arg};
use image::RgbaImage;
use piston_window::*;

use feo_boy::{Emulator, SCREEN_DIMENSIONS};
use feo_boy::errors::*;

#[derive(Debug, Clone)]
struct Config {
    rom: PathBuf,
    bios: Option<PathBuf>,
    debug: bool,
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

    let mut window: PistonWindow = WindowSettings::new("FeO Boy", [512; 2]).build().unwrap();
    window.set_ups(60);

    let (width, height) = SCREEN_DIMENSIONS;
    let mut buffer = RgbaImage::new(width, height);
    let mut texture = Texture::from_image(&mut window.factory, &buffer, &TextureSettings::new())
        .unwrap();

    while let Some(event) = window.next() {
        if let Some(update_args) = event.update_args() {
            emulator.update(&update_args)?;
        }

        if event.render_args().is_some() {
            emulator.render(&mut buffer);
            texture.update(&mut window.encoder, &buffer).unwrap();

            window.draw_2d(&event, |context, graphics| {
                image(&texture, context.transform, graphics);
            });
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
