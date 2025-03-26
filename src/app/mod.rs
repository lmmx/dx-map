use dioxus::prelude::*;

mod canvas;
mod layer_panel;
mod key_panel;
mod simulation; // New module for vehicle simulation

use canvas::Canvas;
use layer_panel::LayerPanel;
use key_panel::KeyPanel;
use simulation::VehicleSimulation; // Import the simulation component

// If you have images or CSS as assets, define them with Dioxus' asset! macro
const FAVICON: Asset = asset!("/assets/favicon.ico");
const LOGO_SVG: Asset = asset!("/assets/header.svg");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TFL_CSS: Asset = asset!("/assets/tfl.css");
const KEY_CSS: Asset = asset!("/assets/key.css");

// Model to track layer visibility
#[derive(Clone, Copy, PartialEq)]
pub struct TflLayers {
    pub tube: bool,
    pub overground: bool,
    pub dlr: bool,
    pub elizabeth_line: bool,
    pub buses: bool,
    pub trams: bool,
    pub cable_car: bool,
    pub stations: bool,
    pub depots: bool,
    pub simulation: bool, // New field for simulation visibility
}

impl Default for TflLayers {
    fn default() -> Self {
        Self {
            tube: true,
            overground: true,
            dlr: true,
            elizabeth_line: true,
            buses: false,
            trams: false,
            cable_car: true,
            stations: true,
            depots: false,
            simulation: true, // Show simulation by default
        }
    }
}

#[component]
pub fn app() -> Element {
    let mut show_layers_panel = use_signal(|| false);
    let mut show_key_panel = use_signal(|| false);
    let mut show_simulation_panel = use_signal(|| false); // New signal for simulation controls
    let layers = use_signal(|| TflLayers::default());

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TFL_CSS }
        document::Link { rel: "stylesheet", href: KEY_CSS }

        header {
            img { src: LOGO_SVG }
            p { "Real-time TfL network simulation" }
            
            nav {
                ul {
                    li { a { href: "#", "About" } }
                    li { a { href: "#", "Stats" } }
                    li { a { href: "#", "Exports" } }
                }
            }
        }

        main {
            class: "app-content",
            
            // Main map container
            Canvas { layers: layers }
            
            // Controls for showing/hiding panels
            div {
                class: "maplibregl-ctrl maplibregl-ctrl-group layer-controls",
                
                // Layers button
                button {
                    class: "maplibregl-ctrl-layers",
                    title: "Show/hide layers",
                    onclick: move |_| {
                        let current = *show_layers_panel.read();
                        show_layers_panel.set(!current);
                    },
                    "☰"
                }
                
                // Key button
                button {
                    class: "maplibregl-ctrl-key",
                    title: "Show map key",
                    onclick: move |_| {
                        let current = *show_key_panel.read();
                        show_key_panel.set(!current);
                    },
                    "ⓘ"
                }
                
                // Simulation button
                button {
                    class: "maplibregl-ctrl-simulation",
                    title: "Simulation controls",
                    onclick: move |_| {
                        let current = *show_simulation_panel.read();
                        show_simulation_panel.set(!current);
                    },
                    "▶"
                }
            }
            
            // Layer panel component - conditionally shown
            LayerPanel {
                visible: *show_layers_panel.read(),
                layers: layers,
                on_close: move |_| show_layers_panel.set(false)
            }
            
            // Key panel component - conditionally shown
            KeyPanel {
                visible: *show_key_panel.read(),
                on_close: move |_| show_key_panel.set(false)
            }
            
            // Simulation panel - conditionally shown
            if *show_simulation_panel.read() {
                div {
                    class: "simulation-panel",
                    VehicleSimulation {}
                    
                    button {
                        class: "close-button",
                        onclick: move |_| show_simulation_panel.set(false),
                        "Close"
                    }
                }
            }
        }
    }
}
