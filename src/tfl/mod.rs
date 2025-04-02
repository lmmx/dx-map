// src/tfl/mod.rs
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use js_sys::{Array, Object, Reflect};
use web_sys::console;

mod routes;
mod stations;
mod vehicles;

pub use routes::{TubeRoute, BusRoute, parse_tube_routes, parse_bus_routes};
pub use stations::{Station, StationType};
pub use vehicles::{Vehicle, VehicleType, SimulationController};

// Color definitions for TfL lines
pub const TFL_COLORS: [(&str, &str); 11] = [
    ("Bakerloo Line", "#B36305"),
    ("Central Line", "#E32017"),
    ("Circle Line", "#FFD300"),
    ("District Line", "#00782A"),
    ("Hammersmith & City Line", "#F3A9BB"),
    ("Jubilee Line", "#A0A5A9"),
    ("Metropolitan Line", "#9B0056"),
    ("Northern Line", "#000000"),
    ("Piccadilly Line", "#003688"),
    ("Victoria Line", "#0098D4"),
    ("Waterloo & City Line", "#95CDBA"),
];

// Main TfL data manager 
pub struct TflDataManager {
    tube_routes: Vec<TubeRoute>,
    bus_routes: Vec<BusRoute>,
    stations: Vec<Station>,
    vehicles: Vec<Vehicle>,
    simulation_controller: Option<SimulationController>,
}

impl TflDataManager {
    pub fn new() -> Self {
        Self {
            tube_routes: Vec::new(),
            bus_routes: Vec::new(),
            stations: Vec::new(),
            vehicles: Vec::new(),
            simulation_controller: None,
        }
    }

    // Load data from TSV files
    pub async fn load_data(&mut self) -> Result<(), JsValue> {
        console::log_1(&"Loading TfL data...".into());

        // Load tube routes
        let tube_routes_data = self.load_tsv_file("tube_routes.tsv").await?;
        self.tube_routes = parse_tube_routes(&tube_routes_data);
        console::log_1(&format!("Loaded {} tube routes", self.tube_routes.len()).into());

        // Load bus routes
        let bus_routes_data = self.load_tsv_file("bus_routes.tsv").await?;
        self.bus_routes = parse_bus_routes(&bus_routes_data);
        console::log_1(&format!("Loaded {} bus routes", self.bus_routes.len()).into());

        // Process stations from both tube and bus data
        self.process_stations();
        console::log_1(&format!("Processed {} stations", self.stations.len()).into());

        // Create vehicles for simulation
        self.initialize_vehicles();
        console::log_1(&format!("Created {} vehicles for simulation", self.vehicles.len()).into());

        Ok(())
    }

    // Helper to load TSV file content
    async fn load_tsv_file(&self, filename: &str) -> Result<String, JsValue> {
        let window = web_sys::window().expect("no global window exists");
        let fs = Reflect::get(&window, &JsValue::from_str("fs"))?;
        let read_file = Reflect::get(&fs, &JsValue::from_str("readFile"))?;
        
        let filename_js = JsValue::from_str(filename);
        let options_obj = Object::new();
        Reflect::set(
            &options_obj, 
            &JsValue::from_str("encoding"),
            &JsValue::from_str("utf8")
        )?;
        
        let args = Array::new();
        args.push(&filename_js);
        args.push(&options_obj);
        
        let promise = js_sys::Function::new_with_args("filename, options", 
            "return window.fs.readFile(filename, options);")
            .call2(&JsValue::null(), &filename_js, &options_obj)?;
        
        let promise_obj = js_sys::Promise::resolve(&promise);
        let file_content = wasm_bindgen_futures::JsFuture::from(promise_obj).await?;
        let content = file_content.as_string().ok_or_else(|| {
            JsValue::from_str(&format!("Could not convert file {} to string", filename))
        })?;
        
        Ok(content)
    }

