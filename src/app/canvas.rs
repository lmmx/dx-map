use dioxus::prelude::*;
use wasm_bindgen::{JsValue, closure::Closure};
use web_sys::console;

use super::TflLayers;
use crate::maplibre::helpers::{
    add_inline_script,
    load_css,
    load_script,
};
use crate::maplibre::manager::MapLibreManager;

#[component]
pub fn Canvas(layers: Signal<TflLayers>) -> Element {
    // 1) A Dioxus state handle for your manager
    let mut manager = use_signal(|| MapLibreManager::new());

    // 2) Run this effect only once during mount
    // To avoid the infinite loop, we won't read and write to the same signal
    use_effect(move || {
        // Load any CSS or inline scripts - these run once since they're in a use_effect with no dependencies
        let _ = load_css("https://unpkg.com/maplibre-gl@3.6.2/dist/maplibre-gl.css");
        let _ = load_css("/assets/layerswitcher.css");
        let _ = add_inline_script(include_str!("../app/js/key_control.js"));
        let _ = add_inline_script(include_str!("../app/js/layer_switcher.js"));

        // Prepare the "on_load" closure for when the external script finishes
        let mut manager_clone = manager.clone(); // Create a clone to avoid capturing the original signal
        let on_load = Closure::wrap(Box::new(move || {
            console::log_1(&"MapLibre script loaded!".into());

            let mg = &mut manager_clone.write();
            if let Err(err) = mg.create_map("maplibre-canvas") {
                console::error_1(&format!("Failed to create map: {err:?}").into());
            }
            if let Err(err) = mg.add_map_controls() {
                console::error_1(&format!("Failed to add map controls: {err:?}").into());
            }
            if let Err(err) = mg.setup_map_data() {
                console::error_1(&format!("Failed to set up map data: {err:?}").into());
            }
        }) as Box<dyn FnMut()>);

        // Load the main MapLibre script and pass our closure
        let _ = load_script(
            "https://unpkg.com/maplibre-gl@3.6.2/dist/maplibre-gl.js",
            Some(on_load),
        );

        // Return an empty cleanup closure
        (|| {})()
    });

    // 3) Render the container in your JSX/RSX
    rsx! {
        div {
            id: "map-container",
            style: "position:relative; width:100%; height:100vh;",
            div {
                id: "maplibre-canvas",
                style: "position:absolute; top:0; bottom:0; width:100%;"
            }
        }
    }
}
