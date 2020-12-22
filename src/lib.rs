//! A Game Boy emulator written in Rust.

#![allow(clippy::needless_range_loop)]
#![allow(clippy::unreadable_literal)]

pub mod audio;
pub mod bus;
pub mod bytes;
pub mod cpu;
pub mod graphics;
pub mod input;
pub mod memory;
pub mod tui;

use std::collections::HashSet;
use std::fmt::Debug;
use std::io::Write;
use std::process;
use std::time::{Duration, Instant};

use anyhow::Result;
use log::*;
use pixels::{Pixels, SurfaceTexture};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use crate::audio::SoundController;
use crate::bus::Bus;
use crate::cpu::{Cpu, Instruction, MCycles, TCycles};
use crate::graphics::Ppu;
use crate::memory::Mmu;

pub use crate::graphics::SCREEN_DIMENSIONS;
pub use crate::input::Button;

/// The amount of time it takes for a physical Game Boy to complete a single cycle.
///
/// Sourced from this [timing document](http://gameboy.mongenel.com/dmg/gbc_cpu_timing.txt).
const CYCLE_DURATION: Duration = Duration::from_nanos(234);

/// The emulator itself. Contains all components required to emulate the Game Boy.
#[derive(Debug)]
pub struct Emulator {
    /// The CPU.
    pub cpu: Cpu,

    /// Other components of the emulator.
    pub bus: Bus,

    debug: Option<Debugger>,
}

impl Emulator {
    /// Create a new emulator.
    pub fn new() -> Self {
        Self::builder().build()
    }

    /// Create a new emulator builder.
    pub fn builder() -> EmulatorBuilder {
        EmulatorBuilder::new()
    }

    /// Reset all emulator components to their initial states.
    ///
    /// If the BIOS has been loaded, remaps it and sets the PC to 0.
    ///
    /// If a BIOS was not loaded, sets register values as if the BIOS had already executed.
    pub fn reset(&mut self) {
        self.bus.mmu.reset();
        self.cpu.reset(self.bios_loaded());

        if !self.bios_loaded() {
            // https://gbdev.io/pandocs/#power-up-sequence
            //
            // TODO: The values in the Pan Docs disagree with the values in BGB.
            // Change these to match what we do when executing the BIOS.
            const IO_REGISTER_VALUES: &[(u16, u8)] = &[
                (0xff05, 0x00),
                (0xff06, 0x00),
                (0xff07, 0x00),
                (0xff10, 0x80),
                (0xff11, 0xbf),
                (0xff12, 0xf3),
                (0xff14, 0xbf),
                (0xff16, 0x3f),
                (0xff17, 0x00),
                (0xff19, 0xbf),
                (0xff1a, 0x7f),
                (0xff1b, 0xff),
                (0xff1c, 0x9f),
                (0xff1e, 0xbf),
                (0xff20, 0xff),
                (0xff21, 0x00),
                (0xff22, 0x00),
                (0xff23, 0xbf),
                (0xff24, 0x77),
                (0xff25, 0xf3),
                (0xff26, 0xf1),
                (0xff40, 0x91),
                (0xff42, 0x00),
                (0xff43, 0x00),
                (0xff45, 0x00),
                (0xff47, 0xfc),
                (0xff48, 0xff),
                (0xff49, 0xff),
                (0xff4a, 0x00),
                (0xff4b, 0x00),
                (0xffff, 0x00),
            ];

            for (addr, value) in IO_REGISTER_VALUES {
                self.bus.write_byte_no_tick(*addr, *value);
            }
        }
    }

    /// Load a BIOS dump into the emulator.
    pub fn load_bios(&mut self, bios: &[u8]) -> Result<()> {
        self.bus.mmu.load_bios(&bios)?;

        info!("loaded BIOS successfully");

        Ok(())
    }

    /// Load a cartridge ROM into the emulator.
    pub fn load_rom(&mut self, rom: &[u8]) -> Result<()> {
        self.bus.mmu.load_rom(&rom)?;

        info!("loaded ROM successfully");

        Ok(())
    }

