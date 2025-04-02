use crate::maplibre::bindings::*;
use crate::maplibre::helpers::*;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use web_sys::{console, window};

// Type to manage the MapLibre map and its state
pub struct MapLibreManager {
    map: Option<Map>,
    // Add a field to store closures so they don't get dropped
    _event_listeners: Vec<Closure<dyn FnMut()>>,
}

impl MapLibreManager {
    // Create a new manager (without initializing the map yet)
    pub fn new() -> Self {
        console::log_1(&"MapLibreManager::new() called".into());
        Self {
            map: None,
            _event_listeners: Vec::new(),
        }
    }

    /// Create the actual map instance
    pub fn create_map(&mut self, container_id: &str) -> Result<(), JsValue> {
        console::log_1(&format!("MapLibreManager::create_map('{}') called", container_id).into());

        // First check if maplibregl is loaded
        Self::debug_check_maplibregl()?;

        // Create map configuration
        let options = create_map_options(container_id)?;
        console::log_1(&"Map options created successfully".into());

        // Create the map
        console::log_1(&"Creating new Map instance".into());
        let map = Map::new(&options);
        console::log_1(&"Map instance created successfully".into());

        // Store the map in our manager
        self.map = Some(map);
        console::log_1(&"Map stored in manager".into());

        // Store in window.mapInstance for compatibility with existing code
        if let Some(window) = window() {
            console::log_1(&"Setting window.mapInstance".into());
            js_sys::Reflect::set(
                &window,
                &JsValue::from_str("mapInstance"),
                &JsValue::from(self.map.as_ref().unwrap()),
            )?;
            console::log_1(&"window.mapInstance set successfully".into());
        }

        Ok(())
    }

    /// Add map controls (the buttons)
    pub fn add_map_controls(&mut self) -> Result<(), JsValue> {
        console::log_1(&"MapLibreManager::add_map_controls() called".into());

        if let Some(map) = &self.map {
            // Add navigation control
            console::log_1(&"Adding NavigationControl".into());
            let nav_control = NavigationControl::new();
            map.addControl(&JsValue::from(nav_control), None);
            console::log_1(&"NavigationControl added".into());

            // Add scale control
            console::log_1(&"Adding ScaleControl".into());
            let scale_options = create_scale_control_options()?;
            let scale_control = ScaleControl::new(&scale_options);
            map.addControl(&JsValue::from(scale_control), Some("bottom-left"));
            console::log_1(&"ScaleControl added".into());

            // Add key control
            console::log_1(&"Adding KeyControl".into());
            let key_control = KeyControl::new();
            map.addControl(&JsValue::from(key_control), Some("top-right"));
            console::log_1(&"KeyControl added".into());

            // Add layer switcher
            console::log_1(&"Adding LayerSwitcher".into());
            let layers = create_layer_groups()?;
            let layer_switcher = LayerSwitcher::new(&layers, "TfL Layers");
            map.addControl(&JsValue::from(layer_switcher), Some("top-right"));
            console::log_1(&"LayerSwitcher added".into());

            // Add simulation control
            console::log_1(&"Adding SimulationControl".into());
            let simulation_control = SimulationControl::new();
            map.addControl(&JsValue::from(simulation_control), Some("top-right"));
            console::log_1(&"SimulationControl added".into());

            console::log_1(&"All controls added successfully".into());
        } else {
            console::error_1(&"Cannot add controls: Map not initialized".into());
            return Err(JsValue::from_str("Map not initialized"));
        }

        Ok(())
    }

