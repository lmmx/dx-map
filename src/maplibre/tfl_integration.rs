// src/maplibre/tfl_integration.rs - Separate module for TfL data integration
use crate::maplibre::bindings::*;
use js_sys::{Array, Function, Object, Promise, Reflect};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use web_sys::console;

// Create a new struct for handling TfL data specifically
pub struct TflMapIntegration {
    // Store references to map layers for easy access
    tube_lines_source_id: String,
    bus_routes_source_id: String,
    stations_source_id: String,
    vehicles_source_id: String,
    simulation_active: bool,
}

impl TflMapIntegration {
    // Create a new TfL integration
    pub fn new() -> Self {
        Self {
            tube_lines_source_id: "tfl-tube-lines".to_string(),
            bus_routes_source_id: "tfl-bus-routes".to_string(),
            stations_source_id: "tfl-stations".to_string(),
            vehicles_source_id: "tfl-vehicles".to_string(),
            simulation_active: false,
        }
    }

    // Initialize the TfL data on the map
    pub fn initialize(
        &mut self,
        map_instance: &Map,
        simulation_enabled: bool,
    ) -> Result<(), JsValue> {
        console::log_1(&"Initializing TfL data on map...".into());

        // Load the data and create the visualizations
        self.load_tfl_data(map_instance, simulation_enabled)?;

        console::log_1(&"TfL data initialized successfully".into());
        Ok(())
    }

    // Load TfL data from TSV files asynchronously
    fn load_tfl_data(&mut self, map: &Map, simulation_enabled: bool) -> Result<(), JsValue> {
        console::log_1(&"Loading TfL data from TSV files...".into());

        // Setup tube lines from tube_routes.tsv
        self.setup_tube_lines(map)?;

        // Setup bus routes from bus_routes.tsv
        self.setup_bus_routes(map)?;

        // Setup stations from both files
        self.setup_stations(map)?;

        // Setup vehicle simulation if enabled
        if simulation_enabled {
            self.setup_simulation(map)?;
            self.simulation_active = true;
        }

        console::log_1(&"TfL data loaded successfully".into());
        Ok(())
    }

    // Parse TSV file asynchronously
    fn parse_tsv_file(&self, filename: &str) -> Result<Promise, JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window found"))?;

        // Create a promise to load and parse the file
        let promise = Promise::new(&mut |resolve, reject| {
            let filename_js = JsValue::from_str(filename);

            // Function to handle async file read
            let js_code = format!(
                r#"
                async function loadTsvFile(filename) {{
                    try {{
                        const content = await window.fs.readFile(filename, {{ encoding: 'utf8' }});

                        // Basic TSV parser
                        const lines = content.split('\n');
                        const headers = lines[0].split('\t');

                        const results = [];
                        for (let i = 1; i < lines.length; i++) {{
                            if (!lines[i].trim()) continue;

                            const fields = lines[i].split('\t');
                            const record = {{}};

                            for (let j = 0; j < headers.length; j++) {{
                                record[headers[j]] = fields[j];
                            }}

                            results.push(record);
                        }}

                        return results;
                    }} catch (err) {{
                        throw new Error(`Failed to load TSV file: ${{err.message}}`);
                    }}
                }}

                loadTsvFile('{}').then(resolve).catch(reject);
                "#,
                filename
            );

            // Execute the JS code
            if let Err(e) = js_sys::eval(&js_code) {
                reject
                    .call1(
                        &JsValue::NULL,
                        &JsValue::from_str(&format!("Failed to execute JS: {:?}", e)),
                    )
                    .unwrap();
            }
        });

