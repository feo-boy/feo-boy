use std::borrow::Cow;
use std::path::PathBuf;
use std::process;
use std::time::Duration;

use ::image::imageops;
use ::image::{FilterType, RgbaImage};
use failure::ResultExt;
use piston_window::*;
use structopt::clap::AppSettings::*;
use structopt::StructOpt;

use feo_boy::{Emulator, Result, SCREEN_DIMENSIONS};

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
    let mut emulator = if opt.debug {
        Emulator::new_with_debug()
    } else {
        Emulator::new()
    };

    if let Some(ref bios) = opt.bios {
        emulator.load_bios(bios).context("could not load BIOS")?;
    }

    emulator.load_rom(&opt.rom).context("could not load ROM")?;

    emulator.reset();

    let scaled_dimensions = [
        SCREEN_DIMENSIONS.0 * u32::from(opt.scaling),
        SCREEN_DIMENSIONS.1 * u32::from(opt.scaling),
    ];
    let mut window: PistonWindow = WindowSettings::new("FeO Boy", scaled_dimensions)
        .build()
        .unwrap();

    let window_size = window.size();

    let mut texture = Texture::from_image(
        &mut window.factory,
        &RgbaImage::new(window_size.width, window_size.height),
        &TextureSettings::new(),
    )
    .unwrap();

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
            emulator.update(Duration::from_secs_f64(args.dt))?;
        }

        if event.render_args().is_some() {
            let display_buffer = if opt.scaling == 1 {
                Cow::Borrowed(emulator.frame_buffer())
            } else {
                Cow::Owned(imageops::resize(
                    emulator.frame_buffer(),
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

fn main() {
    pretty_env_logger::init().unwrap();
    let opt = Opt::from_args();

    if let Err(e) = run(opt) {
        eprintln!("fatal error");

        for cause in e.iter_chain() {
            eprintln!("cause: {}", cause);
        }

        process::exit(1);
    }
}
