// src/tfl/stations.rs
use std::collections::HashSet;

// Station types
#[derive(Clone, Debug, PartialEq)]
pub enum StationType {
    TubeStation,
    BusStop,
    Interchange,
}

// Station structure
#[derive(Clone, Debug)]
pub struct Station {
    pub name: String,
    pub coordinates: (f64, f64), // (longitude, latitude)
    pub tube_lines: HashSet<String>,
    pub bus_routes: HashSet<i32>,
    pub is_terminus: bool,
    pub is_interchange: bool,
    pub station_type: StationType,
}

impl Station {
    // Create a new station
    pub fn new(name: String, coordinates: (f64, f64), station_type: StationType) -> Self {
        Self {
            name,
            coordinates,
            tube_lines: HashSet::new(),
            bus_routes: HashSet::new(),
            is_terminus: false,
            is_interchange: false,
            station_type,
        }
    }
    
    // Add a tube line to the station
    pub fn add_tube_line(&mut self, line_name: &str) {
        self.tube_lines.insert(line_name.to_string());
    }
    
    // Add a bus route to the station
    pub fn add_bus_route(&mut self, route_number: i32) {
        self.bus_routes.insert(route_number);
    }
    
    // Check if this is an interchange station (serves multiple lines)
    pub fn is_interchange(&self) -> bool {
        (self.tube_lines.len() + self.bus_routes.len()) > 1
    }
}