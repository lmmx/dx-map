use dioxus::prelude::*;

mod canvas;
mod key_panel;
mod layer_panel;
mod simulation;
mod tfl_helper;

use crate::maplibre::helpers;
use tfl_helper::TflHelper;
use wasm_bindgen::prelude::*;
use js_sys::Promise;
use web_sys::console;
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
            simulation: false, // Show simulation by default
        }
    }
}

#[component]
pub fn app() -> Element {
    let mut show_layers_panel = use_signal(|| false);
    let mut show_key_panel = use_signal(|| false);
    let mut show_simulation_panel = use_signal(|| false); // New signal for simulation controls
    let layers = use_signal(|| TflLayers::default());
    let helper = use_signal(|| TflHelper::new());

    // Initialize TfL data when the map is ready
    use_effect(move || {
        console::log_1(&"Setting up TfL data...".into());
        
        // First wait for the map to be loaded
        let js_code = r#"
            function waitForMap() {
                return new Promise((resolve, reject) => {
                    if (window.mapInstance) {
                        return resolve(true);
                    }
                    
                    const MAX_ATTEMPTS = 20;
                    let attempts = 0;
                    
                    const interval = setInterval(() => {
                        attempts++;
                        if (window.mapInstance) {
                            clearInterval(interval);
                            return resolve(true);
                        }
                        
                        if (attempts >= MAX_ATTEMPTS) {
                            clearInterval(interval);
                            return reject(new Error("Map not loaded after timeout"));
                        }
                    }, 500);
                });
            }
            
            waitForMap();
        "#;
        
        let promise = Promise::new(&mut |resolve, reject| {
            if let Err(e) = js_sys::eval(js_code) {
                reject.call1(&JsValue::NULL, &JsValue::from_str(&format!("Failed to setup wait: {:?}", e))).unwrap();
            }
        });
        
        // Spawn an async task to wait for the map and then initialize TfL
        wasm_bindgen_futures::spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(_) => {
                    console::log_1(&"Map is loaded, initializing TfL data...".into());
                    
                    // Wait a bit longer to ensure map is fully initialized
                    let delay_promise = Promise::new(&mut |resolve, _| {
                        let timeout_code = r#"
                            setTimeout(() => {
                                resolve(true);
                            }, 1000);
                        "#;
                        let _ = js_sys::eval(timeout_code);
                    });
                    
                    let _ = wasm_bindgen_futures::JsFuture::from(delay_promise).await;
                    
                    // Now initialize the TfL helper
                    if let Err(e) = helper.write().initialize(layers.read().simulation) {
                        console::error_1(&format!("Failed to initialize TfL data: {:?}", e).into());
                    } else {
                        console::log_1(&"TfL data initialized successfully".into());
                    }
                },
                Err(e) => {
                    console::error_1(&format!("Map load timed out: {:?}", e).into());
                }
            }
        });
    });
    
    // Add effects to update TfL layers when settings change
    use_effect(move || {
        // Update visibility when any layer option changes
        if let Err(e) = helper.read().update_visibility(&layers.read()) {
            console::error_1(&format!("Failed to update TfL visibility: {:?}", e).into());
        }
    });
    
    // Add effect to update simulation state
    use_effect(move || {
        // Toggle simulation when simulation flag changes
        let simulation_enabled = layers.read().simulation;
        
        console::log_1(&format!("Simulation enabled: {}", simulation_enabled).into());
        
        // Call toggle_simulation if it should be different than current state
        if let Err(e) = helper.write().toggle_simulation() {
            console::error_1(&format!("Failed to toggle simulation: {:?}", e).into());
        }
    });
    
    // Add effect to update simulation speed
    use_effect(move || {
        let speed = layers.read().simulation_speed;
        
        // Convert to 0.0-1.0 range
        let normalized_speed = speed as f64 / 5.0;
        
        console::log_1(&format!("Setting simulation speed: {} ({})", speed, normalized_speed).into());
        
        if let Err(e) = helper.write().set_simulation_speed(normalized_speed) {
            console::error_1(&format!("Failed to set simulation speed: {:?}", e).into());
        }
    });

    // Initialize simulation JS when app loads
    use_effect(move || {
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
            web_sys::console::error_1(&format!("Failed to add simulation script: {:?}", e).into());
        } else {
            web_sys::console::log_1(&"Simulation controller script added".into());
        }
    });

    use_effect(move || {
        // Try to expose simulation functions if available
        if let Ok(_) = simulation::expose_simulation_functions() {
            web_sys::console::log_1(&"Simulation functions exposed on app start".into());
        }

        // Add the controller script
        let controller_script = r#"
	// SimulationController code here...
	"#;

        if let Err(e) = helpers::add_inline_script(controller_script) {
            web_sys::console::error_1(&format!("Failed to add simulation script: {:?}", e).into());
        }
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
            Canvas { layers: layers }

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
