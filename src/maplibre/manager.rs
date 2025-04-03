use crate::maplibre::bindings::*;
use crate::maplibre::helpers::*;
use crate::utils::log::{self, LogCategory, with_context};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::window;

// Type to manage the MapLibre map and its state
pub struct MapLibreManager {
    map: Option<Map>,
    // Add a field to store closures so they don't get dropped
    _event_listeners: Vec<Closure<dyn FnMut()>>,
}

impl MapLibreManager {
    // Create a new manager (without initializing the map yet)
    pub fn new() -> Self {
        log::info_with_category(LogCategory::Map, "MapLibreManager::new() called");
        Self {
            map: None,
            _event_listeners: Vec::new(),
        }
    }

    /// Create the actual map instance
    pub fn create_map(&mut self, container_id: &str) -> Result<(), JsValue> {
        with_context("MapLibreManager::create_map", LogCategory::Map, |logger| {
            logger.info(&format!("Creating map in container '{}'", container_id));

            // First check if maplibregl is loaded
            Self::debug_check_maplibregl()?;

            // Create map configuration
            let options = create_map_options(container_id)?;
            logger.debug("Map options created successfully");

            // Create the map
            logger.info("Creating new Map instance");
            let map = Map::new(&options);
            logger.debug("Map instance created successfully");

            // Store the map in our manager
            self.map = Some(map);
            logger.debug("Map stored in manager");

            // Store in window.mapInstance for compatibility with existing code
            if let Some(window) = window() {
                logger.debug("Setting window.mapInstance");
                js_sys::Reflect::set(
                    &window,
                    &JsValue::from_str("mapInstance"),
                    &JsValue::from(self.map.as_ref().unwrap()),
                )?;
                logger.debug("window.mapInstance set successfully");
            }

            Ok(())
        })
    }

    /// Add map controls (the buttons)
    pub fn add_map_controls(&mut self) -> Result<(), JsValue> {
        with_context(
            "MapLibreManager::add_map_controls",
            LogCategory::Map,
            |logger| {
                if let Some(map) = &self.map {
                    // Add navigation control
                    logger.debug("Adding NavigationControl");
                    let nav_control = NavigationControl::new();
                    map.addControl(&JsValue::from(nav_control), None);
                    logger.debug("NavigationControl added");

                    // Add scale control
                    logger.debug("Adding ScaleControl");
                    let scale_options = create_scale_control_options()?;
                    let scale_control = ScaleControl::new(&scale_options);
                    map.addControl(&JsValue::from(scale_control), Some("bottom-left"));
                    logger.debug("ScaleControl added");

                    // Add key control
                    logger.debug("Adding KeyControl");
                    let key_control = KeyControl::new();
                    map.addControl(&JsValue::from(key_control), Some("top-right"));
                    logger.debug("KeyControl added");

                    // Add layer switcher
                    logger.debug("Adding LayerSwitcher");
                    let layers = create_layer_groups()?;
                    let layer_switcher = LayerSwitcher::new(&layers, "TfL Layers");
                    map.addControl(&JsValue::from(layer_switcher), Some("top-right"));
                    logger.debug("LayerSwitcher added");

                    // Add simulation control
                    logger.debug("Adding SimulationControl");
                    let simulation_control = SimulationControl::new();
                    map.addControl(&JsValue::from(simulation_control), Some("top-right"));
                    logger.debug("SimulationControl added");

                    logger.info("All controls added successfully");
                } else {
                    logger.error("Cannot add controls: Map not initialized");
                    return Err(JsValue::from_str("Map not initialized"));
                }

                Ok(())
            },
        )
    }