    // Process stations from route data 
    fn process_stations(&mut self) {
        let mut station_map: HashMap<String, Station> = HashMap::new();
        
        // Add tube termini
        for route in &self.tube_routes {
            // Add start terminus
            let station_name = route.start_terminus.clone();
            let coords = route.start_coordinates;
            
            if let Some(station) = station_map.get_mut(&station_name) {
                station.add_tube_line(&route.line_name);
            } else {
                let mut station = Station::new(
                    station_name.clone(), 
                    coords, 
                    StationType::TubeStation
                );
                station.add_tube_line(&route.line_name);
                station.is_terminus = true;
                station_map.insert(station_name, station);
            }
            
            // Add end terminus
            let station_name = route.end_terminus.clone();
            let coords = route.end_coordinates;
            
            if let Some(station) = station_map.get_mut(&station_name) {
                station.add_tube_line(&route.line_name);
            } else {
                let mut station = Station::new(
                    station_name.clone(), 
                    coords, 
                    StationType::TubeStation
                );
                station.add_tube_line(&route.line_name);
                station.is_terminus = true;
                station_map.insert(station_name, station);
            }
        }
        
        // Add bus termini
        for route in &self.bus_routes {
            // Add start terminus
            let station_name = route.start_terminus.clone();
            let coords = route.start_coordinates;
            
            if let Some(station) = station_map.get_mut(&station_name) {
                station.add_bus_route(route.route_number);
            } else {
                let mut station = Station::new(
                    station_name.clone(), 
                    coords, 
                    StationType::BusStop
                );
                station.add_bus_route(route.route_number);
                station.is_terminus = true;
                station_map.insert(station_name, station);
            }
            
            // Add end terminus
            let station_name = route.end_terminus.clone();
            let coords = route.end_coordinates;
            
            if let Some(station) = station_map.get_mut(&station_name) {
                station.add_bus_route(route.route_number);
            } else {
                let mut station = Station::new(
                    station_name.clone(), 
                    coords, 
                    StationType::BusStop
                );
                station.add_bus_route(route.route_number);
                station.is_terminus = true;
                station_map.insert(station_name, station);
            }
        }
        
        // Identify interchanges (stations with multiple lines or routes)
        for (_, station) in station_map.iter_mut() {
            let tube_count = station.tube_lines.len();
            let bus_count = station.bus_routes.len();
            
            if tube_count + bus_count > 1 {
                station.is_interchange = true;
            }
            
            if tube_count > 0 && bus_count > 0 {
                station.station_type = StationType::Interchange;
            }
        }
        
        self.stations = station_map.into_values().collect();
    }

    // Initialize vehicles for simulation
    fn initialize_vehicles(&mut self) {
        // Create tube trains
        let mut vehicle_id = 0;
        
        for route in &self.tube_routes {
            // Create 3-5 trains per line based on the line length
            let train_count = 3 + (js_sys::Math::random() * 3.0) as usize;
            
            for _ in 0..train_count {
                let vehicle = Vehicle::new(
                    vehicle_id,
                    VehicleType::Train,
                    route.line_name.clone(),
                    route.start_coordinates,
                    route.end_coordinates,
                );
                
                self.vehicles.push(vehicle);
                vehicle_id += 1;
            }
        }
        
        // Create buses
        for route in &self.bus_routes {
            // Create 1-3 buses per route
            let bus_count = 1 + (js_sys::Math::random() * 3.0) as usize;
            
            for _ in 0..bus_count {
                let vehicle = Vehicle::new(
                    vehicle_id,
                    VehicleType::Bus,
                    format!("Bus {}", route.route_number),
                    route.start_coordinates,
                    route.end_coordinates,
                );
                
                self.vehicles.push(vehicle);
                vehicle_id += 1;
            }
        }
        
        // Initialize simulation controller
        self.simulation_controller = Some(SimulationController::new(self.vehicles.clone()));
    }