    /// Set up map data sources and layers
    /// This is likely where the issue is happening with the 'load' event
    pub fn setup_map_data(&mut self, simulation_enabled: bool) -> Result<(), JsValue> {
        console::log_1(&"MapLibreManager::setup_map_data() called".into());

        if let Some(map) = &self.map {
            // Create a static listener ID to help with debugging
            static mut LISTENER_ID: usize = 0;
            let listener_id = unsafe {
                LISTENER_ID += 1;
                LISTENER_ID
            };

            console::log_1(&format!("Creating 'load' event listener #{}", listener_id).into());

            // Pass simulation_enabled to closure
            let simulation_enabled_copy = simulation_enabled;
            // Set up an onload handler for the map - THIS IS LIKELY WHERE THE RECURSION HAPPENS
            let load_handler = Closure::wrap(Box::new(move || {
                console::log_1(
                    &format!("Map 'load' event fired (listener #{})", listener_id).into(),
                );

                // This runs when the map style is fully loaded
                let window = match window() {
                    Some(w) => w,
                    None => {
                        console::error_1(&"Window not available in load handler".into());
                        return;
                    }
                };

                console::log_1(&"Getting mapInstance from window".into());
                let map_instance =
                    match js_sys::Reflect::get(&window, &JsValue::from_str("mapInstance")) {
                        Ok(m) => {
                            if m.is_undefined() {
                                console::error_1(&"mapInstance is undefined".into());
                                return;
                            }
                            m
                        }
                        Err(e) => {
                            console::error_1(&format!("Failed to get mapInstance: {:?}", e).into());
                            return;
                        }
                    };

                console::log_1(&"Adding map layers".into());
                // Add our sources and layers
                let result = add_map_layers(&map_instance, simulation_enabled_copy);

                if let Err(err) = result {
                    console::error_1(&format!("Error adding map layers: {:?}", err).into());
                } else {
                    console::log_1(&"Map layers added successfully".into());
                }
            }) as Box<dyn FnMut()>);

            // Add the load event handler
            console::log_1(&"Registering 'load' event handler".into());
            map.on("load", &load_handler);

            // IMPORTANT: Store the handler to prevent it from being dropped
            self._event_listeners.push(load_handler);
            console::log_1(&"'load' event handler registered and stored".into());
        } else {
            console::error_1(&"Cannot setup map data: Map not initialized".into());
            return Err(JsValue::from_str("Map not initialized"));
        }

        Ok(())
    }

