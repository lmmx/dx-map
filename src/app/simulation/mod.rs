use crate::app::simulation::model::build_routes_from_tfl_data;
use crate::data::TflDataRepository;
use crate::utils::log::{self, LogCategory, with_context};
use js_sys::{Object, Reflect};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::window;

// Import from our modules
mod model;
mod state;

pub use model::{VehicleType, build_sample_routes, initialize_vehicles};
pub use state::{
    SimulationState, get_animation_frame_id, get_vehicle_count, initialize_state, is_paused,
    set_animation_frame_id, toggle_pause, with_simulation_state, with_simulation_state_ref,
};

// MapLibre integration components
// ------------------------------

/// Expose initialization function globally
// Expose Rust functions to JavaScript
pub fn expose_simulation_functions(tfl_data: Option<TflDataRepository>) -> Result<(), JsValue> {
    with_context(
        "expose_simulation_functions",
        LogCategory::Simulation,
        |logger| {
            logger.info("Exposing simulation functions to JavaScript");

            // Clone tfl_data so the closure can be called more than once.
            let tfl_data_for_closure = tfl_data.clone();
            // Create initialize function
            let init_closure = Closure::wrap(Box::new({
                let tfl_data_inner = tfl_data_for_closure;
                move || {
                    log::info_with_category(
                        LogCategory::Simulation,
                        "rust_initialize_simulation called from JS",
                    );
                    initialize_simulation(tfl_data_inner.clone());
                }
            }) as Box<dyn FnMut()>);

            // Create toggle function
            let toggle_closure = Closure::wrap(Box::new(|| {
                log::info_with_category(
                    LogCategory::Simulation,
                    "rust_toggle_simulation called from JS",
                );
                toggle_simulation();
            }) as Box<dyn FnMut()>);

            // Clone tfl_data for reset closure too
            let tfl_data_for_reset = tfl_data.clone();
            // Create reset function
            let reset_closure = Closure::wrap(Box::new({
                let tfl_data_inner = tfl_data_for_reset;
                move || {
                    log::info_with_category(
                        LogCategory::Simulation,
                        "rust_reset_simulation called from JS",
                    );
                    reset_simulation(tfl_data_inner.clone());
                }
            }) as Box<dyn FnMut()>);

            // Set them on the window object
            if let Some(window) = window() {
                js_sys::Reflect::set(
                    &window,
                    &JsValue::from_str("rust_initialize_simulation"),
                    init_closure.as_ref(),
                )
                .expect("Could not set rust_initialize_simulation");

                js_sys::Reflect::set(
                    &window,
                    &JsValue::from_str("rust_toggle_simulation"),
                    toggle_closure.as_ref(),
                )
                .expect("Could not set rust_toggle_simulation");

                js_sys::Reflect::set(
                    &window,
                    &JsValue::from_str("rust_reset_simulation"),
                    reset_closure.as_ref(),
                )
                .expect("Could not set rust_reset_simulation");

                logger.info("Simulation functions exposed to JavaScript");
            }

            // Leak the closures (they will live for the lifetime of the page)
            init_closure.forget();
            toggle_closure.forget();
            reset_closure.forget();

            Ok(())
        },
    )
}

// SIMULATION FUNCTIONS
// -------------------

/// Initialize the vehicle simulation
pub fn initialize_simulation(tfl_data: Option<TflDataRepository>) {
    with_context("initialize_simulation", LogCategory::Simulation, |logger| {
        logger.info("Initializing vehicle simulation...");

        // Set a global flag to track simulation visibility
        let js_code = r#"
        window.simulationVisible = true;
        console.log('Set window.simulationVisible = true');
        "#;
        let _ = js_sys::eval(js_code);

        // Build routes from real TfL data if available, otherwise use sample routes
        let routes = match tfl_data {
            Some(repo) => build_routes_from_tfl_data(&repo),
            None => {
                logger.warn("No TfL data provided, using sample routes");
                build_sample_routes()
            }
        };

        // Initialize vehicles on those routes
        let vehicles = initialize_vehicles(&routes);

        // Store in global state
        initialize_state(routes, vehicles);

        // Register with MapLibre and start animation
        register_vehicle_layers();
        start_animation_loop();

        logger.info("Simulation initialized");
    });
}