    // Create GeoJSON for routes
    pub fn create_tube_lines_geojson(&self) -> Result<JsValue, JsValue> {
        let features = Array::new();
        
        for route in &self.tube_routes {
            let feature = Object::new();
            Reflect::set(&feature, &JsValue::from_str("type"), &JsValue::from_str("Feature"))?;
            
            // Properties
            let properties = Object::new();
            Reflect::set(
                &properties, 
                &JsValue::from_str("name"), 
                &JsValue::from_str(&route.line_name)
            )?;
            
            Reflect::set(
                &properties, 
                &JsValue::from_str("color"), 
                &JsValue::from_str(self.get_line_color(&route.line_name))
            )?;
            
            Reflect::set(&feature, &JsValue::from_str("properties"), &properties)?;
            
            // Geometry
            let geometry = Object::new();
            Reflect::set(
                &geometry, 
                &JsValue::from_str("type"), 
                &JsValue::from_str("LineString")
            )?;
            
            let coordinates = Array::new();
            
            // For now we just connect start to end directly
            // In a real app, you'd have intermediate points
            let start_coords = Array::new();
            start_coords.push(&JsValue::from_f64(route.start_coordinates.0));
            start_coords.push(&JsValue::from_f64(route.start_coordinates.1));
            coordinates.push(&start_coords);
            
            let end_coords = Array::new();
            end_coords.push(&JsValue::from_f64(route.end_coordinates.0));
            end_coords.push(&JsValue::from_f64(route.end_coordinates.1));
            coordinates.push(&end_coords);
            
            Reflect::set(&geometry, &JsValue::from_str("coordinates"), &coordinates)?;
            Reflect::set(&feature, &JsValue::from_str("geometry"), &geometry)?;
            
            features.push(&feature);
        }
        
        // Create the GeoJSON object
        let geojson = Object::new();
        Reflect::set(&geojson, &JsValue::from_str("type"), &JsValue::from_str("FeatureCollection"))?;
        Reflect::set(&geojson, &JsValue::from_str("features"), &features)?;
        
        Ok(geojson.into())
    }

    // Create GeoJSON for bus routes
    pub fn create_bus_routes_geojson(&self) -> Result<JsValue, JsValue> {
        let features = Array::new();
        
        for route in &self.bus_routes {
            let feature = Object::new();
            Reflect::set(&feature, &JsValue::from_str("type"), &JsValue::from_str("Feature"))?;
            
            // Properties
            let properties = Object::new();
            Reflect::set(
                &properties, 
                &JsValue::from_str("routeNumber"), 
                &JsValue::from_f64(route.route_number as f64)
            )?;
            
            Reflect::set(
                &properties, 
                &JsValue::from_str("startTerminus"), 
                &JsValue::from_str(&route.start_terminus)
            )?;
            
            Reflect::set(
                &properties, 
                &JsValue::from_str("endTerminus"), 
                &JsValue::from_str(&route.end_terminus)
            )?;
            
            Reflect::set(&feature, &JsValue::from_str("properties"), &properties)?;
            
            // Geometry
            let geometry = Object::new();
            Reflect::set(
                &geometry, 
                &JsValue::from_str("type"), 
                &JsValue::from_str("LineString")
            )?;
            
            let coordinates = Array::new();
            
            // For now we just connect start to end directly
            let start_coords = Array::new();
            start_coords.push(&JsValue::from_f64(route.start_coordinates.0));
            start_coords.push(&JsValue::from_f64(route.start_coordinates.1));
            coordinates.push(&start_coords);
            
            let end_coords = Array::new();
            end_coords.push(&JsValue::from_f64(route.end_coordinates.0));
            end_coords.push(&JsValue::from_f64(route.end_coordinates.1));
            coordinates.push(&end_coords);
            
            Reflect::set(&geometry, &JsValue::from_str("coordinates"), &coordinates)?;
            Reflect::set(&feature, &JsValue::from_str("geometry"), &geometry)?;
            
            features.push(&feature);
        }
        
        // Create the GeoJSON object
        let geojson = Object::new();
        Reflect::set(&geojson, &JsValue::from_str("type"), &JsValue::from_str("FeatureCollection"))?;
        Reflect::set(&geojson, &JsValue::from_str("features"), &features)?;
        
        Ok(geojson.into())
    }

