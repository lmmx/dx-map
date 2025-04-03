use js_sys::Math;

#[derive(Clone, Debug)]
pub enum VehicleType {
    Bus,
    Train,
}

#[derive(Clone, Debug)]
pub struct Vehicle {
    pub id: usize,
    pub vehicle_type: VehicleType,
    pub route_index: usize,
    pub position: f64,       // 0.0 to 1.0 position along route segment
    pub speed: f64,          // Movement speed
    pub direction: i8,       // 1 = forward, -1 = backward
    pub last_station: usize, // Index of last station
    pub next_station: usize, // Index of station we're heading towards
    pub lng: f64,            // Current longitude
    pub lat: f64,            // Current latitude
}

#[derive(Clone, Debug)]
pub struct Route {
    pub id: usize,
    pub name: String,
    pub vehicle_type: VehicleType,
    pub stations: Vec<(f64, f64)>, // Vec of (lng, lat) coordinates
}

/// Create sample routes based on TfL network
pub fn build_sample_routes() -> Vec<Route> {
    // Create a vector to hold the routes
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

    routes
}

/// Initialize vehicles on the routes
pub fn initialize_vehicles(routes: &[Route]) -> Vec<Vehicle> {
    let mut vehicles = Vec::new();
    let mut id_counter = 0;

    for route in routes {
        let vehicle_count = match route.vehicle_type {
            VehicleType::Train => 10, // 10 trains per line
            VehicleType::Bus => 20,   // 20 buses per route
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