/// Register vehicle layers with MapLibre GL
fn register_vehicle_layers() {
    with_context(
        "register_vehicle_layers",
        LogCategory::Simulation,
        |logger| {
            logger.info("Registering vehicle layers with MapLibre using Rust bindings");

            // Get the map instance from window
            if let Some(window) = window() {
                if let Ok(map_instance) =
                    js_sys::Reflect::get(&window, &JsValue::from_str("mapInstance"))
                {
                    let map: crate::maplibre::bindings::Map = map_instance.into();

                    // Check if source already exists
                    if map.get_source("vehicles-source").is_none() {
                        // Create GeoJSON source for vehicles
                        let source = {
                            let obj = Object::new();
                            Reflect::set(
                                &obj,
                                &JsValue::from_str("type"),
                                &JsValue::from_str("geojson"),
                            )
                            .unwrap();

                            let data = Object::new();
                            Reflect::set(
                                &data,
                                &JsValue::from_str("type"),
                                &JsValue::from_str("FeatureCollection"),
                            )
                            .unwrap();
                            Reflect::set(
                                &data,
                                &JsValue::from_str("features"),
                                &js_sys::Array::new(),
                            )
                            .unwrap();

                            Reflect::set(&obj, &JsValue::from_str("data"), &data).unwrap();
                            obj
                        };

                        // Add the source
                        map.add_source("vehicles-source", &source);

                        // Create bus layer
                        let bus_layer = {
                            let layer = Object::new();
                            Reflect::set(
                                &layer,
                                &JsValue::from_str("id"),
                                &JsValue::from_str("buses-layer"),
                            )
                            .unwrap();
                            Reflect::set(
                                &layer,
                                &JsValue::from_str("type"),
                                &JsValue::from_str("circle"),
                            )
                            .unwrap();
                            Reflect::set(
                                &layer,
                                &JsValue::from_str("source"),
                                &JsValue::from_str("vehicles-source"),
                            )
                            .unwrap();

                            // Add filter
                            let filter = js_sys::Array::new();
                            filter.push(&JsValue::from_str("=="));

                            let get_expr = js_sys::Array::new();
                            get_expr.push(&JsValue::from_str("get"));
                            get_expr.push(&JsValue::from_str("vehicleType"));

                            filter.push(&get_expr);
                            filter.push(&JsValue::from_str("Bus"));

                            Reflect::set(&layer, &JsValue::from_str("filter"), &filter).unwrap();

                            // Paint properties
                            let paint = Object::new();
                            Reflect::set(
                                &paint,
                                &JsValue::from_str("circle-radius"),
                                &JsValue::from_f64(6.0),
                            )
                            .unwrap();
                            Reflect::set(
                                &paint,
                                &JsValue::from_str("circle-color"),
                                &JsValue::from_str("#0000FF"),
                            )
                            .unwrap();
                            Reflect::set(
                                &paint,
                                &JsValue::from_str("circle-stroke-color"),
                                &JsValue::from_str("#FFFFFF"),
                            )
                            .unwrap();
                            Reflect::set(
                                &paint,
                                &JsValue::from_str("circle-stroke-width"),
                                &JsValue::from_f64(2.0),
                            )
                            .unwrap();

                            Reflect::set(&layer, &JsValue::from_str("paint"), &paint).unwrap();
                            layer
                        };

                        // Add bus layer
                        map.add_layer(&bus_layer);

                        // Create train layer (similar to bus layer but different color and filter)
                        let train_layer = {
                            let layer = Object::new();
                            Reflect::set(
                                &layer,
                                &JsValue::from_str("id"),
                                &JsValue::from_str("trains-layer"),
                            )
                            .unwrap();
                            Reflect::set(
                                &layer,
                                &JsValue::from_str("type"),
                                &JsValue::from_str("circle"),
                            )
                            .unwrap();
                            Reflect::set(
                                &layer,
                                &JsValue::from_str("source"),
                                &JsValue::from_str("vehicles-source"),
                            )
                            .unwrap();

                            // Add filter
                            let filter = js_sys::Array::new();
                            filter.push(&JsValue::from_str("=="));

                            let get_expr = js_sys::Array::new();
                            get_expr.push(&JsValue::from_str("get"));
                            get_expr.push(&JsValue::from_str("vehicleType"));

                            filter.push(&get_expr);
                            filter.push(&JsValue::from_str("Train"));

                            Reflect::set(&layer, &JsValue::from_str("filter"), &filter).unwrap();

                            // Paint properties
                            let paint = Object::new();
                            Reflect::set(
                                &paint,
                                &JsValue::from_str("circle-radius"),
                                &JsValue::from_f64(6.0),
                            )
                            .unwrap();
                            Reflect::set(
                                &paint,
                                &JsValue::from_str("circle-color"),
                                &JsValue::from_str("#FF0000"),
                            )
                            .unwrap();
                            Reflect::set(
                                &paint,
                                &JsValue::from_str("circle-stroke-color"),
                                &JsValue::from_str("#FFFFFF"),
                            )
                            .unwrap();
                            Reflect::set(
                                &paint,
                                &JsValue::from_str("circle-stroke-width"),
                                &JsValue::from_f64(2.0),
                            )
                            .unwrap();

                            Reflect::set(&layer, &JsValue::from_str("paint"), &paint).unwrap();
                            layer
                        };

                        // Add train layer
                        map.add_layer(&train_layer);
                        logger.info("Vehicle layers successfully added using Rust bindings");
                    } else {
                        logger.info("Vehicle source already exists, skipping layer creation");
                    }
                } else {
                    logger.error("Could not get mapInstance from window");
                }
            } else {
                logger.error("Window object not available");
            }
        },
    )
}