        Ok(promise)
    }

    // Setup tube lines visualization
    fn setup_tube_lines(&self, map: &Map) -> Result<(), JsValue> {
        console::log_1(&"Setting up tube lines...".into());

        // Function to process tube routes data
        let js_code = r#"
        async function setupTubeLines() {
            try {
                // Load the TSV file
                const content = await window.fs.readFile('tube_routes.tsv', { encoding: 'utf8' });

                // Parse TSV into JSON
                const lines = content.split('\n');
                const headers = lines[0].split('\t');

                const tubeRoutes = [];
                for (let i = 1; i < lines.length; i++) {
                    if (!lines[i].trim()) continue;

                    const fields = lines[i].split('\t');
                    const record = {};

                    for (let j = 0; j < headers.length; j++) {
                        record[headers[j]] = fields[j];
                    }

                    tubeRoutes.push(record);
                }

                // Create GeoJSON for tube lines
                const features = tubeRoutes.map(route => {
                    // Parse coordinates
                    const startCoords = route['Start Coordinates'].split(',').map(c => parseFloat(c));
                    const endCoords = route['End Coordinates'].split(',').map(c => parseFloat(c));

                    // Get line color
                    const lineColor = getTubeLineColor(route['Tube Line']);

                    return {
                        type: 'Feature',
                        properties: {
                            name: route['Tube Line'],
                            color: lineColor,
                            startTerminus: route['Start Terminus'],
                            endTerminus: route['End Terminus']
                        },
                        geometry: {
                            type: 'LineString',
                            coordinates: [
                                [startCoords[1], startCoords[0]], // [lng, lat]
                                [endCoords[1], endCoords[0]]      // [lng, lat]
                            ]
                        }
                    };
                });

                // Return the GeoJSON
                return {
                    type: 'FeatureCollection',
                    features: features
                };
            } catch (err) {
                console.error('Error loading tube routes:', err);
                throw err;
            }
        }

        // Helper function to get TfL colors
        function getTubeLineColor(lineName) {
            const colors = {
                'Bakerloo Line': '#B36305',
                'Central Line': '#E32017',
                'Circle Line': '#FFD300',
                'District Line': '#00782A',
                'Hammersmith & City Line': '#F3A9BB',
                'Jubilee Line': '#A0A5A9',
                'Metropolitan Line': '#9B0056',
                'Northern Line': '#000000',
                'Piccadilly Line': '#003688',
                'Victoria Line': '#0098D4',
                'Waterloo & City Line': '#95CDBA'
            };

            return colors[lineName] || '#888888';
        }

        // Run the setup
        setupTubeLines();
        "#;

        // Execute the JS code to get tube lines data
        let tube_lines_promise = Promise::new(&mut |resolve, reject| {
            let run_js = format!(
                r#"
                {js_code}
                setupTubeLines().then(resolve).catch(reject);
                "#
            );

            if let Err(e) = js_sys::eval(&run_js) {
                reject
                    .call1(
                        &JsValue::NULL,
                        &JsValue::from_str(&format!("Failed to setup tube lines: {:?}", e)),
                    )
                    .unwrap();
            }
        });

        // Function to wait for promise and add layers
        let wait_for_promise = format!(
            r#"
            async function addTubeLinesToMap() {{
                try {{
                    const geojson = await arguments[0];

                    // Add source if it doesn't exist
                    if (!window.mapInstance.getSource('{}')) {{
                        window.mapInstance.addSource('{}', {{
                            type: 'geojson',
                            data: geojson
                        }});

                        // Add tube lines layer
                        window.mapInstance.addLayer({{
                            id: '{}-layer',
                            type: 'line',
                            source: '{}',
                            layout: {{
                                'line-join': 'round',
                                'line-cap': 'round'
                            }},
                            paint: {{
                                'line-color': ['get', 'color'],
                                'line-width': 4
                            }}
                        }});

                        console.log('Added tube lines to map');
                    }}

                    return true;
                }} catch (err) {{
                    console.error('Error adding tube lines to map:', err);
                    throw err;
                }}
            }}

            addTubeLinesToMap();
            "#,
            self.tube_lines_source_id,
            self.tube_lines_source_id,
            self.tube_lines_source_id,
            self.tube_lines_source_id
        );

        // Execute the wait function
        let result = Function::new_with_args("tubeLines", &wait_for_promise)
            .call1(&JsValue::NULL, &tube_lines_promise)?;

        // Check if we got an error
        if result.is_instance_of::<js_sys::Error>() {
            return Err(JsValue::from_str(&format!(
                "Error setting up tube lines: {:?}",
                result
            )));
        }

        console::log_1(&"Tube lines setup successfully".into());
        Ok(())
    }

    // Setup bus routes visualization
    fn setup_bus_routes(&self, map: &Map) -> Result<(), JsValue> {
        console::log_1(&"Setting up bus routes...".into());

        // Function to process bus routes data
        let js_code = r#"
        async function setupBusRoutes() {
            try {
                // Load the TSV file
                const content = await window.fs.readFile('bus_routes.tsv', { encoding: 'utf8' });

                // Parse TSV into JSON
                const lines = content.split('\n');
                const headers = lines[0].split('\t');

                const busRoutes = [];
                for (let i = 1; i < lines.length; i++) {
                    if (!lines[i].trim()) continue;

                    const fields = lines[i].split('\t');
                    const record = {};

                    for (let j = 0; j < headers.length; j++) {
                        record[headers[j]] = fields[j];
                    }

                    busRoutes.push(record);
                }

                // Create GeoJSON for bus routes
                const features = busRoutes.map(route => {
                    // Parse coordinates
                    const startCoords = route['Start Coordinates'].split(',').map(c => parseFloat(c));
                    const endCoords = route['End Coordinates'].split(',').map(c => parseFloat(c));

                    return {
                        type: 'Feature',
                        properties: {
                            routeNumber: route['Route Number'],
                            startTerminus: route['Start Terminus'],
                            endTerminus: route['End Terminus']
                        },
                        geometry: {
                            type: 'LineString',
                            coordinates: [
                                [startCoords[1], startCoords[0]], // [lng, lat]
                                [endCoords[1], endCoords[0]]      // [lng, lat]
                            ]
                        }
                    };
                });

                // Return the GeoJSON
                return {
                    type: 'FeatureCollection',
                    features: features
                };
            } catch (err) {
                console.error('Error loading bus routes:', err);
                throw err;
            }
        }

        // Run the setup
        setupBusRoutes();
        "#;

        // Execute the JS code to get bus routes data
        let bus_routes_promise = Promise::new(&mut |resolve, reject| {
            let run_js = format!(
                r#"
                {js_code}
                setupBusRoutes().then(resolve).catch(reject);
                "#
            );

            if let Err(e) = js_sys::eval(&run_js) {
                reject
                    .call1(
                        &JsValue::NULL,
                        &JsValue::from_str(&format!("Failed to setup bus routes: {:?}", e)),
                    )
                    .unwrap();
            }
        });

        // Function to wait for promise and add layers
        let wait_for_promise = format!(
            r#"
            async function addBusRoutesToMap() {{
                try {{
                    const geojson = await arguments[0];

                    // Add source if it doesn't exist
                    if (!window.mapInstance.getSource('{}')) {{
                        window.mapInstance.addSource('{}', {{
                            type: 'geojson',
                            data: geojson
                        }});

                        // Add bus routes layer
                        window.mapInstance.addLayer({{
                            id: '{}-layer',
                            type: 'line',
                            source: '{}',
                            layout: {{
                                'line-join': 'round',
                                'line-cap': 'round',
                                'visibility': 'visible'
                            }},
                            paint: {{
                                'line-color': '#EE7C0E',
                                'line-width': 2,
                                'line-opacity': 0.7
                            }}
                        }});

                        console.log('Added bus routes to map');
                    }}

                    return true;
                }} catch (err) {{
                    console.error('Error adding bus routes to map:', err);
                    throw err;
                }}
            }}

            addBusRoutesToMap();
            "#,
            self.bus_routes_source_id,
            self.bus_routes_source_id,
            self.bus_routes_source_id,
            self.bus_routes_source_id
        );

        // Execute the wait function
        let result = Function::new_with_args("busRoutes", &wait_for_promise)
            .call1(&JsValue::NULL, &bus_routes_promise)?;

        // Check if we got an error
        if result.is_instance_of::<js_sys::Error>() {
            return Err(JsValue::from_str(&format!(
                "Error setting up bus routes: {:?}",
                result
            )));
        }

        console::log_1(&"Bus routes setup successfully".into());
        Ok(())
    }

    // Setup stations visualization
    fn setup_stations(&self, map: &Map) -> Result<(), JsValue> {
        console::log_1(&"Setting up stations...".into());

        // Function to process stations data
        let js_code = r#"
        async function setupStations() {
            try {
                // Load both TSV files
                const tubeContent = await window.fs.readFile('tube_routes.tsv', { encoding: 'utf8' });
                const busContent = await window.fs.readFile('bus_routes.tsv', { encoding: 'utf8' });

                // Parse tube TSV
                const tubeLines = tubeContent.split('\n');
                const tubeHeaders = tubeLines[0].split('\t');

                const tubeRoutes = [];
                for (let i = 1; i < tubeLines.length; i++) {
                    if (!tubeLines[i].trim()) continue;

                    const fields = tubeLines[i].split('\t');
                    const record = {};

                    for (let j = 0; j < tubeHeaders.length; j++) {
                        record[tubeHeaders[j]] = fields[j];
                    }

                    tubeRoutes.push(record);
                }

                // Parse bus TSV
                const busLines = busContent.split('\n');
                const busHeaders = busLines[0].split('\t');

                const busRoutes = [];
                for (let i = 1; i < busLines.length; i++) {
                    if (!busLines[i].trim()) continue;

                    const fields = busLines[i].split('\t');
                    const record = {};

                    for (let j = 0; j < busHeaders.length; j++) {
                        record[busHeaders[j]] = fields[j];
                    }

                    busRoutes.push(record);
                }

                // Collect all stations
                const stationsMap = new Map();

                // Add tube stations
                tubeRoutes.forEach(route => {
                    // Add start terminus
                    const startCoords = route['Start Coordinates'].split(',').map(c => parseFloat(c));
                    const startName = route['Start Terminus'];

                    if (!stationsMap.has(startName)) {
                        stationsMap.set(startName, {
                            name: startName,
                            coordinates: [startCoords[1], startCoords[0]], // [lng, lat]
                            tubeLines: [route['Tube Line']],
                            busRoutes: [],
                            isTerminus: true,
                            stationType: 'tube'
                        });
                    } else {
                        const station = stationsMap.get(startName);
                        if (!station.tubeLines.includes(route['Tube Line'])) {
                            station.tubeLines.push(route['Tube Line']);
                        }
                    }

                    // Add end terminus
                    const endCoords = route['End Coordinates'].split(',').map(c => parseFloat(c));
                    const endName = route['End Terminus'];

                    if (!stationsMap.has(endName)) {
                        stationsMap.set(endName, {
                            name: endName,
                            coordinates: [endCoords[1], endCoords[0]], // [lng, lat]
                            tubeLines: [route['Tube Line']],
                            busRoutes: [],
                            isTerminus: true,
                            stationType: 'tube'
                        });
                    } else {
                        const station = stationsMap.get(endName);
                        if (!station.tubeLines.includes(route['Tube Line'])) {
                            station.tubeLines.push(route['Tube Line']);
                        }
                    }
                });

                // Add bus stations
                busRoutes.forEach(route => {
                    // Add start terminus
                    const startCoords = route['Start Coordinates'].split(',').map(c => parseFloat(c));
                    const startName = route['Start Terminus'];

                    if (!stationsMap.has(startName)) {
                        stationsMap.set(startName, {
                            name: startName,
                            coordinates: [startCoords[1], startCoords[0]], // [lng, lat]
                            tubeLines: [],
                            busRoutes: [route['Route Number']],
                            isTerminus: true,
                            stationType: 'bus'
                        });
                    } else {
                        const station = stationsMap.get(startName);
                        if (!station.busRoutes.includes(route['Route Number'])) {
                            station.busRoutes.push(route['Route Number']);
                        }
                    }

                    // Add end terminus
                    const endCoords = route['End Coordinates'].split(',').map(c => parseFloat(c));
                    const endName = route['End Terminus'];

                    if (!stationsMap.has(endName)) {
                        stationsMap.set(endName, {
                            name: endName,
                            coordinates: [endCoords[1], endCoords[0]], // [lng, lat]
                            tubeLines: [],
                            busRoutes: [route['Route Number']],
                            isTerminus: true,
                            stationType: 'bus'
                        });
                    } else {
                        const station = stationsMap.get(endName);
                        if (!station.busRoutes.includes(route['Route Number'])) {
                            station.busRoutes.push(route['Route Number']);
                        }
                    }
                });

                // Identify interchanges
                stationsMap.forEach((station, name) => {
                    const totalRoutes = station.tubeLines.length + station.busRoutes.length;
                    station.isInterchange = totalRoutes > 1;

                    // If a station serves both tube and bus, mark as interchange
                    if (station.tubeLines.length > 0 && station.busRoutes.length > 0) {
                        station.stationType = 'interchange';
                    }
                });

                // Create GeoJSON features
                const features = Array.from(stationsMap.values()).map(station => {
                    return {
                        type: 'Feature',
                        properties: {
                            name: station.name,
                            tubeLines: station.tubeLines,
                            busRoutes: station.busRoutes,
                            isTerminus: station.isTerminus,
                            isInterchange: station.isInterchange,
                            stationType: station.stationType
                        },
                        geometry: {
                            type: 'Point',
                            coordinates: station.coordinates
                        }
                    };
                });

                // Return the GeoJSON
                return {
                    type: 'FeatureCollection',
                    features: features
                };
            } catch (err) {
                console.error('Error setting up stations:', err);
                throw err;
            }
        }

        // Run the setup
        setupStations();
        "#;

        // Execute the JS code to get stations data
        let stations_promise = Promise::new(&mut |resolve, reject| {
            let run_js = format!(
                r#"
                {js_code}
                setupStations().then(resolve).catch(reject);
                "#
            );

            if let Err(e) = js_sys::eval(&run_js) {
                reject
                    .call1(
                        &JsValue::NULL,
                        &JsValue::from_str(&format!("Failed to setup stations: {:?}", e)),
                    )
                    .unwrap();
            }
        });

        // Function to wait for promise and add layers
        let wait_for_promise = format!(
            r#"
            async function addStationsToMap() {{
                try {{
                    const geojson = await arguments[0];

                    // Add source if it doesn't exist
                    if (!window.mapInstance.getSource('{}')) {{
                        window.mapInstance.addSource('{}', {{
                            type: 'geojson',
                            data: geojson
                        }});

                        // Add stations layer
                        window.mapInstance.addLayer({{
                            id: '{}-layer',
                            type: 'circle',
                            source: '{}',
                            layout: {{
                                'visibility': 'visible'
                            }},
                            paint: {{
                                'circle-radius': [
                                    'match',
                                    ['get', 'stationType'],
                                    'interchange', 8,
                                    'tube', 6,
                                    'bus', 4,
                                    5 // default
                                ],
                                'circle-color': '#ffffff',
                                'circle-stroke-color': '#000000',
                                'circle-stroke-width': 2
                            }}
                        }});

                        // Add station labels layer (only for interchanges)
                        window.mapInstance.addLayer({{
                            id: '{}-labels',
                            type: 'symbol',
                            source: '{}',
                            filter: ['==', ['get', 'isInterchange'], true],
                            layout: {{
                                'text-field': ['get', 'name'],
                                'text-size': 12,
                                'text-anchor': 'top',
                                'text-offset': [0, 1],
                                'visibility': 'visible'
                            }},
                            paint: {{
                                'text-color': '#000000',
                                'text-halo-color': '#ffffff',
                                'text-halo-width': 2
                            }}
                        }});

                        console.log('Added stations to map');
                    }}

                    return true;
                }} catch (err) {{
                    console.error('Error adding stations to map:', err);
                    throw err;
                }}
            }}

            addStationsToMap();
            "#,
            self.stations_source_id,
            self.stations_source_id,
            self.stations_source_id,
            self.stations_source_id,
            self.stations_source_id,
            self.stations_source_id
        );

        // Execute the wait function
        let result = Function::new_with_args("stations", &wait_for_promise)
            .call1(&JsValue::NULL, &stations_promise)?;

        // Check if we got an error
        if result.is_instance_of::<js_sys::Error>() {
            return Err(JsValue::from_str(&format!(
                "Error setting up stations: {:?}",
                result
            )));
        }

        console::log_1(&"Stations setup successfully".into());
        Ok(())
    }

    // Setup vehicle simulation
    fn setup_simulation(&mut self, map: &Map) -> Result<(), JsValue> {
        console::log_1(&"Setting up vehicle simulation...".into());

        // Add empty vehicles source and layers
        let js_code = format!(
            r#"
            function setupVehicleSimulation() {{
                try {{
                    // Add empty source for vehicles
                    if (!window.mapInstance.getSource('{}')) {{
                        window.mapInstance.addSource('{}', {{
                            type: 'geojson',
                            data: {{
                                type: 'FeatureCollection',
                                features: []
                            }}
                        }});

                        // Add buses layer
                        window.mapInstance.addLayer({{
                            id: 'buses-layer',
                            type: 'circle',
                            source: '{}',
                            filter: ['==', ['get', 'vehicleType'], 'Bus'],
                            paint: {{
                                'circle-radius': 6,
                                'circle-color': '#0000FF',
                                'circle-stroke-color': '#FFFFFF',
                                'circle-stroke-width': 2
                            }}
                        }});

                        // Add trains layer
                        window.mapInstance.addLayer({{
                            id: 'trains-layer',
                            type: 'circle',
                            source: '{}',
                            filter: ['==', ['get', 'vehicleType'], 'Train'],
                            paint: {{
                                'circle-radius': 6,
                                'circle-color': '#FF0000',
                                'circle-stroke-color': '#FFFFFF',
                                'circle-stroke-width': 2
                            }}
                        }});

                        console.log('Added vehicle simulation layers');

                        // Initialize simulation controller
                        setupSimulationController();
                    }}

                    return true;
                }} catch (err) {{
                    console.error('Error setting up vehicle simulation:', err);
                    throw err;
                }}
            }}

            // Setup the simulation controller
            function setupSimulationController() {{
                window.SimulationController = {{
                    vehicles: [],
                    isRunning: false,
                    animationFrameId: null,
                    simSpeed: 0.005, // Default speed

                    initialize: function() {{
                        console.log('Initializing simulation controller');
                        this.createVehicles();
                        this.start();
                    }},

                    createVehicles: function() {{
                        // Get all tube lines and bus routes
                        const tubeSource = window.mapInstance.getSource('{}');
                        const busSource = window.mapInstance.getSource('{}');

                        if (!tubeSource || !busSource) {{
                            console.error('Cannot create vehicles: sources not found');
                            return;
                        }}

                        // Get the data from sources
                        const tubeData = tubeSource._data;
                        const busData = busSource._data;

                        // Create vehicles
                        const vehicles = [];

                        // Create trains for tube lines
                        if (tubeData && tubeData.features) {{
                            tubeData.features.forEach((feature, index) => {{
                                // Create 3-5 trains per line
                                const trainCount = 3 + Math.floor(Math.random() * 3);

                                for (let i = 0; i < trainCount; i++) {{
                                    const coords = feature.geometry.coordinates;

                                    // Only create if we have valid coordinates
                                    if (coords && coords.length >= 2) {{
                                        // Random position along the line
                                        const progress = Math.random();
                                        const position = this.interpolatePosition(
                                            coords[0],
                                            coords[1],
                                            progress
                                        );

                                        vehicles.push({{
                                            id: vehicles.length,
                                            vehicleType: 'Train',
                                            routeName: feature.properties.name,
                                            position: position,
                                            startPosition: coords[0],
                                            endPosition: coords[1],
                                            progress: progress,
                                            speed: 0.001 + (Math.random() * 0.009),
                                            direction: Math.random() > 0.5 ? 1 : -1
                                        }});
                                    }}
                                }}
                            }});
                        }}

                        // Create buses for bus routes
                        if (busData && busData.features) {{
                            busData.features.forEach((feature, index) => {{
                                // Create 1-2 buses per route
                                const busCount = 1 + Math.floor(Math.random() * 2);

                                for (let i = 0; i < busCount; i++) {{
                                    const coords = feature.geometry.coordinates;

                                    // Only create if we have valid coordinates
                                    if (coords && coords.length >= 2) {{
                                        // Random position along the route
                                        const progress = Math.random();
                                        const position = this.interpolatePosition(
                                            coords[0],
                                            coords[1],
                                            progress
                                        );

                                        vehicles.push({{
                                            id: vehicles.length,
                                            vehicleType: 'Bus',
                                            routeNumber: feature.properties.routeNumber,
                                            position: position,
                                            startPosition: coords[0],
                                            endPosition: coords[1],
                                            progress: progress,
                                            speed: 0.0005 + (Math.random() * 0.0045), // Buses move slower than trains
                                            direction: Math.random() > 0.5 ? 1 : -1
                                        }});
                                    }}
                                }}
                            }});
                        }}

                        this.vehicles = vehicles;
                        console.log(`Created ${{vehicles.length}} vehicles for simulation`);
                    }},

                    start: function() {{
                        console.log('Starting simulation');
                        this.isRunning = true;
                        this.updateVehicles();
                    }},

                    pause: function() {{
                        console.log('Pausing simulation');
                        this.isRunning = false;

                        if (this.animationFrameId) {{
                            cancelAnimationFrame(this.animationFrameId);
                            this.animationFrameId = null;
                        }}
                    }},

                    toggle: function() {{
                        if (this.isRunning) {{
                            this.pause();
                        }} else {{
                            this.start();
                        }}
                    }},

                    reset: function() {{
                        console.log('Resetting simulation');
                        this.pause();

                        // Reset all vehicles to random starting positions
                        this.vehicles.forEach(vehicle => {{
                            vehicle.progress = Math.random();
                            vehicle.position = this.interpolatePosition(
                                vehicle.startPosition,
                                vehicle.endPosition,
                                vehicle.progress
                            );
                            vehicle.direction = Math.random() > 0.5 ? 1 : -1;
                        }});

                        // Update the map
                        this.updateVehiclePositions();

                        // Restart
                        this.start();
                    }},

                    setSpeed: function(speed) {{
                        // speed should be between 0 and 1
                        const normalizedSpeed = Math.min(Math.max(speed, 0), 1);
                        // Convert to a reasonable speed factor (0.001 to 0.01)
                        this.simSpeed = 0.001 + (normalizedSpeed * 0.009);
                        console.log(`Set simulation speed to ${{this.simSpeed}}`);
                    }},

                    updateVehicles: function() {{
                        if (!this.isRunning) return;

                        // Update positions
                        this.vehicles.forEach(vehicle => {{
                            // Update progress based on speed and direction
                            vehicle.progress += vehicle.speed * vehicle.direction * (this.simSpeed / 0.005);

                            // Check if we've reached the end
                            if (vehicle.progress >= 1.0) {{
                                vehicle.progress = 1.0;
                                vehicle.direction = -1;
                            }} else if (vehicle.progress <= 0.0) {{
                                vehicle.progress = 0.0;
                                vehicle.direction = 1;
                            }}

                            // Update position
                            vehicle.position = this.interpolatePosition(
                                vehicle.startPosition,
                                vehicle.endPosition,
                                vehicle.progress
                            );
                        }});

                        // Update the map
                        this.updateVehiclePositions();

                        // Request next frame
                        this.animationFrameId = requestAnimationFrame(() => this.updateVehicles());
                    }},

                    updateVehiclePositions: function() {{
                        // Create GeoJSON for vehicles
                        const features = this.vehicles.map(vehicle => {{
                            return {{
                                type: 'Feature',
                                properties: {{
                                    id: vehicle.id,
                                    vehicleType: vehicle.vehicleType,
                                    routeName: vehicle.routeName || `Bus ${{vehicle.routeNumber}}`
                                }},
                                geometry: {{
                                    type: 'Point',
                                    coordinates: vehicle.position
                                }}
                            }};
                        }});

                        // Update the source
                        const source = window.mapInstance.getSource('{}');
                        if (source) {{
                            source.setData({{
                                type: 'FeatureCollection',
                                features: features
                            }});
                        }}
                    }},

                    interpolatePosition: function(start, end, progress) {{
                        const lng = start[0] + (end[0] - start[0]) * progress;
                        const lat = start[1] + (end[1] - start[1]) * progress;
                        return [lng, lat];
                    }}
                }};

                console.log('Simulation controller setup complete');
            }}

            setupVehicleSimulation();
            "#,
            self.vehicles_source_id,
            self.vehicles_source_id,
            self.vehicles_source_id,
            self.vehicles_source_id,
            self.tube_lines_source_id,
            self.bus_routes_source_id,
            self.vehicles_source_id
        );

        // Execute the JS code to set up simulation
        let result = js_sys::eval(&js_code)?;

        // Add global functions to control the simulation
        let control_js = r#"
            // Global functions to control the simulation
            window.rust_toggle_simulation = function() {
                console.log('Toggle simulation called from Rust');
                if (window.SimulationController) {
                    window.SimulationController.toggle();
                }}
            }};

            window.rust_reset_simulation = function() {
                console.log('Reset simulation called from Rust');
                if (window.SimulationController) {
                    window.SimulationController.reset();
                }}
            }};

            window.rust_set_simulation_speed = function(speed) {
                console.log('Set simulation speed called from Rust:', speed);
                if (window.SimulationController) {
                    window.SimulationController.setSpeed(speed);
                }}
            }};

            // Initialize the simulation if automatic start is enabled
            if (window.SimulationController && !window.SimulationController.vehicles.length) {
                window.SimulationController.initialize();
            }}
        "#;

        js_sys::eval(control_js)?;

        console::log_1(&"Vehicle simulation setup successfully".into());
        Ok(())
    }

    // Toggle simulation on/off
    pub fn toggle_simulation(&mut self) -> Result<(), JsValue> {
        console::log_1(&"Toggling TfL simulation...".into());

        let js_code = r#"
            if (window.SimulationController) {
                window.SimulationController.toggle();
                return window.SimulationController.isRunning;
            }}
            return false;
        "#;

        let result = js_sys::eval(js_code)?;

        if let Some(is_running) = result.as_bool() {
            self.simulation_active = is_running;
            console::log_1(
                &format!(
                    "Simulation is now {}",
                    if is_running { "running" } else { "paused" }
                )
                .into(),
            );
        }

        Ok(())
    }

    // Reset simulation
    pub fn reset_simulation(&mut self) -> Result<(), JsValue> {
        console::log_1(&"Resetting TfL simulation...".into());

        let js_code = r#"
            if (window.SimulationController) {
                window.SimulationController.reset();
                return true;
            }}
            return false;
        "#;

        js_sys::eval(js_code)?;

        Ok(())
    }

    // Set simulation speed
    pub fn set_simulation_speed(&mut self, speed: f64) -> Result<(), JsValue> {
        console::log_1(&format!("Setting TfL simulation speed to {}", speed).into());

        let js_code = format!(
            r#"
            if (window.SimulationController) {{
                window.SimulationController.setSpeed({{{}}});
                return true;
            }}
            return false;
            "#,
            speed
        );

        js_sys::eval(&js_code)?;

        Ok(())
    }

    // Update layer visibility
    pub fn update_layer_visibility(&self, layers: &crate::app::TflLayers) -> Result<(), JsValue> {
        console::log_1(&"Updating TfL layer visibility...".into());

        let js_code = format!(
            r#"
            function updateTflLayers() {{
                const map = window.mapInstance;
                if (!map) return false;

                // Tube lines visibility
                if (map.getLayer('{}-layer')) {{
                    map.setLayoutProperty(
                        '{}-layer',
                        'visibility',
                        {} ? 'visible' : 'none'
                    );
                }}

                // Bus routes visibility
                if (map.getLayer('{}-layer')) {{
                    map.setLayoutProperty(
                        '{}-layer',
                        'visibility',
                        {} ? 'visible' : 'none'
                    );
                }}

                // Stations visibility
                if (map.getLayer('{}-layer')) {{
                    map.setLayoutProperty(
                        '{}-layer',
                        'visibility',
                        {} ? 'visible' : 'none'
                    );
                }}

                // Station labels visibility (interchanges only or all stations)
                if (map.getLayer('{}-labels')) {{
                    // First set visibility based on stations toggle
                    map.setLayoutProperty(
                        '{}-labels',
                        'visibility',
                        {} ? 'visible' : 'none'
                    );

                    // Then update the filter based on show_all_stations toggle
                    if ({}) {{
                        // Show all station labels
                        map.setFilter('{}-labels', null);
                    }} else {{
                        // Only show interchange station labels
                        map.setFilter('{}-labels', ['==', ['get', 'isInterchange'], true]);
                    }}
                }}

                // Vehicle simulation layers
                if (map.getLayer('buses-layer')) {{
                    map.setLayoutProperty(
                        'buses-layer',
                        'visibility',
                        ({} && {}) ? 'visible' : 'none'
                    );
                }}

                if (map.getLayer('trains-layer')) {{
                    map.setLayoutProperty(
                        'trains-layer',
                        'visibility',
                        ({} && {}) ? 'visible' : 'none'
                    );
                }}

                return true;
            }}

            updateTflLayers();
            "#,
            self.tube_lines_source_id,
            self.tube_lines_source_id,
            layers.tube,
            self.bus_routes_source_id,
            self.bus_routes_source_id,
            layers.buses,
            self.stations_source_id,
            self.stations_source_id,
            layers.stations,
            self.stations_source_id,
            self.stations_source_id,
            layers.stations,
            layers.show_all_stations,
            self.stations_source_id,
            self.stations_source_id,
            layers.buses,
            layers.simulation,
            layers.tube,
            layers.simulation
        );

        js_sys::eval(&js_code)?;

        console::log_1(&"TfL layer visibility updated".into());
        Ok(())
    }
}
