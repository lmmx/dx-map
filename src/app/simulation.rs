use crate::utils::log::{self, LogCategory, with_context};
use dioxus::prelude::*;
use js_sys::{Array, Math, Object, Reflect};
use std::cell::RefCell;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::window;

// Model components
// ---------------

// Core shared state that contains all vehicles
#[derive(Default)]
pub struct SimulationState {
    vehicles: Vec<Vehicle>,
    routes: Vec<Route>,
    is_paused: bool,
    animation_frame_id: Option<i32>,
}

// We'll use thread_local for our global state
thread_local! {
    static SIMULATION_STATE: RefCell<SimulationState> = RefCell::new(SimulationState::default());
}

#[derive(Clone, Debug)]
pub enum VehicleType {
    Bus,
    Train,
}

#[derive(Clone, Debug)]
pub struct Vehicle {
    id: usize,
    vehicle_type: VehicleType,
    route_index: usize,
    position: f64,       // 0.0 to 1.0 position along route segment
    speed: f64,          // Movement speed
    direction: i8,       // 1 = forward, -1 = backward
    last_station: usize, // Index of last station
    next_station: usize, // Index of station we're heading towards
    lng: f64,            // Current longitude
    lat: f64,            // Current latitude
}

#[derive(Clone, Debug)]
pub struct Route {
    id: usize,
    name: String,
    vehicle_type: VehicleType,
    stations: Vec<(f64, f64)>, // Vec of (lng, lat) coordinates
}

// MapLibre integration components
// ------------------------------