/// Start the animation loop for vehicle movement
fn start_animation_loop() {
    with_context("start_animation_loop", LogCategory::Simulation, |logger| {
        logger.debug("Starting animation loop for vehicle movement");

        // Create a callback that will run on each animation frame
        let animation_callback = Closure::wrap(Box::new(move || {
            // Only update if not paused
            let should_continue = with_simulation_state(|sim_state| {
                if !sim_state.is_paused {
                    // 1. Update vehicle positions
                    update_vehicle_positions(sim_state);

                    // 2. Update MapLibre with new positions
                    update_maplibre_vehicles(sim_state);
                }

                // Return whether we're paused to determine if we should request another frame
                !sim_state.is_paused
            });

            // Request next animation frame if not paused
            if should_continue {
                request_animation_frame();
            }
        }) as Box<dyn FnMut()>);

        // Store the callback and request first frame
        if let Some(window) = window() {
            match window.request_animation_frame(animation_callback.as_ref().unchecked_ref()) {
                Ok(id) => {
                    // Store the callback and frame ID
                    set_animation_frame_id(id);

                    logger.debug(&format!("Animation frame requested, ID: {}", id));

                    // Forget the closure to keep it alive (will be cleaned up when simulation is reset)
                    animation_callback.forget();
                }
                Err(err) => {
                    logger.error(&format!("Failed to request animation frame: {:?}", err));
                }
            }
        } else {
            logger.error("No global window exists, cannot start animation loop");
        }
    })
}

/// Request a new animation frame
fn request_animation_frame() {
    let animation_callback = Closure::wrap(Box::new(move || {
        let should_continue = with_simulation_state(|sim_state| {
            if !sim_state.is_paused {
                // Update vehicle positions
                update_vehicle_positions(sim_state);

                // Update MapLibre with new positions
                update_maplibre_vehicles(sim_state);
            }

            // Return whether we're paused to determine if we should request another frame
            !sim_state.is_paused
        });

        // Request next animation frame if not paused
        if should_continue {
            request_animation_frame();
        }
    }) as Box<dyn FnMut()>);

    // Request animation frame
    if let Some(window) = window() {
        match window.request_animation_frame(animation_callback.as_ref().unchecked_ref()) {
            Ok(id) => {
                // Store the animation frame ID
                set_animation_frame_id(id);

                // Forget the closure to keep it alive
                animation_callback.forget();
            }
            Err(err) => {
                log::error_with_category(
                    LogCategory::Simulation,
                    &format!("Failed to request animation frame: {:?}", err),
                );
            }
        }
    } else {
        log::error_with_category(
            LogCategory::Simulation,
            "No global window exists, cannot request animation frame",
        );
    }
}

/// Update positions of all vehicles based on their speed and direction
fn update_vehicle_positions(sim_state: &mut SimulationState) {
    // This function is called many times per second - avoid excessive logging
    // Only log periodically for debugging purposes
    static mut POSITION_UPDATE_COUNTER: u32 = 0;
    let should_log = unsafe {
        POSITION_UPDATE_COUNTER += 1;
        POSITION_UPDATE_COUNTER % 300 == 0 // Log roughly every 5 seconds (assuming 60fps)
    };

    if should_log {
        log::debug_with_category(
            LogCategory::Simulation,
            &format!(
                "Updating positions for {} vehicles",
                sim_state.vehicles.len()
            ),
        );
    }

    for vehicle in &mut sim_state.vehicles {
        // Get the current route
        let route = &sim_state.routes[vehicle.route_index];

        // Update position along segment
        vehicle.position += vehicle.speed;

        // Check if we've reached the next station
        while vehicle.position >= 1.0 {
            vehicle.position -= 1.0;
            vehicle.last_station = vehicle.next_station;

            // Determine next station based on direction
            let next_station = (vehicle.last_station as i32) + (vehicle.direction as i32);

            // Check if we need to reverse direction (reached end of line)
            if next_station < 0 || next_station >= route.stations.len() as i32 {
                // Reverse direction
                vehicle.direction *= -1;

                // Recalculate next station
                let next_station = (vehicle.last_station as i32) + (vehicle.direction as i32);
                vehicle.next_station = next_station as usize;
            } else {
                vehicle.next_station = next_station as usize;
            }
        }

        // Interpolate position between stations
        let (last_lng, last_lat) = route.stations[vehicle.last_station];
        let (next_lng, next_lat) = route.stations[vehicle.next_station];

        // Linear interpolation based on position (0.0 to 1.0)
        vehicle.lng = last_lng + (next_lng - last_lng) * vehicle.position;
        vehicle.lat = last_lat + (next_lat - last_lat) * vehicle.position;
    }
}

