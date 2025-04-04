// Controls management for map
use crate::maplibre::bindings::{
    KeyControl, LayerSwitcher, Map, NavigationControl, ScaleControl, SimulationControl,
};
use crate::maplibre::helpers::{create_layer_groups, create_scale_control_options};
use crate::utils::log::{self, LogCategory, with_context};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// Manager for map controls
pub struct ControlManager {
    registered_controls: HashMap<String, ControlInfo>,
}

/// Information about a registered control
struct ControlInfo {
    id: String,
    position: String,
}

impl ControlManager {
    pub fn new() -> Self {
        Self {
            registered_controls: HashMap::new(),
        }
    }

    /// Add a control to the map
    pub fn add_control(
        &mut self,
        map: &Map,
        id: &str,
        control: &JsValue,
        position: Option<&str>,
    ) -> Result<(), JsValue> {
        with_context("ControlManager::add_control", LogCategory::Map, |logger| {
            logger.debug(&format!(
                "Adding control '{}' at position '{}'",
                id,
                position.unwrap_or("default")
            ));

            // Add the control to the map
            map.addControl(control, position);

            // Register the control
            self.registered_controls.insert(
                id.to_string(),
                ControlInfo {
                    id: id.to_string(),
                    position: position.unwrap_or("default").to_string(),
                },
            );

            logger.debug(&format!("Control '{}' added and registered", id));

            Ok(())
        })
    }

    /// Remove a control from the map
    pub fn remove_control(&mut self, map: &Map, id: &str) -> Result<(), JsValue> {
        with_context(
            "ControlManager::remove_control",
            LogCategory::Map,
            |logger| {
                if let Some(control_info) = self.registered_controls.remove(id) {
                    // We don't actually have a way to get the control object back,
                    // so this would need to be changed to store the actual control objects
                    // Not implemented in this version as it's not used in the original code
                    logger.warn(&format!(
                        "Control '{}' removed from registry but not from map (not implemented)",
                        id
                    ));
                    Ok(())
                } else {
                    logger.error(&format!("Control '{}' not found", id));
                    Err(JsValue::from_str(&format!("Control '{}' not found", id)))
                }
            },
        )
    }

    /// Add all standard controls to the map
    pub fn add_all_controls(&mut self, map: &Map) -> Result<(), JsValue> {
        with_context(
            "ControlManager::add_all_controls",
            LogCategory::Map,
            |logger| {
                // Add navigation control
                logger.debug("Adding NavigationControl");
                let nav_control = NavigationControl::new();
                self.add_control(map, "navigation", &JsValue::from(nav_control), None)?;

                // Add scale control
                logger.debug("Adding ScaleControl");
                let scale_options = create_scale_control_options()?;
                let scale_control = ScaleControl::new(&scale_options);
                self.add_control(
                    map,
                    "scale",
                    &JsValue::from(scale_control),
                    Some("bottom-left"),
                )?;

                // Add key control
                logger.debug("Adding KeyControl");
                let key_control = KeyControl::new();
                self.add_control(map, "key", &JsValue::from(key_control), Some("top-right"))?;

                // Add layer switcher
                logger.debug("Adding LayerSwitcher");
                let layers = create_layer_groups()?;
                let layer_switcher = LayerSwitcher::new(&layers, "TfL Layers");
                self.add_control(
                    map,
                    "layer-switcher",
                    &JsValue::from(layer_switcher),
                    Some("top-right"),
                )?;

                // Add simulation control
                logger.debug("Adding SimulationControl");
                let simulation_control = SimulationControl::new();
                self.add_control(
                    map,
                    "simulation",
                    &JsValue::from(simulation_control),
                    Some("top-right"),
                )?;

                logger.info("All controls added successfully");
                Ok(())
            },
        )
    }
}
