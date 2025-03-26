use crate::maplibre::bindings::*;
use crate::maplibre::helpers::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{console, window};

// Type to manage the MapLibre map and its state
pub struct MapLibreManager {
    map: Option<Map>,
}

impl MapLibreManager {
    // Create a new manager (without initializing the map yet)
    pub fn new() -> Self {
        Self { map: None }
    }

    /// Create the actual map instance
    pub fn create_map(&mut self, container_id: &str) -> Result<(), JsValue> {
        // Create map configuration
        let options = create_map_options(container_id)?;

        // Create the map
        let map = Map::new(&options);

        // Store the map in our manager
        self.map = Some(map);

        // Store in window.mapInstance for compatibility with existing code
        if let Some(window) = window() {
            js_sys::Reflect::set(
                &window,
                &JsValue::from_str("mapInstance"),
                &JsValue::from(self.map.as_ref().unwrap()),
            )?;
        }

        Ok(())
    }

    /// Add map controls (the buttons)
    pub fn add_map_controls(&mut self) -> Result<(), JsValue> {
        if let Some(map) = &self.map {
            // Add navigation control
            let nav_control = NavigationControl::new();
            map.addControl(&JsValue::from(nav_control), None);

            // Add scale control
            let scale_options = create_scale_control_options()?;
            let scale_control = ScaleControl::new(&scale_options);
            map.addControl(&JsValue::from(scale_control), Some("bottom-left"));

            // Add key control
            let key_control = KeyControl::new();
            map.addControl(&JsValue::from(key_control), Some("top-right"));

            // Add layer switcher
            let layers = create_layer_groups()?;
            let layer_switcher = LayerSwitcher::new(&layers, "TfL Layers");
            map.addControl(&JsValue::from(layer_switcher), Some("top-right"));
        }

        Ok(())
    }

    /// Set up map data sources and layers
    pub fn setup_map_data(&mut self) -> Result<(), JsValue> {
        if let Some(map) = &self.map {
            // Set up an onload handler for the map
            let closure = Closure::wrap(Box::new(move || {
                // This runs when the map style is fully loaded
                let window = window().unwrap();
                let map_instance =
                    js_sys::Reflect::get(&window, &JsValue::from_str("mapInstance")).unwrap();

                if map_instance.is_undefined() {
                    return;
                }

                // Add our sources and layers
                let result = add_map_layers(&map_instance);

                if let Err(err) = result {
                    console::error_1(&format!("Error adding map layers: {:?}", err).into());
                }
            }) as Box<dyn FnMut()>);

            // Add the load event handler
            map.on("load", &closure);
        }

        Ok(())
    }

    /// Update layer visibility based on TflLayers struct
    pub fn update_layer_visibility(&self, layers: &crate::app::TflLayers) -> Result<(), JsValue> {
        if let Some(map) = &self.map {
            // Helper function to set visibility
            let set_visibility = |layer_id: &str, visible: bool| -> Result<(), JsValue> {
                if map.get_layer(layer_id).is_some() {
                    let visibility = if visible { "visible" } else { "none" };
                    map.set_layout_property(layer_id, "visibility", &JsValue::from_str(visibility));
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
        }

        Ok(())
    }
}

/// Helper function to add MapLibre layers
fn add_map_layers(map_instance: &JsValue) -> Result<(), JsValue> {
    let map: Map = map_instance.clone().into();

    // Central Line
    let central_coords = [
        (-0.22, 51.51),
        (-0.18, 51.52),
        (-0.14, 51.515),
        (-0.10, 51.52),
        (-0.05, 51.52),
    ];
    let central_source = create_geojson_line_source(&central_coords)?;
    map.add_source("central-line", &central_source);

    let central_layer = create_line_layer("central-line-layer", "central-line", "#DC241F", 4.0)?;
    map.add_layer(&central_layer);

    // Northern Line
    let northern_coords = [
        (-0.15, 51.48),
        (-0.12, 51.50),
        (-0.12, 51.53),
        (-0.14, 51.55),
    ];
    let northern_source = create_geojson_line_source(&northern_coords)?;
    map.add_source("northern-line", &northern_source);

    let northern_layer = create_line_layer("northern-line-layer", "northern-line", "#000000", 4.0)?;
    map.add_layer(&northern_layer);

    // Overground
    let overground_coords = [
        (-0.20, 51.53),
        (-0.16, 51.54),
        (-0.10, 51.54),
        (-0.05, 51.55),
    ];
    let overground_source = create_geojson_line_source(&overground_coords)?;
    map.add_source("overground-line", &overground_source);

    let overground_layer =
        create_line_layer("overground-line-layer", "overground-line", "#EE7C0E", 4.0)?;
    map.add_layer(&overground_layer);

    // Stations
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

    let stations_layer = create_circle_layer("stations-layer", "stations")?;
    map.add_layer(&stations_layer);

    let labels_layer = create_label_layer("station-labels", "stations")?;
    map.add_layer(&labels_layer);

    Ok(())
}

/// Implement Drop to clean up resources
impl Drop for MapLibreManager {
    fn drop(&mut self) {
        // Clear any global references
        if let Some(window) = window() {
            let _ =
                js_sys::Reflect::set(&window, &JsValue::from_str("mapInstance"), &JsValue::null());
        }
    }
}
