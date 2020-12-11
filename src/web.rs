//! Web-specific bindings and functions.

use log::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::Emulator;

fn handle_error(e: anyhow::Error) {
    error!("fatal error: {}", e);

    let document = web_sys::window()
        .and_then(|window| window.document())
        .unwrap();
    document
        .get_element_by_id("error-text")
        .unwrap()
        .set_inner_html(&e.to_string());
    document
        .get_element_by_id("modal")
        .and_then(|element| element.dyn_into::<HtmlElement>().ok())
        .unwrap()
        .style()
        .set_property("display", "block")
        .unwrap();
}

#[wasm_bindgen]
pub fn run_emulator(rom: &[u8]) {
    let mut emulator = Emulator::new();

    if let Err(e) = emulator.load_rom(rom) {
        handle_error(e);
        return;
    }

    wasm_bindgen_futures::spawn_local(async move {
        if let Err(e) = emulator.run().await {
            handle_error(e);
        }
    });
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init().unwrap();
}