    /// Open a graphical window and start execution of the emulator.
    pub fn run(mut self) -> Result<()> {
        let event_loop = EventLoop::new();
        let mut input = WinitInputHelper::new();
        let window = {
            let size = LogicalSize::new(SCREEN_DIMENSIONS.0, SCREEN_DIMENSIONS.1);
            WindowBuilder::new()
                .with_title("FeO Boy")
                .with_inner_size(size)
                .with_min_inner_size(size)
                .build(&event_loop)
                .unwrap()
        };
        let mut hidpi_factor = window.scale_factor();

        let mut pixels = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, &window);
            Pixels::new(SCREEN_DIMENSIONS.0, SCREEN_DIMENSIONS.1, surface_texture)?
        };

        self.reset();

        let mut last_update = Instant::now();

        event_loop.run(move |event, _, control_flow| {
            if let Event::RedrawRequested(_) = event {
                self.render(pixels.get_frame());

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

                self.handle_keys(&input);

                if let Some(factor) = input.scale_factor_changed() {
                    hidpi_factor = factor;
                }

                if let Some(size) = input.window_resized() {
                    // FIXME: User-specified scaling is currently ignored: parasyte/pixels/issues/89
                    pixels.resize(size.width, size.height);
                }

                let current_time = Instant::now();
                if let Err(e) = self.update(current_time - last_update) {
                    error!("unable to update emulator state: {}", e);
                    *control_flow = ControlFlow::Exit;
                }
                last_update = current_time;
                window.request_redraw();
            }
        });
    }

    fn handle_keys(&mut self, input: &WinitInputHelper) {
        macro_rules! button_mapping {
            ( $( $winit_key:expr => $feo_boy_key:expr),+ $(,)? ) => {{
                $(
                    if input.key_pressed($winit_key) {
                        self.press($feo_boy_key)
                    }
                    if input.key_released($winit_key) {
                        self.release($feo_boy_key)
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

    /// Render the current frame into a frame buffer.
    pub fn render(&self, frame: &mut [u8]) {
        self.bus.ppu.render(frame);
    }

    /// Fetch and execute a single instruction. Returns the number of cycles executed.
    pub fn step(&mut self) -> TCycles {
        self.bus.timer.reset_diff();

        let mut cycles = MCycles(0);

        self.cpu.handle_interrupts(&mut self.bus);
        cycles += self.bus.timer.diff();

        // FIXME: Hack: the cycle timing debug assert at the end of Cpu::execute is dependent on
        // this state, but it shouldn't be.
        self.bus.timer.reset_diff();

        self.cpu.step(&mut self.bus);
        cycles += self.bus.timer.diff();

        self.bus.audio.step(cycles.into());

        if let Some(ref mut debugger) = self.debug {
            let pc = self.cpu.reg.pc;
            if debugger.breakpoints.contains(&pc) {
                debugger.paused = true;
            }
        }

        TCycles::from(cycles)
    }

    pub fn press(&mut self, button: Button) {
        self.bus.button_state.press(button);
    }

    pub fn release(&mut self, button: Button) {
        self.bus.button_state.release(button);
    }

    /// Step the emulation state for the given time in seconds.
    ///
    /// If the debugger is enabled, debug commands will be read from stdin.
    pub fn update(&mut self, dt: Duration) -> Result<()> {
        let cycles_to_execute = TCycles((dt.as_nanos() / CYCLE_DURATION.as_nanos()) as u32);

        let mut cycles_executed = TCycles(0);

        while cycles_executed < cycles_to_execute {
            if self.is_paused() {
                let readline = {
                    let editor = &mut self.debug.as_mut().unwrap().editor;
                    let prompt = format!("feo debug [{}] >> ", tui::COMMANDS);
                    editor.readline(&prompt)
                };

                match readline {
                    Ok(line) => {
                        self.debug.as_mut().unwrap().editor.add_history_entry(&line);
                        // FIXME: Don't propagate this error.
                        tui::parse_command(self, line.trim())?
                    }
                    Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => process::exit(0),
                    Err(err) => panic!("{}", err),
                }
            } else {
                cycles_executed += self.step();
            }
        }

        Ok(())
    }

    /// Resume execution after pausing.
    pub fn resume(&mut self) {
        if let Some(ref mut debugger) = self.debug {
            debugger.paused = false;
        }
    }

    /// Whether the emulator is paused.
    pub fn is_paused(&self) -> bool {
        self.debug.as_ref().map_or(false, |d| d.paused)
    }

    /// Insert a breakpoint at a given memory address.
    pub fn add_breakpoint(&mut self, breakpoint: u16) {
        if let Some(ref mut debugger) = self.debug {
            debugger.breakpoints.insert(breakpoint);
        }
    }

    /// Return a list of active breakpoints.
    pub fn breakpoints(&self) -> Vec<u16> {
        self.debug
            .as_ref()
            .map_or(vec![], |d| d.breakpoints.iter().cloned().collect())
    }

    /// Returns the current value of the program counter and the instruction at that memory
    /// address.
    pub fn current_instruction(&self) -> (u16, Instruction) {
        (self.cpu.reg.pc, self.cpu.current_instruction(&self.bus))
    }

    fn bios_loaded(&self) -> bool {
        self.bus.mmu.has_bios()
    }
}

impl Default for Emulator {
    fn default() -> Self {
        Emulator::new()
    }
}

/// Non-default emulator options.
pub struct EmulatorBuilder {
    debug: bool,
    serial_out: Option<Box<dyn Write>>,
    playback: bool,
}

impl EmulatorBuilder {
    /// Create a new builder.
    pub fn new() -> EmulatorBuilder {
        EmulatorBuilder {
            serial_out: None,
            debug: false,
            playback: false,
        }
    }

    /// Connect the emulator's serial port to a write instance.
    pub fn with_serial_out(mut self, out: impl Write + 'static) -> Self {
        self.serial_out = Some(Box::new(out));
        self
    }

    /// Enable the debugger.
    pub fn with_debug(mut self) -> Self {
        self.debug = true;
        self
    }

    /// Enable audio playback.
    ///
    /// If the system does not support audio or the audio controller cannot be initialized, the
    /// builder will fall back to no audio.
    pub fn with_playback(mut self) -> Self {
        self.playback = true;
        self
    }

    /// Construct the emulator from the builder options.
    pub fn build(self) -> Emulator {
        let audio = if self.playback {
            SoundController::new_with_playback()
                .map_err(|err| {
                    error!("unable to initialize audio playback: {}", err);
                    err
                })
                .unwrap_or_default()
        } else {
            SoundController::default()
        };

        Emulator {
            cpu: Cpu::new(),
            bus: Bus {
                ppu: Ppu::new(),
                audio,
                mmu: Mmu::new(),
                serial_out: self.serial_out,
                ..Default::default()
            },
            debug: if self.debug {
                Some(Debugger::new())
            } else {
                None
            },
        }
    }
}

#[derive(Debug)]
struct Debugger {
    editor: Editor<()>,
    breakpoints: HashSet<u16>,
    paused: bool,
}

impl Debugger {
    fn new() -> Debugger {
        Debugger {
            breakpoints: Default::default(),
            paused: true,
            editor: Editor::<()>::new(),
        }
    }
}

impl Default for Debugger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::cpu::State;
    use super::Emulator;

    #[test]
    fn tick_while_halted() {
        // TODO: There should be a way to load multiple instructions into ROM for testing purposes.

        let mut emulator = Emulator::new();
        emulator.cpu.state = State::Halted;

        assert_eq!(emulator.bus.timer.divider(), 0);

        // Step at least enough times for the divider to increase.
        for _ in 0..64 {
            emulator.step();
        }

        assert!(emulator.bus.timer.divider() > 0);
    }

    #[test]
    fn wake_from_halt() {
        let mut emulator = Emulator::new();

        // Load a test program into RAM
        emulator.cpu.reg.pc = 0xC000;
        assert!(!emulator.bus.interrupts.enabled);

        let test_program = [
            0x76, // HALT
            0x00, // NOP
            0x00, // NOP
        ];

        for (offset, byte) in test_program.iter().enumerate() {
            emulator
                .bus
                .write_byte_no_tick(emulator.cpu.reg.pc + offset as u16, *byte);
        }

        emulator.step();

        assert_eq!(emulator.cpu.state, State::Halted);
        assert_eq!(emulator.cpu.reg.pc, 0xC001);

        emulator.step();

        assert_eq!(emulator.cpu.state, State::Halted);
        assert_eq!(emulator.cpu.reg.pc, 0xC001);

        // Request an interrupt
        emulator.bus.interrupts.timer.enabled = true;
        emulator.bus.interrupts.timer.requested = true;

        emulator.step();
        assert_eq!(emulator.cpu.reg.pc, 0xC003);
        assert!(emulator.bus.interrupts.timer.requested);
    }

    #[test]
    fn halt_bug() {
        // The notorious "HALT bug". If interrupts are disabled and there is a pending interrupt
        // when a HALT instruction is encountered, then the HALT state is skipped, and the PC fails
        // to increase for the next instruction.
        let mut emulator = Emulator::new();

        // Load a test program into RAM
        emulator.cpu.reg.pc = 0xC000;

        let test_program = [
            0xAF, // XOR A
            0x76, // HALT
            0x3C, // INC A    (this instruction will be executed twice)
            0x22, // LD (HL+),A
        ];

        emulator.cpu.reg.hl_mut().write(0xD000);
        emulator.bus.interrupts.enabled = false;
        emulator.bus.interrupts.timer.enabled = true;
        emulator.bus.interrupts.timer.requested = true;

        for (offset, byte) in test_program.iter().enumerate() {
            emulator
                .bus
                .write_byte(emulator.cpu.reg.pc + offset as u16, *byte);
        }

        for _ in 0..5 {
            emulator.step();
        }

        assert_eq!(emulator.cpu.reg.a, 2);
        assert_eq!(emulator.bus.read_byte(0xD000), 2);
    }
}
