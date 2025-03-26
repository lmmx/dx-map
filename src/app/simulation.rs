use dioxus::prelude::*;
use js_sys::{Array, Math, Object, Reflect};
use std::cell::RefCell;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::console;

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
    Train
}

#[derive(Clone, Debug)]
pub struct Vehicle {
    id: usize,
    vehicle_type: VehicleType,
    route_index: usize,
    position: f64,        // 0.0 to 1.0 position along route segment
    speed: f64,           // Movement speed
    direction: i8,        // 1 = forward, -1 = backward
    last_station: usize,  // Index of last station
    next_station: usize,  // Index of station we're heading towards
    lng: f64,             // Current longitude
    lat: f64,             // Current latitude
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

/// Component that initializes and manages the vehicle simulation
#[component]
pub fn VehicleSimulation() -> Element {
    let mut initialized = use_signal(|| false);
    
    // Set up a use_effect to initialize the simulation once
    use_effect(move || {
        if !*initialized.read() {
            initialize_simulation();
            initialized.set(true);
        }
    });
    
    rsx! {
        // UI Controls for simulation
        div {
            class: "simulation-controls",
            button {
                id: "toggle-simulation",
                onclick: move |_| toggle_simulation(),
                "Pause/Resume Simulation"
            }
            button {
                id: "reset-simulation",
                onclick: move |_| reset_simulation(),
                "Reset Simulation"
            }
        }
    }
}

// SIMULATION FUNCTIONS
// -------------------

/// Initialize the vehicle simulation
fn initialize_simulation() {
    console::log_1(&"Initializing vehicle simulation...".into());
    
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
    
    console::log_1(&"Simulation initialized".into());
}

/// Create sample routes based on TfL network
fn build_sample_routes() -> Vec<Route> {
    let mut routes = Vec::new();
    
    // Central Line (simplified)
    routes.push(Route {
        id: 0,
        name: "Central Line".to_string(),
        vehicle_type: VehicleType::Train,
        stations: vec![
            // West to East: Longitude, Latitude
            (-0.2810, 51.5170),  // West Ruislip
            (-0.2528, 51.5113),  // Ruislip Gardens
            (-0.2194, 51.5136),  // South Ruislip
            (-0.1987, 51.5202),  // Northolt
            (-0.1652, 51.5259),  // Greenford
            (-0.1350, 51.5210),  // Perivale
            (-0.0997, 51.5152),  // Hanger Lane
            (-0.0638, 51.5165),  // North Acton
            (-0.0362, 51.5111),  // East Acton
            (-0.0244, 51.5043),  // White City
            (-0.0048, 51.5035),  // Shepherd's Bush
            (-0.0125, 51.5009),  // Holland Park
            (-0.0199, 51.4996),  // Notting Hill Gate
            (-0.0457, 51.5068),  // Queensway
            (-0.0742, 51.5113),  // Lancaster Gate
            (-0.0983, 51.5142),  // Marble Arch
            (-0.1280, 51.5151),  // Bond Street
            (-0.1410, 51.5154),  // Oxford Circus
            (-0.1687, 51.5174),  // Tottenham Court Road
            (-0.1889, 51.5206),  // Holborn
            (-0.1205, 51.5152),  // Chancery Lane
            (-0.1025, 51.5168),  // St. Paul's
            (-0.0911, 51.5155),  // Bank
            (-0.0765, 51.5108),  // Liverpool Street
        ],
    });
    
    // Northern Line (simplified)
    routes.push(Route {
        id: 1,
        name: "Northern Line".to_string(),
        vehicle_type: VehicleType::Train,
        stations: vec![
            // North to South
            (-0.1938, 51.6503),  // High Barnet
            (-0.1932, 51.6302),  // Totteridge & Whetstone
            (-0.1858, 51.6179),  // Woodside Park
            (-0.1750, 51.6071),  // West Finchley
            (-0.1647, 51.5998),  // Finchley Central
            (-0.1534, 51.5874),  // East Finchley
            (-0.1419, 51.5775),  // Highgate
            (-0.1303, 51.5717),  // Archway
            (-0.1123, 51.5656),  // Tufnell Park
            (-0.1051, 51.5545),  // Kentish Town
            (-0.1426, 51.5302),  // Camden Town
            (-0.1385, 51.5248),  // Mornington Crescent
            (-0.1343, 51.5287),  // Euston
            (-0.1304, 51.5295),  // King's Cross St. Pancras
            (-0.1231, 51.5203),  // Angel
            (-0.1065, 51.5121),  // Old Street
            (-0.0882, 51.5176),  // Moorgate
            (-0.0911, 51.5155),  // Bank
            (-0.0924, 51.5113),  // London Bridge
            (-0.1002, 51.5044),  // Borough
            (-0.1052, 51.4944),  // Elephant & Castle
        ],
    });
    
    // Bus route (sample)
    routes.push(Route {
        id: 2,
        name: "Bus 88".to_string(),
        vehicle_type: VehicleType::Bus,
        stations: vec![
            // West to East (Camden to Canning Town)
            (-0.1465, 51.5365),  // Camden Town
            (-0.1325, 51.5300),  // St Pancras
            (-0.1155, 51.5235),  // Farringdon
            (-0.0958, 51.5181),  // Barbican
            (-0.0879, 51.5155),  // Moorgate
            (-0.0825, 51.5127),  // Liverpool Street
            (-0.0754, 51.5101),  // Aldgate
            (-0.0650, 51.5088),  // Aldgate East
            (-0.0550, 51.5070),  // Whitechapel
            (-0.0449, 51.5055),  // Stepney Green
            (-0.0349, 51.5040),  // Mile End
            (-0.0250, 51.5025),  // Bow Road
            (-0.0150, 51.5010),  // Bow Church
            (-0.0050, 51.4995),  // Devons Road
            (0.0050, 51.4980),   // Langdon Park
            (0.0150, 51.4965),   // All Saints
            (0.0250, 51.4950),   // Poplar
            (0.0350, 51.4935),   // Blackwall
            (0.0450, 51.4920),   // East India
            (0.0550, 51.4905),   // Canning Town
        ],
    });
    
    routes
}

/// Initialize vehicles on the routes
fn initialize_vehicles(routes: &[Route]) -> Vec<Vehicle> {
    let mut vehicles = Vec::new();
    let mut id_counter = 0;
    
    for route in routes {
        let vehicle_count = match route.vehicle_type {
            VehicleType::Train => 10,  // 10 trains per line
            VehicleType::Bus => 20,    // 20 buses per route
        };
        
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
    
    vehicles
}

/// Register vehicle layers with MapLibre GL
fn register_vehicle_layers() {
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
                    'circle-radius': 5,
                    'circle-color': '#0000FF',
                    'circle-stroke-color': '#FFFFFF',
                    'circle-stroke-width': 1
                }
            });
            
            // Add a layer for train vehicles
            window.mapInstance.addLayer({
                id: 'trains-layer',
                type: 'circle',
                source: 'vehicles-source',
                filter: ['==', ['get', 'vehicleType'], 'Train'],
                paint: {
                    'circle-radius': 5,
                    'circle-color': '#FF0000',
                    'circle-stroke-color': '#FFFFFF',
                    'circle-stroke-width': 1
                }
            });
            
            console.log('Vehicle layers registered with MapLibre');
        }
    } else {
        console.error('MapInstance not found!');
    }
    "#;
    
    let _ = js_sys::eval(js_code);
}