    /// Set up map data sources and layers
    /// This is likely where the issue is happening with the 'load' event
    pub fn setup_map_data(&mut self, simulation_enabled: bool) -> Result<(), JsValue> {
        with_context(
            "MapLibreManager::setup_map_data",
            LogCategory::Map,
            |logger| {
                if let Some(map) = &self.map {
                    // Create a static listener ID to help with debugging
                    static mut LISTENER_ID: usize = 0;
                    let listener_id = unsafe {
                        LISTENER_ID += 1;
                        LISTENER_ID
                    };

                    logger.debug(&format!("Creating 'load' event listener #{}", listener_id));

                    // Pass simulation_enabled to closure
                    let simulation_enabled_copy = simulation_enabled;
                    // Set up an onload handler for the map - THIS IS LIKELY WHERE THE RECURSION HAPPENS
                    let load_handler = Closure::wrap(Box::new(move || {
                        log::info_with_category(
                            LogCategory::Map,
                            &format!("Map 'load' event fired (listener #{})", listener_id),
                        );

                        // This runs when the map style is fully loaded
                        let window = match window() {
                            Some(w) => w,
                            None => {
                                log::error_with_category(
                                    LogCategory::Map,
                                    "Window not available in load handler",
                                );
                                return;
                            }
                        };

                        log::debug_with_category(
                            LogCategory::Map,
                            "Getting mapInstance from window",
                        );
                        let map_instance = match js_sys::Reflect::get(
                            &window,
                            &JsValue::from_str("mapInstance"),
                        ) {
                            Ok(m) => {
                                if m.is_undefined() {
                                    log::error_with_category(
                                        LogCategory::Map,
                                        "mapInstance is undefined",
                                    );
                                    return;
                                }
                                m
                            }
                            Err(e) => {
                                log::error_with_category(
                                    LogCategory::Map,
                                    &format!("Failed to get mapInstance: {:?}", e),
                                );
                                return;
                            }
                        };

                        log::info_with_category(LogCategory::Map, "Adding map layers");
                        // Add our sources and layers
                        let result = add_map_layers(&map_instance, simulation_enabled_copy);

                        if let Err(err) = result {
                            log::error_with_category(
                                LogCategory::Map,
                                &format!("Error adding map layers: {:?}", err),
                            );
                        } else {
                            log::info_with_category(
                                LogCategory::Map,
                                "Map layers added successfully",
                            );
                        }
                    }) as Box<dyn FnMut()>);

                    // Add the load event handler
                    logger.debug("Registering 'load' event handler");
                    map.on("load", &load_handler);

                    // IMPORTANT: Store the handler to prevent it from being dropped
                    self._event_listeners.push(load_handler);
                    logger.debug("'load' event handler registered and stored");
                } else {
                    logger.error("Cannot setup map data: Map not initialized");
                    return Err(JsValue::from_str("Map not initialized"));
                }

                Ok(())
            },
        )
    }

    /// Update layer visibility based on TflLayers struct
    pub fn update_layer_visibility(&self, layers: &crate::app::TflLayers) -> Result<(), JsValue> {
        with_context(
            "MapLibreManager::update_layer_visibility",
            LogCategory::Map,
            |logger| {
                if let Some(map) = &self.map {
                    // Helper function to set visibility
                    let set_visibility = |layer_id: &str, visible: bool| -> Result<(), JsValue> {
                        logger.debug(&format!("Checking if layer '{}' exists", layer_id));
                        if map.get_layer(layer_id).is_some() {
                            logger.debug(&format!(
                                "Setting '{}' visibility to {}",
                                layer_id,
                                if visible { "visible" } else { "none" }
                            ));
                            let visibility = if visible { "visible" } else { "none" };
                            map.set_layout_property(
                                layer_id,
                                "visibility",
                                &JsValue::from_str(visibility),
                            );
                        } else {
                            logger.debug(&format!("Layer '{}' not found, skipping", layer_id));
                        }
                        Ok(())
                    };

                    // Update tube layers
                    set_visibility("central-line-layer", layers.tube)?;
                    set_visibility("northern-line-layer", layers.tube)?;

                    // Update overground layer
                    set_visibility("overground-line-layer", layers.overground)?;

                    // Update stations layers
                    set_visibility("stations-layer", layers.stations)?;
                    set_visibility("station-labels", layers.stations)?;
                } else {
                    logger.error("Cannot update layer visibility: Map not initialized");
                    return Err(JsValue::from_str("Map not initialized"));
                }

                Ok(())
            },
        )
    }

    // Static debug function to check if maplibregl is available
    pub fn debug_check_maplibregl() -> Result<(), JsValue> {
        with_context(
            "MapLibreManager::debug_check_maplibregl",
            LogCategory::Map,
            |logger| {
                logger.debug("Checking if maplibregl is loaded");

                let window = window().ok_or_else(|| JsValue::from_str("No window found"))?;

                // Check if maplibregl exists
                let maplibregl = js_sys::Reflect::get(&window, &JsValue::from_str("maplibregl"))?;

                if maplibregl.is_undefined() {
                    logger.error("maplibregl is undefined!");
                    return Err(JsValue::from_str("maplibregl is undefined"));
                }

                logger.debug("Found maplibregl object");

                // Check if Map constructor exists
                let map_constructor = js_sys::Reflect::get(&maplibregl, &JsValue::from_str("Map"))?;

                if map_constructor.is_undefined() {
                    logger.error("maplibregl.Map is undefined!");
                    return Err(JsValue::from_str("maplibregl.Map is undefined"));
                }

                logger.debug("Found maplibregl.Map constructor");

                // Check if it's actually a constructor/function
                if !JsValue::is_function(&map_constructor) {
                    logger.error("maplibregl.Map is not a function!");
                    return Err(JsValue::from_str("maplibregl.Map is not a function"));
                }

                logger.debug("maplibregl.Map is a function (constructor)");

                Ok(())
            },
        )
    }
}