    /// Update layer visibility based on TflLayers struct
    pub fn update_layer_visibility(&self, layers: &crate::app::TflLayers) -> Result<(), JsValue> {
        console::log_1(&"MapLibreManager::update_layer_visibility() called".into());

        if let Some(map) = &self.map {
            // Helper function to set visibility
            let set_visibility = |layer_id: &str, visible: bool| -> Result<(), JsValue> {
                console::log_1(&format!("Checking if layer '{}' exists", layer_id).into());
                if map.get_layer(layer_id).is_some() {
                    console::log_1(
                        &format!(
                            "Setting '{}' visibility to {}",
                            layer_id,
                            if visible { "visible" } else { "none" }
                        )
                        .into(),
                    );
                    let visibility = if visible { "visible" } else { "none" };
                    map.set_layout_property(layer_id, "visibility", &JsValue::from_str(visibility));
                } else {
                    console::log_1(&format!("Layer '{}' not found, skipping", layer_id).into());
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
            console::error_1(&"Cannot update layer visibility: Map not initialized".into());
            return Err(JsValue::from_str("Map not initialized"));
        }

        Ok(())
    }

    // Static debug function to check if maplibregl is available
    pub fn debug_check_maplibregl() -> Result<(), JsValue> {
        console::log_1(&"Checking if maplibregl is loaded".into());

        let window = window().ok_or_else(|| JsValue::from_str("No window found"))?;

        // Check if maplibregl exists
        let maplibregl = js_sys::Reflect::get(&window, &JsValue::from_str("maplibregl"))?;

        if maplibregl.is_undefined() {
            console::error_1(&"maplibregl is undefined!".into());
            return Err(JsValue::from_str("maplibregl is undefined"));
        }

        console::log_1(&"Found maplibregl object".into());

        // Check if Map constructor exists
        let map_constructor = js_sys::Reflect::get(&maplibregl, &JsValue::from_str("Map"))?;

        if map_constructor.is_undefined() {
            console::error_1(&"maplibregl.Map is undefined!".into());
            return Err(JsValue::from_str("maplibregl.Map is undefined"));
        }

        console::log_1(&"Found maplibregl.Map constructor".into());

        // Check if it's actually a constructor/function
        if !JsValue::is_function(&map_constructor) {
            console::error_1(&"maplibregl.Map is not a function!".into());
            return Err(JsValue::from_str("maplibregl.Map is not a function"));
        }

        console::log_1(&"maplibregl.Map is a function (constructor)".into());

        Ok(())
    }
}

/// Helper function to add MapLibre layers
fn add_map_layers(map_instance: &JsValue, simulation_enabled: bool) -> Result<(), JsValue> {
    console::log_1(&"add_map_layers() called".into());

    let map: Map = map_instance.clone().into();
    console::log_1(&"Map instance cloned".into());

    // Central Line
    console::log_1(&"Adding Central Line".into());
    let central_coords = [
        (-0.22, 51.51),
        (-0.18, 51.52),
        (-0.14, 51.515),
        (-0.10, 51.52),
        (-0.05, 51.52),
    ];
    let central_source = create_geojson_line_source(&central_coords)?;
    map.add_source("central-line", &central_source);
    console::log_1(&"Central Line source added".into());

    let central_layer = create_line_layer("central-line-layer", "central-line", "#DC241F", 4.0)?;
    map.add_layer(&central_layer);
    console::log_1(&"Central Line layer added".into());

    // Northern Line
    console::log_1(&"Adding Northern Line".into());
    let northern_coords = [
        (-0.15, 51.48),
        (-0.12, 51.50),
        (-0.12, 51.53),
        (-0.14, 51.55),
    ];
    let northern_source = create_geojson_line_source(&northern_coords)?;
    map.add_source("northern-line", &northern_source);
    console::log_1(&"Northern Line source added".into());

    let northern_layer = create_line_layer("northern-line-layer", "northern-line", "#000000", 4.0)?;
    map.add_layer(&northern_layer);
    console::log_1(&"Northern Line layer added".into());

    // Overground
    console::log_1(&"Adding Overground".into());
    let overground_coords = [
        (-0.20, 51.53),
        (-0.16, 51.54),
        (-0.10, 51.54),
        (-0.05, 51.55),
    ];
    let overground_source = create_geojson_line_source(&overground_coords)?;
    map.add_source("overground-line", &overground_source);
    console::log_1(&"Overground source added".into());

    let overground_layer =
        create_line_layer("overground-line-layer", "overground-line", "#EE7C0E", 4.0)?;
    map.add_layer(&overground_layer);
    console::log_1(&"Overground layer added".into());

    // Stations
    console::log_1(&"Adding Stations".into());
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
    console::log_1(&"Stations source added".into());

    let stations_layer = create_circle_layer("stations-layer", "stations")?;
    map.add_layer(&stations_layer);
    console::log_1(&"Stations layer added".into());

    let labels_layer = create_label_layer("station-labels", "stations")?;
    map.add_layer(&labels_layer);
    console::log_1(&"Station labels layer added".into());

    console::log_1(&"All map layers added successfully".into());

    if simulation_enabled {
        // Initialize the vehicle simulation after all other layers are added
        console::log_1(&"Initializing vehicle simulation from map_layers".into());
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
        console::log_1(&"Vehicle simulation initialization requested".into());
    } else {
        console::log_1(&"Simulation disabled, skipping initialization".into());
    }

    Ok(())
}

/// Implement Drop to clean up resources
impl Drop for MapLibreManager {
    fn drop(&mut self) {
        console::log_1(&"MapLibreManager being dropped".into());

        // Clear any global references
        if let Some(window) = window() {
            let _ =
                js_sys::Reflect::set(&window, &JsValue::from_str("mapInstance"), &JsValue::null());
        }

        console::log_1(&"Global references cleared".into());
    }
}
