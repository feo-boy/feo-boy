use std::path::PathBuf;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use instant::Instant;

use anyhow::Result;
use log::*;
use pixels::{PixelsBuilder, SurfaceTexture};
use structopt::clap::AppSettings::*;
use structopt::StructOpt;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use feo_boy::{Button, Emulator, SCREEN_DIMENSIONS};

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

async fn run(mut emulator: Emulator) -> Result<()> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(SCREEN_DIMENSIONS.0, SCREEN_DIMENSIONS.1);

        let mut window_builder = WindowBuilder::new()
            .with_title("FeO Boy")
            .with_inner_size(size)
            .with_min_inner_size(size);

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use web_sys::HtmlCanvasElement;
            use winit::platform::web::WindowBuilderExtWebSys;

            let document = web_sys::window().and_then(|window| window.document()).unwrap();
            let screen: HtmlCanvasElement = document.get_element_by_id("screen")
                .expect("no element with id 'screen'")
                .dyn_into()
                .expect("element with id 'screen' was not a canvas");

            let size = LogicalSize::new(screen.width(), screen.height());
            window_builder = window_builder.with_canvas(Some(screen));
            window_builder = window_builder.with_min_inner_size(size);
            window_builder = window_builder.with_inner_size(size);
        }

        window_builder
            .build(&event_loop)
            .unwrap()
    };
    let mut hidpi_factor = window.scale_factor();

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, &window);
        PixelsBuilder::new(SCREEN_DIMENSIONS.0, SCREEN_DIMENSIONS.1, surface_texture)
            .texture_format(pixels::wgpu::TextureFormat::Bgra8Unorm)
            .build()
            .await
            .unwrap()
    };

    let mut last_update = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            emulator.render(pixels.get_frame());

            if let Err(e) = pixels.render() {
                *control_flow = ControlFlow::Exit;
                error!("unable to render: {}", e);
                return;
            }
        }

        if input.update(&event) {
            if input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            handle_keys(&input, &mut emulator);

            if let Some(factor) = input.scale_factor_changed() {
                hidpi_factor = factor;
            }

            if let Some(size) = input.window_resized() {
                // FIXME: User-specified scaling is currently ignored: parasyte/pixels/issues/89
                pixels.resize(size.width, size.height);
            }

            let current_time = Instant::now();
            if let Err(e) = emulator.update(current_time - last_update) {
                error!("unable to update emulator state: {}", e);
                *control_flow = ControlFlow::Exit;
            }
            last_update = current_time;
            window.request_redraw();
        }
    });
}

fn handle_keys(input: &WinitInputHelper, emulator: &mut Emulator) {
    macro_rules! button_mapping {
        ( $( $winit_key:expr => $feo_boy_key:expr),+ $(,)? ) => {{
            $(
                if input.key_pressed($winit_key) {
                    emulator.press($feo_boy_key)
                }
                if input.key_released($winit_key) {
                    emulator.release($feo_boy_key)
                }
            )*
        }}
    }

    button_mapping! {
        VirtualKeyCode::Up => Button::Up,
        VirtualKeyCode::Down => Button::Down,
        VirtualKeyCode::Left => Button::Left,
        VirtualKeyCode::Right => Button::Right,
        VirtualKeyCode::X => Button::B,
        VirtualKeyCode::Z => Button::A,
        VirtualKeyCode::Return => Button::Start,
        VirtualKeyCode::Back => Button::Select,
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();
    console_log::init().unwrap();
    info!("initialized");
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn run_emulator(rom: &[u8]) {
    let mut emulator = Emulator::new();

    emulator.load_rom(rom).unwrap();

    wasm_bindgen_futures::spawn_local(async move {
        if let Err(e) = run(emulator).await {
            error!("fatal error: {}", e);
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<()> {
    use std::fs;

    use anyhow::Context;

    pretty_env_logger::init();

    let opt = Opt::from_args();

    let mut emulator = if opt.debug {
        Emulator::new_with_debug()
    } else {
        Emulator::new()
    };

    if let Some(bios) = &opt.bios {
        info!("loading BIOS from file '{}'", bios.display());
        let bios = fs::read(&bios).context("could not read BIOS")?;
        emulator.load_bios(&bios).context("could not load BIOS")?;
    }

    info!("loading ROM from file '{}'", opt.rom.display());
    let rom = fs::read(&opt.rom).context("could not read ROM")?;
    emulator.load_rom(&rom).context("could not load ROM")?;

    emulator.reset();

    pollster::block_on(run(emulator))?;

    Ok(())
}