/// Helper function to add MapLibre layers
fn add_map_layers(map_instance: &JsValue, simulation_enabled: bool) -> Result<(), JsValue> {
    with_context("add_map_layers", LogCategory::Map, |logger| {
        logger.debug("Creating map layers");

        let map: Map = map_instance.clone().into();
        logger.debug("Map instance cloned");

        // Central Line
        logger.debug("Adding Central Line");
        let central_coords = [
            (-0.22, 51.51),
            (-0.18, 51.52),
            (-0.14, 51.515),
            (-0.10, 51.52),
            (-0.05, 51.52),
        ];
        let central_source = create_geojson_line_source(&central_coords)?;
        map.add_source("central-line", &central_source);
        logger.debug("Central Line source added");

        let central_layer =
            create_line_layer("central-line-layer", "central-line", "#DC241F", 4.0)?;
        map.add_layer(&central_layer);
        logger.debug("Central Line layer added");

        // Northern Line
        logger.debug("Adding Northern Line");
        let northern_coords = [
            (-0.15, 51.48),
            (-0.12, 51.50),
            (-0.12, 51.53),
            (-0.14, 51.55),
        ];
        let northern_source = create_geojson_line_source(&northern_coords)?;
        map.add_source("northern-line", &northern_source);
        logger.debug("Northern Line source added");

        let northern_layer =
            create_line_layer("northern-line-layer", "northern-line", "#000000", 4.0)?;
        map.add_layer(&northern_layer);
        logger.debug("Northern Line layer added");

        // Overground
        logger.debug("Adding Overground");
        let overground_coords = [
            (-0.20, 51.53),
            (-0.16, 51.54),
            (-0.10, 51.54),
            (-0.05, 51.55),
        ];
        let overground_source = create_geojson_line_source(&overground_coords)?;
        map.add_source("overground-line", &overground_source);
        logger.debug("Overground source added");

        let overground_layer =
            create_line_layer("overground-line-layer", "overground-line", "#EE7C0E", 4.0)?;
        map.add_layer(&overground_layer);
        logger.debug("Overground layer added");

        // Stations
        logger.debug("Adding Stations");
        let mut stations = Vec::new();

        let mut oxford_circus = HashMap::new();
        oxford_circus.insert("name".to_string(), JsValue::from_str("Oxford Circus"));
        oxford_circus.insert("lng".to_string(), JsValue::from_f64(-0.1418));
        oxford_circus.insert("lat".to_string(), JsValue::from_f64(51.5152));
        stations.push(oxford_circus);

        let mut kings_cross = HashMap::new();
        kings_cross.insert("name".to_string(), JsValue::from_str("Kings Cross"));
        kings_cross.insert("lng".to_string(), JsValue::from_f64(-0.1234));
        kings_cross.insert("lat".to_string(), JsValue::from_f64(51.5308));
        stations.push(kings_cross);

        let mut liverpool_st = HashMap::new();
        liverpool_st.insert("name".to_string(), JsValue::from_str("Liverpool Street"));
        liverpool_st.insert("lng".to_string(), JsValue::from_f64(-0.0827));
        liverpool_st.insert("lat".to_string(), JsValue::from_f64(51.5178));
        stations.push(liverpool_st);

        let stations_source = create_geojson_points_source(&stations)?;
        map.add_source("stations", &stations_source);
        logger.debug("Stations source added");

        let stations_layer = create_circle_layer("stations-layer", "stations")?;
        map.add_layer(&stations_layer);
        logger.debug("Stations layer added");

        let labels_layer = create_label_layer("station-labels", "stations")?;
        map.add_layer(&labels_layer);
        logger.debug("Station labels layer added");

        logger.info("All map layers added successfully");

        if simulation_enabled {
            // Initialize the vehicle simulation after all other layers are added
            logger.info("Initializing vehicle simulation from map_layers");
            let init_simulation_js = r#"
                if (typeof window.initializeSimulation === 'function') {
                    console.log('Calling window.initializeSimulation()');
                    window.initializeSimulation();
                } else {
                    console.log('Creating initializeSimulation placeholder');
                    // Create a placeholder function that will be replaced when the simulation module loads
                    window.initializeSimulation = function() {
                        console.log('Placeholder initializeSimulation called - will retry in 1 second');
                        setTimeout(() => {
                            if (typeof window.realInitializeSimulation === 'function') {
                                window.realInitializeSimulation();
                            }
                        }, 1000);
                    };
                }
            "#;
            let _ = js_sys::eval(init_simulation_js);
            logger.debug("Vehicle simulation initialization requested");
        } else {
            logger.debug("Simulation disabled, skipping initialization");
        }

        Ok(())
    })
}

/// Implement Drop to clean up resources
impl Drop for MapLibreManager {
    fn drop(&mut self) {
        log::info_with_category(LogCategory::Map, "MapLibreManager being dropped");

        // Clear any global references
        if let Some(window) = window() {
            let _ =
                js_sys::Reflect::set(&window, &JsValue::from_str("mapInstance"), &JsValue::null());
        }

        log::debug_with_category(LogCategory::Map, "Global references cleared");
    }
}