    // Create GeoJSON for stations
    pub fn create_stations_geojson(&self) -> Result<JsValue, JsValue> {
        let features = Array::new();
        
        for station in &self.stations {
            let feature = Object::new();
            Reflect::set(&feature, &JsValue::from_str("type"), &JsValue::from_str("Feature"))?;
            
            // Properties
            let properties = Object::new();
            Reflect::set(
                &properties, 
                &JsValue::from_str("name"), 
                &JsValue::from_str(&station.name)
            )?;
            
            Reflect::set(
                &properties, 
                &JsValue::from_str("isTerminus"), 
                &JsValue::from_bool(station.is_terminus)
            )?;
            
            Reflect::set(
                &properties, 
                &JsValue::from_str("isInterchange"), 
                &JsValue::from_bool(station.is_interchange)
            )?;
            
            // Add tube lines as array
            let tube_lines = Array::new();
            for line in &station.tube_lines {
                tube_lines.push(&JsValue::from_str(line));
            }
            Reflect::set(&properties, &JsValue::from_str("tubeLines"), &tube_lines)?;
            
            // Add bus routes as array
            let bus_routes = Array::new();
            for route in &station.bus_routes {
                bus_routes.push(&JsValue::from_f64(*route as f64));
            }
            Reflect::set(&properties, &JsValue::from_str("busRoutes"), &bus_routes)?;
            
            // Add station type
            Reflect::set(
                &properties, 
                &JsValue::from_str("stationType"), 
                &JsValue::from_str(match station.station_type {
                    StationType::TubeStation => "tube",
                    StationType::BusStop => "bus",
                    StationType::Interchange => "interchange"
                })
            )?;
            
            Reflect::set(&feature, &JsValue::from_str("properties"), &properties)?;
            
            // Geometry
            let geometry = Object::new();
            Reflect::set(
                &geometry, 
                &JsValue::from_str("type"), 
                &JsValue::from_str("Point")
            )?;
            
            let coordinates = Array::new();
            coordinates.push(&JsValue::from_f64(station.coordinates.0));
            coordinates.push(&JsValue::from_f64(station.coordinates.1));
            
            Reflect::set(&geometry, &JsValue::from_str("coordinates"), &coordinates)?;
            Reflect::set(&feature, &JsValue::from_str("geometry"), &geometry)?;
            
            features.push(&feature);
        }
        
        // Create the GeoJSON object
        let geojson = Object::new();
        Reflect::set(&geojson, &JsValue::from_str("type"), &JsValue::from_str("FeatureCollection"))?;
        Reflect::set(&geojson, &JsValue::from_str("features"), &features)?;
        
        Ok(geojson.into())
    }

    // Get TfL color for a tube line
    fn get_line_color(&self, line_name: &str) -> &str {
        for (name, color) in TFL_COLORS.iter() {
            if line_name == *name {
                return color;
            }
        }
        
        // Default color
        "#888888"
    }

    // Start the simulation
    pub fn start_simulation(&self) -> Result<(), JsValue> {
        if let Some(sim_controller) = &self.simulation_controller {
            sim_controller.start()?;
            console::log_1(&"Simulation started".into());
        } else {
            console::error_1(&"Simulation controller not initialized".into());
        }
        
        Ok(())
    }

    // Pause the simulation
    pub fn pause_simulation(&self) -> Result<(), JsValue> {
        if let Some(sim_controller) = &self.simulation_controller {
            sim_controller.pause()?;
            console::log_1(&"Simulation paused".into());
        } else {
            console::error_1(&"Simulation controller not initialized".into());
        }
        
        Ok(())
    }

    // Reset the simulation
    pub fn reset_simulation(&self) -> Result<(), JsValue> {
        if let Some(sim_controller) = &self.simulation_controller {
            sim_controller.reset()?;
            console::log_1(&"Simulation reset".into());
        } else {
            console::error_1(&"Simulation controller not initialized".into());
        }
        
        Ok(())
    }
}
