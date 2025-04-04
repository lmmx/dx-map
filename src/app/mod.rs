//! # Application Module
//!
//! Main application components for the TfL Simulation.

use dioxus::prelude::*;

mod canvas;
mod key_panel;
mod layer_panel;
mod simulation; // New module for vehicle simulation

use crate::maplibre::helpers;
use crate::utils::log::{self, LogCategory, with_context};
use crate::data::TflDataRepository;
use canvas::Canvas;
use key_panel::KeyPanel;
use layer_panel::LayerPanel;

// If you have images or CSS as assets, define them with Dioxus' asset! macro
const FAVICON: Asset = asset!("/assets/favicon.ico");
const LOGO_SVG: Asset = asset!("/assets/header.svg");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TFL_CSS: Asset = asset!("/assets/tfl.css");
const KEY_CSS: Asset = asset!("/assets/key.css");
const LAYER_CSS: Asset = asset!("/assets/layerswitcher.css");

/// Model to track layer visibility.
///
/// This structure tracks which layers are visible in the TfL network map.
#[derive(Clone, Copy, PartialEq)]
pub struct TflLayers {
    /// Underground/tube lines
    pub tube: bool,
    /// Overground rail services
    pub overground: bool,
    /// Docklands Light Railway
    pub dlr: bool,
    /// Elizabeth Line (Crossrail)
    pub elizabeth_line: bool,
    /// Bus routes
    pub buses: bool,
    /// Tram services
    pub trams: bool,
    /// Emirates Air Line cable car
    pub cable_car: bool,
    /// Station markers
    pub stations: bool,
    /// Depot locations
    pub depots: bool,
    /// Vehicle simulation
    pub simulation: bool,
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
            simulation: false, // Simulation disabled by default
        }
    }
}

/// Main application component.
///
/// This is the root component of the TfL Simulation application.
#[component]
pub fn app() -> Element {
    let mut show_layers_panel = use_signal(|| false);
    let mut show_key_panel = use_signal(|| false);
    let mut show_simulation_panel = use_signal(|| false); // New signal for simulation controls
    let layers = use_signal(|| TflLayers::default());
    let mut tfl_data = use_signal(|| TflDataRepository::default());

    use_future(move || async move {
        with_context("app::load_tfl_data", LogCategory::App, |logger| {
            logger.info("Loading TfL station and platform data");
            
            // Only load if not already loaded
            if !tfl_data.read().is_loaded {
                logger.info("Initializing TfL data repository");

                // Clone the signal to move into the async task
                let tfl_data_clone = tfl_data.clone();
                
                // Use spawn_local for the async operation, but don't use logger inside
                wasm_bindgen_futures::spawn_local(async move {
                    match TflDataRepository::initialize().await {
                        Ok(repository) => {
                            log::info_with_category(
                                LogCategory::App,
                                &format!("TfL data loaded successfully with {} stations", repository.stations.len())
                            );
                            tfl_data.set(repository);
                        }
                        Err(e) => {
                            log::error_with_category(
                                LogCategory::App,
                                &format!("Failed to load TfL data: {}", e)
                            );
                        }
                    }
                });
            } else {
                logger.info("TfL data already loaded, skipping");
            }
        });
    });

    // Initialize simulation JS when app loads
    use_effect(move || {
        with_context("app::simulation_init", LogCategory::App, |logger| {
            logger.info("Initializing simulation controller script");

            let controller_script = format!(
                r#"
    // Global simulation controller
    const SimulationController = {{
      initialized: false,
      running: false,

      initialize: function() {{
        console.log("SimulationController.initialize() called");
        if (this.initialized) {{
          console.log("Simulation already initialized, skipping");
          return;
        }}

        // Call the Rust initialization function
        if (typeof window.rust_initialize_simulation === 'function') {{
          console.log("Calling rust_initialize_simulation()");
          window.rust_initialize_simulation();
          this.initialized = true;
          this.running = true;
        }} else {{
          console.error("rust_initialize_simulation function not found");
        }}
      }},

      toggle: function() {{
        console.log("SimulationController.toggle() called");
        if (!this.initialized) {{
          this.initialize();
          return;
        }}

        if (typeof window.rust_toggle_simulation === 'function') {{
          window.rust_toggle_simulation();
          this.running = !this.running;
          console.log("Simulation running:", this.running);
        }}
      }},

      reset: function() {{
        console.log("SimulationController.reset() called");
        if (typeof window.rust_reset_simulation === 'function') {{
          window.rust_reset_simulation();
          this.running = true;
          console.log("Simulation reset and running");
        }}
      }}
    }};

    // Make it globally available
    window.SimulationController = SimulationController;

    // Only initialize automatically if simulation is enabled
    const simulationEnabled = {0};

    if (simulationEnabled) {{
      // Initialize when map is ready
      if (window.mapInstance && window.mapInstance.isStyleLoaded()) {{
        setTimeout(function() {{
          SimulationController.initialize();
        }}, 1000);
      }} else {{
        const initInterval = setInterval(function() {{
          if (window.mapInstance && window.mapInstance.isStyleLoaded()) {{
            clearInterval(initInterval);
            setTimeout(function() {{
              SimulationController.initialize();
            }}, 1000);
          }}
        }}, 1000);
      }}
    }} else {{
      console.log("Automatic simulation initialization disabled");
    }}
    "#,
                layers.read().simulation
            );

            if let Err(e) = helpers::add_inline_script(&controller_script) {
                logger.error(&format!("Failed to add simulation script: {:?}", e));
            } else {
                logger.info("Simulation controller script added successfully");
            }
        })
    });

    use_effect(move || {
        with_context("app::simulation_functions", LogCategory::App, |logger| {
            logger.info("Exposing simulation functions");

            // Try to expose simulation functions if available
            match simulation::expose_simulation_functions() {
                Ok(_) => {
                    logger.info("Simulation functions exposed successfully");
                }
                Err(err) => {
                    logger.error(&format!("Failed to expose simulation functions: {:?}", err));
                }
            }

            // Add the controller script
            let controller_script = r#"
    // SimulationController code here...
    "#;

            if let Err(e) = helpers::add_inline_script(controller_script) {
                logger.error(&format!("Failed to add simulation script: {:?}", e));
            } else {
                logger.debug("Additional controller script added");
            }
        })
    });

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TFL_CSS }
        document::Link { rel: "stylesheet", href: KEY_CSS }
        document::Link { rel: "stylesheet", href: LAYER_CSS }

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
            Canvas { layers: layers, tfl_data: tfl_data }

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
        }
    }
}