/// Start the animation loop for vehicle movement
fn start_animation_loop() {
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
    let window = web_sys::window().expect("no global window exists");
    let id = window.request_animation_frame(animation_callback.as_ref().unchecked_ref())
        .expect("failed to request animation frame");
    
    // Store the callback and frame ID
    SIMULATION_STATE.with(|state| {
        let mut sim_state = state.borrow_mut();
        sim_state.animation_frame_id = Some(id);
    });
    
    // Forget the closure to keep it alive (will be cleaned up when simulation is reset)
    animation_callback.forget();
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
    let window = web_sys::window().expect("no global window exists");
    let id = window.request_animation_frame(animation_callback.as_ref().unchecked_ref())
        .expect("failed to request animation frame");
    
    // Store the animation frame ID
    SIMULATION_STATE.with(|state| {
        let mut sim_state = state.borrow_mut();
        sim_state.animation_frame_id = Some(id);
    });
    
    // Forget the closure to keep it alive
    animation_callback.forget();
}

/// Update positions of all vehicles based on their speed and direction
fn update_vehicle_positions(sim_state: &mut SimulationState) {
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
    // Create GeoJSON for the vehicles
    let features_array = Array::new();
    
    for vehicle in &sim_state.vehicles {
        // Create a feature for this vehicle
        let feature = Object::new();
        
        // Set feature type
        Reflect::set(
            &feature,
            &JsValue::from_str("type"),
            &JsValue::from_str("Feature")
        ).unwrap();
        
        // Set geometry
        let geometry = Object::new();
        Reflect::set(
            &geometry,
            &JsValue::from_str("type"),
            &JsValue::from_str("Point")
        ).unwrap();
        
        let coordinates = Array::new();
        coordinates.push(&JsValue::from_f64(vehicle.lng));
        coordinates.push(&JsValue::from_f64(vehicle.lat));
        
        Reflect::set(
            &geometry,
            &JsValue::from_str("coordinates"),
            &coordinates
        ).unwrap();
        
        Reflect::set(
            &feature,
            &JsValue::from_str("geometry"),
            &geometry
        ).unwrap();
        
        // Set properties
        let properties = Object::new();
        Reflect::set(
            &properties,
            &JsValue::from_str("id"),
            &JsValue::from_f64(vehicle.id as f64)
        ).unwrap();
        
        let vehicle_type_str = match vehicle.vehicle_type {
            VehicleType::Bus => "Bus",
            VehicleType::Train => "Train",
        };
        
        Reflect::set(
            &properties,
            &JsValue::from_str("vehicleType"),
            &JsValue::from_str(vehicle_type_str)
        ).unwrap();
        
        Reflect::set(
            &properties,
            &JsValue::from_str("routeIndex"),
            &JsValue::from_f64(vehicle.route_index as f64)
        ).unwrap();
        
        Reflect::set(
            &feature,
            &JsValue::from_str("properties"),
            &properties
        ).unwrap();
        
        // Add to features array
        features_array.push(&feature);
    }
    
    // Create the GeoJSON object
    let geojson = Object::new();
    Reflect::set(
        &geojson,
        &JsValue::from_str("type"),
        &JsValue::from_str("FeatureCollection")
    ).unwrap();
    
    Reflect::set(
        &geojson,
        &JsValue::from_str("features"),
        &features_array
    ).unwrap();
    
    // Update the source in MapLibre
    let js_code = format!(r#"
    if (window.mapInstance && window.mapInstance.getSource('vehicles-source')) {{
        window.mapInstance.getSource('vehicles-source').setData({});
    }}
    "#, serialize_geojson_data(&geojson));
    
    let _ = js_sys::eval(&js_code);
}

/// Serialize a JS object to a JSON string for embedding in JS code
fn serialize_geojson_data(geojson: &Object) -> String {
    let js_code = r#"
    function serializeToJSON(obj) {
        return JSON.stringify(obj);
    }
    serializeToJSON(arguments[0]);
    "#;
    
    let serialized = js_sys::Function::new_with_args("obj", js_code)
        .call1(&JsValue::NULL, &geojson.into())
        .unwrap();
    
    serialized.as_string().unwrap()
}

/// Toggle the simulation pause state
fn toggle_simulation() {
    let is_now_paused = SIMULATION_STATE.with(|state| {
        let mut sim_state = state.borrow_mut();
        sim_state.is_paused = !sim_state.is_paused;
        sim_state.is_paused
    });
        
    // If we're resuming, restart the animation loop
    if !is_now_paused {
        start_animation_loop();
    }
        
    console::log_1(&format!("Simulation paused: {}", is_now_paused).into());
}

/// Reset the simulation
fn reset_simulation() {
    console::log_1(&"Resetting simulation...".into());
    
    // Cancel current animation frame if one is active
    SIMULATION_STATE.with(|state| {
        let sim_state = state.borrow();
        if let Some(id) = sim_state.animation_frame_id {
            let window = web_sys::window().expect("no global window exists");
            window.cancel_animation_frame(id).expect("failed to cancel animation frame");
        }
    });
    
    // Reset state and recreate everything
    initialize_simulation();
    
    console::log_1(&"Simulation reset complete".into());
}
