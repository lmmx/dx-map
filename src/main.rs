use dioxus::prelude::*;

// For logging and better errors in WASM
use log::Level;
use console_log;
use console_error_panic_hook;

mod app;

/// Entry point for Dioxus
fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(Level::Info).expect("error initializing logger");

    launch(app::app)
}
