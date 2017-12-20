extern crate feo_boy;

#[macro_use]
extern crate clap;

extern crate failure;
extern crate image;
extern crate piston_window;
extern crate pretty_env_logger;

use std::borrow::Cow;
use std::path::PathBuf;
use std::process;

use clap::{App, AppSettings, Arg};
use failure::ResultExt;
use image::{FilterType, RgbaImage};
use image::imageops;
use piston_window::*;

use feo_boy::{Emulator, Result, SCREEN_DIMENSIONS};

#[derive(Debug, Clone)]
struct Config {
    rom: PathBuf,
    bios: Option<PathBuf>,
    scaling: u8,
    debug: bool,
}

fn start_emulator(config: Config) -> Result<()> {
    let mut emulator = if config.debug {
        Emulator::new_with_debug()
    } else {
        Emulator::new()
    };

    if let Some(ref bios) = config.bios {
        emulator.load_bios(bios).context("could not load BIOS")?;
    }

    emulator
        .load_rom(&config.rom)
        .context("could not load ROM")?;

    emulator.reset();

    let scaled_dimensions = [
        SCREEN_DIMENSIONS.0 * u32::from(config.scaling),
        SCREEN_DIMENSIONS.1 * u32::from(config.scaling),
    ];
    let mut window: PistonWindow = WindowSettings::new("FeO Boy", scaled_dimensions)
        .build()
        .unwrap();

    let window_size = window.size();

    let mut texture = Texture::from_image(
        &mut window.factory,
        &RgbaImage::new(window_size.width, window_size.height),
        &TextureSettings::new(),
    ).unwrap();

    while let Some(event) = window.next() {
        if let Some(args) = event.button_args() {
            // TODO: Make this configurable
            let button = match args.button {
                Button::Keyboard(Key::Up) => Some(feo_boy::Button::Up),
                Button::Keyboard(Key::Down) => Some(feo_boy::Button::Down),
                Button::Keyboard(Key::Left) => Some(feo_boy::Button::Left),
                Button::Keyboard(Key::Right) => Some(feo_boy::Button::Right),
                Button::Keyboard(Key::X) => Some(feo_boy::Button::B),
                Button::Keyboard(Key::Z) => Some(feo_boy::Button::A),
                Button::Keyboard(Key::Return) => Some(feo_boy::Button::Start),
                Button::Keyboard(Key::Backspace) => Some(feo_boy::Button::Select),
                _ => None,
            };

            if let Some(button) = button {
                match args.state {
                    ButtonState::Press => emulator.press(button),
                    ButtonState::Release => emulator.release(button),
                }
            }
        }

        if let Some(args) = event.update_args() {
            emulator.update(args.dt)?;
        }

        if event.render_args().is_some() {
            let display_buffer = if config.scaling == 1 {
                Cow::Borrowed(&emulator.screen_buffer)
            } else {
                Cow::Owned(imageops::resize(
                    &emulator.screen_buffer,
                    window_size.width,
                    window_size.height,
                    FilterType::Nearest,
                ))
            };

            texture
                .update(&mut window.encoder, &display_buffer)
                .unwrap();

            window.draw_2d(&event, |context, graphics| {
                clear([1.0; 4], graphics);
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
        .arg(
            Arg::with_name("rom")
                .required(true)
                .help("a file containing a ROM to load into the emulator"),
        )
        .arg(Arg::with_name("bios").long("bios").takes_value(true).help(
            "a file containing a binary dump of the Game Boy BIOS. If not supplied, the emulator \
             will begin executing the ROM as if the BIOS had succeeded",
        ))
        .arg(
            Arg::with_name("scaling")
                .required(false)
                .long("scaling")
                .takes_value(true)
                .default_value("1")
                .help("amount to scale the emulator screen by"),
        )
        .arg(
            Arg::with_name("debug")
                .long("debug")
                .short("d")
                .help("Enable debug mode"),
        )
        .get_matches();

    let bios = matches.value_of("bios").map(PathBuf::from);
    let rom = matches.value_of("rom").unwrap();
    let scaling = value_t!(matches, "scaling", u8).unwrap_or_else(|e| e.exit());

    let config = Config {
        bios,
        rom: PathBuf::from(rom),
        debug: matches.is_present("debug"),
        scaling,
    };

    start_emulator(config)
}

fn main() {
    if let Err(e) = run() {
        eprintln!("fatal error");

        for cause in e.causes() {
            eprintln!("cause: {}", cause);
        }

        process::exit(1);
    }
}
