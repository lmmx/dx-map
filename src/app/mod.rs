//! # Application Module
//!
//! Main application components for the TfL Simulation.

use dioxus::prelude::*;

mod canvas;
mod key_panel;
mod layer_panel;
mod simulation; // New module for vehicle simulation

use crate::data::TflDataRepository;
use crate::maplibre::helpers;
use crate::utils::log::{self, LogCategory, with_context};
use canvas::Canvas;
use key_panel::KeyPanel;
use layer_panel::LayerPanel;
use wasm_bindgen::{JsValue, closure::Closure};
use web_sys::window;

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
    let layers = use_signal(TflLayers::default);
    let mut tfl_data = use_signal(TflDataRepository::default);

    use_future(move || async move {
        with_context("app::load_tfl_data", LogCategory::App, |logger| {
            logger.info("Loading TfL station and platform data");

            // Only load if not already loaded
            if !tfl_data.read().is_loaded {
                logger.info("Initializing TfL data repository");

                // Use spawn_local for the async operation, but don't use logger inside
                wasm_bindgen_futures::spawn_local(async move {
                    match TflDataRepository::initialize().await {
                        Ok(repository) => {
                            log::info_with_category(
                                LogCategory::App,
                                &format!(
                                    "TfL data loaded successfully with {} stations",
                                    repository.stations.len()
                                ),
                            );
                            tfl_data.set(repository);
                        }
                        Err(e) => {
                            log::error_with_category(
                                LogCategory::App,
                                &format!("Failed to load TfL data: {}", e),
                            );
                        }
                    }
                });
            } else {
                logger.info("TfL data already loaded, skipping");
            }
        });
    });

    // Add an effect to update the map when TFL data is loaded
    use_effect(move || {
        // Only react if the data is loaded
        if tfl_data.read().is_loaded {
            log::info_with_category(LogCategory::App, "TFL data loaded, updating map layers");

            // Update the map with the TFL data
            if let Some(manager) = window()
                .and_then(|w| js_sys::Reflect::get(&w, &JsValue::from_str("mapInstance")).ok())
            {
                let map: crate::maplibre::bindings::Map = manager.clone().into();

                // We need to check if the map style is loaded
                if map.is_style_loaded() {
                    log::info_with_category(
                        LogCategory::App,
                        "Map style loaded, adding TFL data layers",
                    );

                    // Call a helper function to add the TFL data to the map
                    add_tfl_data_to_map(&map, tfl_data.read().clone());
                } else {
                    log::info_with_category(
                        LogCategory::App,
                        "Map style not loaded yet, waiting for 'load' event",
                    );

                    // Create a callback for the 'load' event
                    let tfl_data_clone = tfl_data;
                    let load_callback = Closure::wrap(Box::new(move || {
                        log::info_with_category(
                            LogCategory::App,
                            "Map 'load' event fired, adding TFL data layers",
                        );

                        // If we get here via a callback, we need to get the map again
                        if let Some(window) = window() {
                            if let Ok(map_instance) =
                                js_sys::Reflect::get(&window, &JsValue::from_str("mapInstance"))
                            {
                                let map: crate::maplibre::bindings::Map = map_instance.into();
                                add_tfl_data_to_map(&map, tfl_data_clone.read().clone());
                            }
                        }
                    }) as Box<dyn FnMut()>);

                    // Register the callback
                    map.on("load", &load_callback);

                    // Leak the callback to keep it alive
                    load_callback.forget();
                }
            }
        }
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

/// Helper function to add TFL data layers to an already initialized map
fn add_tfl_data_to_map(map: &crate::maplibre::bindings::Map, tfl_data: TflDataRepository) {
    use crate::maplibre::helpers::{create_circle_layer, create_label_layer, create_line_layer};
    use crate::utils::log::{LogCategory, with_context};

    with_context("add_tfl_data_to_map", LogCategory::Map, |logger| {
        logger.info("Adding TFL data layers to map");

        // Add all stations as a GeoJSON source
        if let Ok(stations_geojson) = crate::data::stations_to_geojson(&tfl_data.stations) {
            // Make sure the source doesn't already exist
            if map.get_layer("tfl-stations-layer").is_none() {
                logger.info(&format!(
                    "Adding {} stations to map",
                    tfl_data.stations.len()
                ));

                // Add the source
                web_sys::console::log_1(&stations_geojson);
                map.add_source("tfl-stations", &stations_geojson);

                // Add a circle layer for the stations
                if let Ok(stations_layer) =
                    create_circle_layer("tfl-stations-layer", "tfl-stations")
                {
                    map.add_layer(&stations_layer);
                    logger.debug("Added stations layer");
                }

                // Add a label layer for the stations
                if let Ok(labels_layer) = create_label_layer("tfl-station-labels", "tfl-stations") {
                    map.add_layer(&labels_layer);
                    logger.debug("Added station labels layer");
                }
            } else {
                logger.debug("Stations layer already exists, skipping");
            }
        } else {
            logger.error("Failed to convert stations to GeoJSON");
        }

        // Add all tube lines
        if let Ok(line_data) = crate::data::generate_all_line_data(&tfl_data) {
            logger.info(&format!("Adding {} TFL lines to map", line_data.len()));

            for (line_name, line_geojson, color) in line_data {
                web_sys::console::log_1(&line_geojson);
                let source_id = format!("{}-line", line_name);
                let layer_id = format!("{}-line-layer", line_name);

                // Make sure the layer doesn't already exist
                if map.get_layer(&layer_id).is_none() {
                    // Add the source
                    map.add_source(&source_id, &line_geojson);

                    // Add the layer
                    if let Ok(line_layer) = create_line_layer(&layer_id, &source_id, &color, 4.0) {
                        map.add_layer(&line_layer);
                        logger.debug(&format!("Added {} line", line_name));
                    }
                } else {
                    logger.debug(&format!("{} line already exists, skipping", line_name));
                }
            }
        } else {
            logger.error("Failed to generate line data");
        }

        // Add route geometries from our new data
        if let Ok(route_data) = crate::data::generate_all_route_geometries(&tfl_data) {
            logger.info(&format!("Adding {} TFL route geometries to map", route_data.len()));

            for (line_id, route_geojson) in route_data {
                logger.debug(&format!("Adding {} route geometry", line_id));
                web_sys::console::log_1(&route_geojson);
                let source_id = format!("{}-route", line_id);
                let layer_id = format!("{}-route-layer", line_id);

                // Make sure the layer doesn't already exist
                if map.get_layer(&layer_id).is_none() {
                    // Add the source
                    map.add_source(&source_id, &route_geojson);

                    // Get the appropriate color for this line
                    let color = get_line_color(&line_id);

                    // Add the layer with a dashed style to distinguish from simplified line data
                    if let Ok(route_layer) = create_line_layer(&layer_id, &source_id, &color, 3.0) {
                        map.add_layer(&route_layer);
                        logger.debug(&format!("Added {} route geometry", line_id));
                    }
                } else {
                    logger.debug(&format!("{} route layer already exists, skipping", line_id));
                }
            }
        } else {
            logger.error("Failed to generate route geometries");
        }

        logger.info("TFL data layers added to map");
    });
}

/// Helper function to get the color for a specific TfL line
fn get_line_color(line_id: &str) -> String {
    match line_id {
        "bakerloo" => "#B36305",
        "central" => "#E32017",
        "circle" => "#FFD300",
        "district" => "#00782A",
        "hammersmith-city" => "#F3A9BB",
        "jubilee" => "#A0A5A9",
        "metropolitan" => "#9B0056",
        "northern" => "#000000",
        "piccadilly" => "#003688",
        "victoria" => "#0098D4",
        "waterloo-city" => "#95CDBA",
        "dlr" => "#00A4A7",
        "london-overground" => "#EE7C0E",
        "elizabeth" => "#6950A1",
        "tram" => "#84B817",
        "cable-car" => "#E21836",
        _ => "#777777",  // Default gray for unknown lines
    }.to_string()
}