/// Expose initialization function globally
// Expose Rust functions to JavaScript
pub fn expose_simulation_functions() -> Result<(), JsValue> {
    with_context(
        "expose_simulation_functions",
        LogCategory::Simulation,
        |logger| {
            logger.info("Exposing simulation functions to JavaScript");

            // Create initialize function
            let init_closure = Closure::wrap(Box::new(|| {
                log::info_with_category(
                    LogCategory::Simulation,
                    "rust_initialize_simulation called from JS",
                );
                initialize_simulation();
            }) as Box<dyn FnMut()>);

            // Create toggle function
            let toggle_closure = Closure::wrap(Box::new(|| {
                log::info_with_category(
                    LogCategory::Simulation,
                    "rust_toggle_simulation called from JS",
                );
                toggle_simulation();
            }) as Box<dyn FnMut()>);

            // Create reset function
            let reset_closure = Closure::wrap(Box::new(|| {
                log::info_with_category(
                    LogCategory::Simulation,
                    "rust_reset_simulation called from JS",
                );
                reset_simulation();
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
fn initialize_simulation() {
    with_context("initialize_simulation", LogCategory::Simulation, |logger| {
        logger.info("Initializing vehicle simulation...");

        // Set a global flag to track simulation visibility
        let js_code = r#"
        window.simulationVisible = true;
        console.log('Set window.simulationVisible = true');
        "#;
        let _ = js_sys::eval(js_code);

        // Build routes
        let routes = build_sample_routes();

        // Initialize vehicles on those routes
        let vehicles = initialize_vehicles(&routes);

        // Store in global state
        SIMULATION_STATE.with(|state| {
            let mut sim_state = state.borrow_mut();
            sim_state.routes = routes;
            sim_state.vehicles = vehicles;
            sim_state.is_paused = false;
        });

        // Register with MapLibre and start animation
        register_vehicle_layers();
        start_animation_loop();

        logger.info("Simulation initialized");
    });
}

/// Create sample routes based on TfL network
fn build_sample_routes() -> Vec<Route> {
    with_context("build_sample_routes", LogCategory::Simulation, |logger| {
        logger.debug("Creating sample routes based on TfL network");

        let mut routes = Vec::new();

        // Central Line (simplified)
        routes.push(Route {
            id: 0,
            name: "Central Line".to_string(),
            vehicle_type: VehicleType::Train,
            stations: vec![
                // West to East: Longitude, Latitude
                (-0.2810, 51.5170), // West Ruislip
                (-0.2528, 51.5113), // Ruislip Gardens
                (-0.2194, 51.5136), // South Ruislip
                (-0.1987, 51.5202), // Northolt
                (-0.1652, 51.5259), // Greenford
                (-0.1350, 51.5210), // Perivale
                (-0.0997, 51.5152), // Hanger Lane
                (-0.0638, 51.5165), // North Acton
                (-0.0362, 51.5111), // East Acton
                (-0.0244, 51.5043), // White City
                (-0.0048, 51.5035), // Shepherd's Bush
                (-0.0125, 51.5009), // Holland Park
                (-0.0199, 51.4996), // Notting Hill Gate
                (-0.0457, 51.5068), // Queensway
                (-0.0742, 51.5113), // Lancaster Gate
                (-0.0983, 51.5142), // Marble Arch
                (-0.1280, 51.5151), // Bond Street
                (-0.1410, 51.5154), // Oxford Circus
                (-0.1687, 51.5174), // Tottenham Court Road
                (-0.1889, 51.5206), // Holborn
                (-0.1205, 51.5152), // Chancery Lane
                (-0.1025, 51.5168), // St. Paul's
                (-0.0911, 51.5155), // Bank
                (-0.0765, 51.5108), // Liverpool Street
            ],
        });
        logger.debug("Added Central Line route");

        // Northern Line (simplified)
        routes.push(Route {
            id: 1,
            name: "Northern Line".to_string(),
            vehicle_type: VehicleType::Train,
            stations: vec![
                // North to South
                (-0.1938, 51.6503), // High Barnet
                (-0.1932, 51.6302), // Totteridge & Whetstone
                (-0.1858, 51.6179), // Woodside Park
                (-0.1750, 51.6071), // West Finchley
                (-0.1647, 51.5998), // Finchley Central
                (-0.1534, 51.5874), // East Finchley
                (-0.1419, 51.5775), // Highgate
                (-0.1303, 51.5717), // Archway
                (-0.1123, 51.5656), // Tufnell Park
                (-0.1051, 51.5545), // Kentish Town
                (-0.1426, 51.5302), // Camden Town
                (-0.1385, 51.5248), // Mornington Crescent
                (-0.1343, 51.5287), // Euston
                (-0.1304, 51.5295), // King's Cross St. Pancras
                (-0.1231, 51.5203), // Angel
                (-0.1065, 51.5121), // Old Street
                (-0.0882, 51.5176), // Moorgate
                (-0.0911, 51.5155), // Bank
                (-0.0924, 51.5113), // London Bridge
                (-0.1002, 51.5044), // Borough
                (-0.1052, 51.4944), // Elephant & Castle
            ],
        });
        logger.debug("Added Northern Line route");

        // Bus route (sample)
        routes.push(Route {
            id: 2,
            name: "Bus 88".to_string(),
            vehicle_type: VehicleType::Bus,
            stations: vec![
                // West to East (Camden to Canning Town)
                (-0.1465, 51.5365), // Camden Town
                (-0.1325, 51.5300), // St Pancras
                (-0.1155, 51.5235), // Farringdon
                (-0.0958, 51.5181), // Barbican
                (-0.0879, 51.5155), // Moorgate
                (-0.0825, 51.5127), // Liverpool Street
                (-0.0754, 51.5101), // Aldgate
                (-0.0650, 51.5088), // Aldgate East
                (-0.0550, 51.5070), // Whitechapel
                (-0.0449, 51.5055), // Stepney Green
                (-0.0349, 51.5040), // Mile End
                (-0.0250, 51.5025), // Bow Road
                (-0.0150, 51.5010), // Bow Church
                (-0.0050, 51.4995), // Devons Road
                (0.0050, 51.4980),  // Langdon Park
                (0.0150, 51.4965),  // All Saints
                (0.0250, 51.4950),  // Poplar
                (0.0350, 51.4935),  // Blackwall
                (0.0450, 51.4920),  // East India
                (0.0550, 51.4905),  // Canning Town
            ],
        });
        logger.debug("Added Bus 88 route");

        logger.info(&format!("Created {} sample routes", routes.len()));
        routes
    })
}

/// Initialize vehicles on the routes
fn initialize_vehicles(routes: &[Route]) -> Vec<Vehicle> {
    with_context("initialize_vehicles", LogCategory::Simulation, |logger| {
        logger.debug("Initializing vehicles on routes");

        let mut vehicles = Vec::new();
        let mut id_counter = 0;

        for route in routes {
            let vehicle_count = match route.vehicle_type {
                VehicleType::Train => 10, // 10 trains per line
                VehicleType::Bus => 20,   // 20 buses per route
            };

            logger.debug(&format!(
                "Adding {} vehicles for route: {} ({})",
                vehicle_count,
                route.name,
                match route.vehicle_type {
                    VehicleType::Train => "Train",
                    VehicleType::Bus => "Bus",
                }
            ));

            // Create vehicles distributed along the route
            for i in 0..vehicle_count {
                // Determine starting positions and directions
                let (last_station, next_station, direction) = if i % 2 == 0 {
                    // Forward direction
                    (0, 1, 1)
                } else {
                    // Backward direction
                    (route.stations.len() - 1, route.stations.len() - 2, -1)
                };

                // Get station coordinates
                let (start_lng, start_lat) = route.stations[last_station];

                // Create vehicle
                vehicles.push(Vehicle {
                    id: id_counter,
                    vehicle_type: route.vehicle_type.clone(),
                    route_index: route.id,
                    position: Math::random(), // Random position along segment
                    speed: 0.005 + Math::random() * 0.01, // Random speed
                    direction,
                    last_station,
                    next_station,
                    lng: start_lng,
                    lat: start_lat,
                });

                id_counter += 1;
            }
        }

        logger.info(&format!(
            "Created {} vehicles across all routes",
            vehicles.len()
        ));
        vehicles
    })
}

/// Register vehicle layers with MapLibre GL
fn register_vehicle_layers() {
    with_context(
        "register_vehicle_layers",
        LogCategory::Simulation,
        |logger| {
            logger.info("Registering vehicle layers with MapLibre");

            // Access the MapLibre instance
            let js_code = r#"
        if (window.mapInstance) {
            // Add a source for vehicles
            if (!window.mapInstance.getSource('vehicles-source')) {
                window.mapInstance.addSource('vehicles-source', {
                    type: 'geojson',
                    data: {
                        type: 'FeatureCollection',
                        features: []
                    }
                });

                // Add a layer for bus vehicles
                window.mapInstance.addLayer({
                    id: 'buses-layer',
                    type: 'circle',
                    source: 'vehicles-source',
                    filter: ['==', ['get', 'vehicleType'], 'Bus'],
                    paint: {
                        'circle-radius': 6,
                        'circle-color': '#0000FF',
                        'circle-stroke-color': '#FFFFFF',
                        'circle-stroke-width': 2
                    }
                });

                // Add a layer for train vehicles
                window.mapInstance.addLayer({
                    id: 'trains-layer',
                    type: 'circle',
                    source: 'vehicles-source',
                    filter: ['==', ['get', 'vehicleType'], 'Train'],
                    paint: {
                        'circle-radius': 6,
                        'circle-color': '#FF0000',
                        'circle-stroke-color': '#FFFFFF',
                        'circle-stroke-width': 2
                    }
                });

                // Make sure the simulation layers are visible by default
                const initialVisibility = window.simulationVisible === false ? 'none' : 'visible';
                window.mapInstance.setLayoutProperty('buses-layer', 'visibility', initialVisibility);
                window.mapInstance.setLayoutProperty('trains-layer', 'visibility', initialVisibility);

                console.log('Vehicle layers registered with MapLibre, visibility:', initialVisibility);
            }
        } else {
            console.error('MapInstance not found!');
        }
        "#;

            let eval_result = js_sys::eval(js_code);
            if let Err(err) = eval_result {
                logger.error(&format!("Failed to register vehicle layers: {:?}", err));
            } else {
                logger.debug("Vehicle layers registered successfully");
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
            let should_continue = SIMULATION_STATE.with(|state| {
                let mut sim_state = state.borrow_mut();
                if !sim_state.is_paused {
                    // 1. Update vehicle positions
                    update_vehicle_positions(&mut sim_state);

                    // 2. Update MapLibre with new positions
                    update_maplibre_vehicles(&sim_state);
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
                    SIMULATION_STATE.with(|state| {
                        let mut sim_state = state.borrow_mut();
                        sim_state.animation_frame_id = Some(id);
                    });

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
        let should_continue = SIMULATION_STATE.with(|state| {
            let mut sim_state = state.borrow_mut();
            if !sim_state.is_paused {
                // Update vehicle positions
                update_vehicle_positions(&mut sim_state);

                // Update MapLibre with new positions
                update_maplibre_vehicles(&sim_state);
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
                SIMULATION_STATE.with(|state| {
                    let mut sim_state = state.borrow_mut();
                    sim_state.animation_frame_id = Some(id);
                });

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
fn toggle_simulation() {
    with_context("toggle_simulation", LogCategory::Simulation, |logger| {
        let is_now_paused = SIMULATION_STATE.with(|state| {
            let mut sim_state = state.borrow_mut();
            sim_state.is_paused = !sim_state.is_paused;
            sim_state.is_paused
        });

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
fn reset_simulation() {
    with_context("reset_simulation", LogCategory::Simulation, |logger| {
        logger.info("Resetting simulation...");

        // Cancel current animation frame if one is active
        SIMULATION_STATE.with(|state| {
            let sim_state = state.borrow();
            if let Some(id) = sim_state.animation_frame_id {
                if let Some(window) = window() {
                    if let Err(err) = window.cancel_animation_frame(id) {
                        logger.error(&format!("Failed to cancel animation frame: {:?}", err));
                    } else {
                        logger.debug(&format!("Canceled animation frame ID: {}", id));
                    }
                }
            }
        });

        // Reset state and recreate everything
        logger.debug("Re-initializing simulation from scratch");
        initialize_simulation();

        logger.info("Simulation reset complete");
    })
}

/// Debug function to log important simulation state
fn debug_simulation_state(sim_state: &SimulationState) {
    // Only log periodically to avoid console spam
    static mut COUNTER: u32 = 0;
    unsafe {
        COUNTER += 1;
        if COUNTER % 60 != 0 {
            // Log every ~60 frames (roughly 1 second)
            return;
        }
    }

    with_context(
        "debug_simulation_state",
        LogCategory::Simulation,
        |logger| {
            // Log general state
            logger.debug(&format!(
                "Simulation state: {} vehicles, paused: {}",
                sim_state.vehicles.len(),
                sim_state.is_paused
            ));

            // Log a sample vehicle
            if !sim_state.vehicles.is_empty() {
                let sample = &sim_state.vehicles[0];
                logger.debug(&format!(
                    "Sample vehicle: id={}, type={:?}, pos=({:.4}, {:.4})",
                    sample.id, sample.vehicle_type, sample.lng, sample.lat
                ));
            }

            // Check if vehicles source exists and log its state
            let js_code = r#"
        let result = "unknown";
        if (window.mapInstance) {
            const source = window.mapInstance.getSource('vehicles-source');
            if (source) {
                try {
                    const data = source._data;
                    const features = data.features || [];
                    result = `Source exists, ${features.length} features`;
                } catch (e) {
                    result = `Source exists but error: ${e.message}`;
                }
            } else {
                result = "Source does not exist";
            }
        } else {
            result = "Map instance not found";
        }
        result;
        "#;

            match js_sys::eval(js_code) {
                Ok(result) => {
                    if let Some(result_str) = result.as_string() {
                        logger.debug(&format!("Vehicles source check: {}", result_str));
                    }
                }
                Err(err) => {
                    logger.error(&format!("Failed to check vehicles source: {:?}", err));
                }
            }
        },
    )
}
