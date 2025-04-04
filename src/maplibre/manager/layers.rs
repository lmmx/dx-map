// Layer management for map
use crate::maplibre::bindings::Map;
use crate::maplibre::helpers::{create_circle_layer, create_line_layer, create_label_layer, create_geojson_points_source, create_geojson_line_source};
use crate::utils::log::{self, LogCategory, with_context};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// Manager for map layers
pub struct LayerManager {
    registered_layers: HashMap<String, LayerInfo>,
}

/// Information about a registered layer
struct LayerInfo {
    id: String,
    type_: String,
    source: String,
    visible: bool,
}

impl LayerManager {
    pub fn new() -> Self {
        Self {
            registered_layers: HashMap::new(),
        }
    }

    /// Add a layer to the map
    pub fn add_layer(
        &mut self,
        map: &Map,
        layer_id: &str,
        source: &str,
        type_: &str,
        style: impl Into<JsValue>,
    ) -> Result<(), JsValue> {
        with_context("LayerManager::add_layer", LogCategory::Map, |logger| {
            logger.debug(&format!(
                "Adding layer '{}' with source '{}'",
                layer_id, source
            ));

            // Add layer to map
            let layer = match type_ {
                "line" => create_line_layer(layer_id, source, "#FFFFFF", 1.0)?, // Default values
                "circle" => create_circle_layer(layer_id, source)?,
                "label" => create_label_layer(layer_id, source)?,
                _ => return Err(JsValue::from_str("Unsupported layer type")),
            };

            // Apply additional style if provided
            if !style.into().is_undefined() {
                // TODO: Apply additional style properties
            }

            map.add_layer(&layer);

            // Register the layer
            self.registered_layers.insert(
                layer_id.to_string(),
                LayerInfo {
                    id: layer_id.to_string(),
                    type_: type_.to_string(),
                    source: source.to_string(),
                    visible: true, // Default to visible
                },
            );

            logger.debug(&format!("Layer '{}' added and registered", layer_id));

            Ok(())
        })
    }

    /// Set the visibility of a layer
    pub fn set_visibility(
        &mut self,
        map: &Map,
        layer_id: &str,
        visible: bool,
    ) -> Result<(), JsValue> {
        with_context("LayerManager::set_visibility", LogCategory::Map, |logger| {
            if map.get_layer(layer_id).is_some() {
                logger.debug(&format!(
                    "Setting '{}' visibility to {}",
                    layer_id,
                    if visible { "visible" } else { "none" }
                ));

                let visibility = if visible { "visible" } else { "none" };
                map.set_layout_property(layer_id, "visibility", &JsValue::from_str(visibility));

                // Update our tracking information
                if let Some(layer_info) = self.registered_layers.get_mut(layer_id) {
                    layer_info.visible = visible;
                }

                Ok(())
            } else {
                logger.error(&format!("Layer '{}' not found", layer_id));
                Err(JsValue::from_str(&format!(
                    "Layer '{}' not found",
                    layer_id
                )))
            }
        })
    }

    /// Update layer visibility based on TflLayers struct
    pub fn update_visibility(
        &self,
        map: &Map,
        layers: &crate::app::TflLayers,
    ) -> Result<(), JsValue> {
        with_context(
            "LayerManager::update_visibility",
            LogCategory::Map,
            |logger| {
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

                Ok(())
            },
        )
    }
}

/// Helper function to add MapLibre layers
pub fn add_map_layers(map_instance: &JsValue, simulation_enabled: bool) -> Result<(), JsValue> {
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
