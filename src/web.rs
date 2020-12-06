//! Web-specific bindings and functions.

use log::*;
use wasm_bindgen::prelude::*;

use crate::Emulator;

#[wasm_bindgen]
pub fn run_emulator(rom: &[u8]) {
    let mut emulator = Emulator::new();

    emulator.load_rom(rom).unwrap();

    wasm_bindgen_futures::spawn_local(async move {
        if let Err(e) = emulator.run().await {
            error!("fatal error: {}", e);
        }
    });
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init().unwrap();
}
