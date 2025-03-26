use dioxus::prelude::*;

// For logging and better errors in WASM
use console_error_panic_hook;
use console_log;
use log::Level;

mod app;

/// Entry point for Dioxus
fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(Level::Info).expect("error initializing logger");

    launch(app::app)
}
