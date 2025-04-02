// src/tfl/routes.rs
use web_sys::console;

// Structure for tube routes
#[derive(Clone, Debug)]
pub struct TubeRoute {
    pub line_name: String,
    pub start_terminus: String,
    pub start_coordinates: (f64, f64), // (longitude, latitude)
    pub end_terminus: String,
    pub end_coordinates: (f64, f64),  // (longitude, latitude)
}

// Structure for bus routes
#[derive(Clone, Debug)]
pub struct BusRoute {
    pub route_number: i32,
    pub start_terminus: String,
    pub start_coordinates: (f64, f64), // (longitude, latitude)
    pub end_terminus: String,
    pub end_coordinates: (f64, f64),  // (longitude, latitude)
}

// Parse tube routes from TSV content
pub fn parse_tube_routes(tsv_content: &str) -> Vec<TubeRoute> {
    let mut routes = Vec::new();
    let mut lines = tsv_content.lines();
    
    // Skip header
    if let Some(_) = lines.next() {
        for line in lines {
            if line.trim().is_empty() {
                continue;
            }
            
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() >= 5 {
                let line_name = fields[0].trim().to_string();
                let start_terminus = fields[1].trim().to_string();
                let start_coords = parse_coordinates(fields[2]);
                let end_terminus = fields[3].trim().to_string();
                let end_coords = parse_coordinates(fields[4]);
                
                if let (Some(start_coords), Some(end_coords)) = (start_coords, end_coords) {
                    routes.push(TubeRoute {
                        line_name,
                        start_terminus,
                        start_coordinates: start_coords,
                        end_terminus,
                        end_coordinates: end_coords,
                    });
                } else {
                    console::warn_1(&format!("Skipping tube route due to invalid coordinates: {}", line).into());
                }
            } else {
                console::warn_1(&format!("Skipping tube route due to invalid format: {}", line).into());
            }
        }
    }
    
    routes
}

// Parse bus routes from TSV content
pub fn parse_bus_routes(tsv_content: &str) -> Vec<BusRoute> {
    let mut routes = Vec::new();
    let mut lines = tsv_content.lines();
    
    // Skip header
    if let Some(_) = lines.next() {
        for line in lines {
            if line.trim().is_empty() {
                continue;
            }
            
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() >= 5 {
                if let Ok(route_number) = fields[0].trim().parse::<i32>() {
                    let start_terminus = fields[1].trim().to_string();
                    let start_coords = parse_coordinates(fields[2]);
                    let end_terminus = fields[3].trim().to_string();
                    let end_coords = parse_coordinates(fields[4]);
                    
                    if let (Some(start_coords), Some(end_coords)) = (start_coords, end_coords) {
                        routes.push(BusRoute {
                            route_number,
                            start_terminus,
                            start_coordinates: start_coords,
                            end_terminus,
                            end_coordinates: end_coords,
                        });
                    } else {
                        console::warn_1(&format!("Skipping bus route due to invalid coordinates: {}", line).into());
                    }
                } else {
                    console::warn_1(&format!("Skipping bus route due to invalid route number: {}", line).into());
                }
            } else {
                console::warn_1(&format!("Skipping bus route due to invalid format: {}", line).into());
            }
        }
    }
    
    routes
}

// Helper to parse coordinates from string (format: "51.1234,-0.1234")
fn parse_coordinates(coords_str: &str) -> Option<(f64, f64)> {
    let parts: Vec<&str> = coords_str.split(',').collect();
    if parts.len() == 2 {
        if let (Ok(lat), Ok(lng)) = (parts[0].trim().parse::<f64>(), parts[1].trim().parse::<f64>()) {
            // Return as (longitude, latitude) for GeoJSON compatibility
            return Some((lng, lat));
        }
    }
    
    None
}