/// Update MapLibre with the current vehicle positions
fn update_maplibre_vehicles(sim_state: &SimulationState) {
    // This function is called many times per second - avoid excessive logging
    static mut MAPLIBRE_UPDATE_COUNTER: u32 = 0;
    let should_log = unsafe {
        MAPLIBRE_UPDATE_COUNTER += 1;
        MAPLIBRE_UPDATE_COUNTER % 600 == 0 // Log roughly every 10 seconds (assuming 60fps)
    };

    if should_log {
        log::debug_with_category(
            LogCategory::Simulation,
            &format!(
                "Updating MapLibre with {} vehicle positions",
                sim_state.vehicles.len()
            ),
        );
    }

    // Instead of trying to build a complex JS object, let's construct a simple JSON string directly
    let mut features = Vec::new();

    for vehicle in &sim_state.vehicles {
        // Format each vehicle as a GeoJSON feature
        let vehicle_type = match vehicle.vehicle_type {
            VehicleType::Bus => "Bus",
            VehicleType::Train => "Train",
        };

        let feature = format!(
            r#"{{
                "type": "Feature",
                "geometry": {{
                    "type": "Point",
                    "coordinates": [{}, {}]
                }},
                "properties": {{
                    "id": {},
                    "vehicleType": "{}"
                }}
            }}"#,
            vehicle.lng, vehicle.lat, vehicle.id, vehicle_type
        );

        features.push(feature);
    }

    // Join all features into a GeoJSON collection
    let geojson = format!(
        r#"{{
            "type": "FeatureCollection",
            "features": [{}]
        }}"#,
        features.join(",")
    );

    // Update the source in MapLibre
    let js_code = format!(
        r#"
        if (window.mapInstance && window.mapInstance.getSource('vehicles-source')) {{
            try {{
                const data = {};
                window.mapInstance.getSource('vehicles-source').setData(data);
            }} catch (e) {{
                console.error("Error updating vehicles source:", e);
            }}
        }}
        "#,
        geojson
    );

    let eval_result = js_sys::eval(&js_code);
    if should_log && eval_result.is_err() {
        log::error_with_category(
            LogCategory::Simulation,
            &format!(
                "Failed to update vehicle positions in MapLibre: {:?}",
                eval_result.err()
            ),
        );
    }
}

/// Serialize a JS object to a JSON string for embedding in JS code
fn serialize_geojson_data(geojson: &Object) -> String {
    with_context(
        "serialize_geojson_data",
        LogCategory::Simulation,
        |logger| {
            logger.debug("Serializing GeoJSON data to string");

            let js_code = r#"
        function serializeToJSON(obj) {
            try {
                return JSON.stringify(obj);
            } catch (e) {
                console.error("Failed to stringify object:", e);
                return "{}";
            }
        }
        serializeToJSON(arguments[0]);
        "#;

            // Try to call the function safely
            match js_sys::Function::new_with_args("obj", js_code)
                .call1(&JsValue::NULL, &geojson.into())
            {
                Ok(result) => {
                    // Try to convert to string, fallback to empty JSON if it fails
                    result.as_string().unwrap_or_else(|| {
                        logger.error("Failed to get string from serialized result");
                        "{}".to_string()
                    })
                }
                Err(err) => {
                    // Log the error and return empty JSON
                    logger.error(&format!("Failed to serialize GeoJSON: {:?}", err));
                    "{}".to_string()
                }
            }
        },
    )
}

/// Toggle the simulation pause state
pub fn toggle_simulation() {
    with_context("toggle_simulation", LogCategory::Simulation, |logger| {
        let is_now_paused = toggle_pause();

        // If we're resuming, restart the animation loop
        if !is_now_paused {
            logger.info("Resuming simulation, restarting animation loop");
            start_animation_loop();
        } else {
            logger.info("Pausing simulation");
        }
    })
}

/// Reset the simulation
pub fn reset_simulation(tfl_data: Option<TflDataRepository>) {
    with_context("reset_simulation", LogCategory::Simulation, |logger| {
        logger.info("Resetting simulation...");

        // Cancel current animation frame if one is active
        if let Some(id) = get_animation_frame_id() {
            if let Some(window) = window() {
                if let Err(err) = window.cancel_animation_frame(id) {
                    logger.error(&format!("Failed to cancel animation frame: {:?}", err));
                } else {
                    logger.debug(&format!("Canceled animation frame ID: {}", id));
                }
            }
        }

        // Reset state and recreate everything
        logger.debug("Re-initializing simulation from scratch");
        initialize_simulation(tfl_data);

        logger.info("Simulation reset complete");
    })
}
