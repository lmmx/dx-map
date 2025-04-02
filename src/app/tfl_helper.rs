// src/app/tfl_helper.rs
use wasm_bindgen::prelude::*;
use web_sys::console;
use crate::maplibre::tfl_integration::TflMapIntegration;
use crate::app::TflLayers;

// Helper struct to integrate TfL data with the existing app
pub struct TflHelper {
    integration: TflMapIntegration,
}

impl TflHelper {
    // Create a new helper
    pub fn new() -> Self {
        Self {
            integration: TflMapIntegration::new(),
        }
    }
    
    // Initialize TfL data when the map is ready
    pub fn initialize(&mut self, simulation_enabled: bool) -> Result<(), JsValue> {
        console::log_1(&"Initializing TfL helper...".into());
        
        // Access the map instance from the window
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window found"))?;
        let map_instance = js_sys::Reflect::get(&window, &JsValue::from_str("mapInstance"))?;
        
        if map_instance.is_undefined() {
            return Err(JsValue::from_str("Map instance not found on window"));
        }
        
        // Convert to the Map type
        let map = map_instance.clone().into();
        
        // Initialize the TfL data
        self.integration.initialize(&map, simulation_enabled)?;
        
        console::log_1(&"TfL helper initialized successfully".into());
        Ok(())
    }
    
    // Update layer visibility based on current settings
    pub fn update_visibility(&self, layers: &TflLayers) -> Result<(), JsValue> {
        self.integration.update_layer_visibility(layers)
    }
    
    // Toggle simulation on/off
    pub fn toggle_simulation(&mut self) -> Result<(), JsValue> {
        self.integration.toggle_simulation()
    }
    
    // Reset simulation
    pub fn reset_simulation(&mut self) -> Result<(), JsValue> {
        self.integration.reset_simulation()
    }
    
    // Set simulation speed
    pub fn set_simulation_speed(&mut self, speed: f64) -> Result<(), JsValue> {
        self.integration.set_simulation_speed(speed)
    }
}