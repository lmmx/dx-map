use super::model::{Platform, PlatformsResponse, Station, StationsResponse};
use crate::data::model::{RouteSequence, RoutesFile};
use crate::utils::log::{self, LogCategory};
use dioxus::prelude::*;
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::Response;

// Define asset paths for our data files
const STATIONS_JSON_PATH: Asset = asset!("/assets/data/stations.json");
const PLATFORMS_JSON_PATH: Asset = asset!("/assets/data/platforms.json");
const ROUTES_JSON_PATH: Asset = asset!("/assets/data/routes.json");

/// Load stations from the JSON data file using fetch
pub async fn load_stations() -> Result<Vec<Station>, String> {
    log::info_with_category(LogCategory::App, "Loading stations from JSON data file");

    // Create a future to fetch the stations data
    let window = web_sys::window().ok_or("No window object available")?;
    let promise = window.fetch_with_str(
        STATIONS_JSON_PATH
            .resolve()
            .to_str()
            .expect("Failed to load stations JSON"),
    );

    // Convert the Promise<Response> to a Future<Result<Response, JsValue>>
    let response_future = wasm_bindgen_futures::JsFuture::from(promise);

    // Await the response
    let response_value = match response_future.await {
        Ok(val) => val,
        Err(e) => return Err(format!("Failed to fetch stations: {:?}", e)),
    };

    let response: Response = response_value
        .dyn_into()
        .map_err(|_| "Failed to convert response")?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    // Get the response text
    let text_promise = response
        .text()
        .map_err(|e| format!("Failed to get response text: {:?}", e))?;
    let text_future = wasm_bindgen_futures::JsFuture::from(text_promise);

    let text = match text_future.await {
        Ok(val) => val.as_string().ok_or("Response is not a string")?,
        Err(e) => return Err(format!("Failed to get response text: {:?}", e)),
    };

    // Parse the JSON
    match serde_json::from_str::<StationsResponse>(&text) {
        Ok(response) => {
            if !response.success {
                return Err("Stations response was unsuccessful".to_string());
            }
            log::info_with_category(
                LogCategory::App,
                &format!("Successfully loaded {} stations", response.results.len()),
            );
            Ok(response.results)
        }
        Err(e) => {
            let error_msg = format!("Failed to parse stations JSON: {}", e);
            log::error_with_category(LogCategory::App, &error_msg);
            Err(error_msg)
        }
    }
}

/// Load platforms from the JSON data file using fetch
pub async fn load_platforms() -> Result<Vec<Platform>, String> {
    log::info_with_category(LogCategory::App, "Loading platforms from JSON data file");

    // Create a future to fetch the platforms data
    let window = web_sys::window().ok_or("No window object available")?;
    let promise = window.fetch_with_str(
        PLATFORMS_JSON_PATH
            .resolve()
            .to_str()
            .expect("Failed to load stations JSON"),
    );

    // Convert the Promise<Response> to a Future<Result<Response, JsValue>>
    let response_future = wasm_bindgen_futures::JsFuture::from(promise);

    // Await the response
    let response_value = match response_future.await {
        Ok(val) => val,
        Err(e) => return Err(format!("Failed to fetch platforms: {:?}", e)),
    };

    let response: Response = response_value
        .dyn_into()
        .map_err(|_| "Failed to convert response")?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    // Get the response text
    let text_promise = response
        .text()
        .map_err(|e| format!("Failed to get response text: {:?}", e))?;
    let text_future = wasm_bindgen_futures::JsFuture::from(text_promise);

    let text = match text_future.await {
        Ok(val) => val.as_string().ok_or("Response is not a string")?,
        Err(e) => return Err(format!("Failed to get response text: {:?}", e)),
    };

    // Parse the JSON
    match serde_json::from_str::<PlatformsResponse>(&text) {
        Ok(response) => {
            if !response.success {
                return Err("Platforms response was unsuccessful".to_string());
            }
            log::info_with_category(
                LogCategory::App,
                &format!("Successfully loaded {} platforms", response.results.len()),
            );
            Ok(response.results)
        }
        Err(e) => {
            let error_msg = format!("Failed to parse platforms JSON: {}", e);
            log::error_with_category(LogCategory::App, &error_msg);
            Err(error_msg)
        }
    }
}

/// Filter stations to only include those with valid coordinates
pub fn filter_valid_stations(stations: Vec<Station>) -> Vec<Station> {
    stations
        .into_iter()
        .filter(|station| {
            !station.lat.is_nan()
                && !station.lon.is_nan()
                && station.lat != 0.0
                && station.lon != 0.0
        })
        .collect()
}

/// Group platforms by station
pub fn group_platforms_by_station(
    platforms: Vec<Platform>,
) -> std::collections::HashMap<String, Vec<Platform>> {
    let mut map = std::collections::HashMap::new();

    for platform in platforms {
        map.entry(platform.station_unique_id.clone())
            .or_insert_with(Vec::new)
            .push(platform);
    }

    map
}

/// Load routes from the JSON data file using fetch
pub async fn load_routes() -> Result<HashMap<String, HashMap<String, Vec<RouteSequence>>>, String> {
    log::info_with_category(LogCategory::App, "Loading routes from JSON data file");

    // Create a future to fetch the routes data
    let window = web_sys::window().ok_or("No window object available")?;
    let promise = window.fetch_with_str(
        ROUTES_JSON_PATH
            .resolve()
            .to_str()
            .expect("Failed to load routes JSON"),
    );

    // Convert the Promise<Response> to a Future<Result<Response, JsValue>>
    let response_future = wasm_bindgen_futures::JsFuture::from(promise);

    // Await the response
    let response_value = match response_future.await {
        Ok(val) => val,
        Err(e) => return Err(format!("Failed to fetch routes: {:?}", e)),
    };

    let response: Response = response_value
        .dyn_into()
        .map_err(|_| "Failed to convert response")?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    // Get the response text
    let text_promise = response
        .text()
        .map_err(|e| format!("Failed to get response text: {:?}", e))?;
    let text_future = wasm_bindgen_futures::JsFuture::from(text_promise);

    let text = match text_future.await {
        Ok(val) => val.as_string().ok_or("Response is not a string")?,
        Err(e) => return Err(format!("Failed to get response text: {:?}", e)),
    };

    // Parse the JSON
    match serde_json::from_str::<RoutesFile>(&text) {
        Ok(routes_file) => {
            // Convert the nested HashMap to our desired format
            let mut routes_map: HashMap<String, HashMap<String, Vec<RouteSequence>>> =
                HashMap::new();

            // Process each line and its directions
            for (line_id, directions) in routes_file.routes {
                let mut direction_map: HashMap<String, Vec<RouteSequence>> = HashMap::new();

                // Process each direction
                for (direction, response) in directions {
                    if response.success {
                        direction_map.insert(direction, response.results);
                    } else {
                        // Error out immediately (a failed response shouldn't have been stored)
                        return Err(format!(
                            "Unsuccessful response record for line {}, direction {}",
                            line_id, direction
                        ));
                    }
                }

                routes_map.insert(line_id, direction_map);
            }

            log::info_with_category(
                LogCategory::App,
                &format!("Successfully loaded routes for {} lines", routes_map.len()),
            );

            Ok(routes_map)
        }
        Err(e) => {
            let error_msg = format!("Failed to parse routes JSON: {}", e);
            log::error_with_category(LogCategory::App, &error_msg);
            Err(error_msg)
        }
    }
}

/// Process route data to create a mapping of line ID to route geometry
pub fn process_route_geometries(
    routes: &HashMap<String, HashMap<String, Vec<RouteSequence>>>,
) -> HashMap<String, Vec<Vec<[f64; 2]>>> {
    let mut line_geometries: HashMap<String, Vec<Vec<[f64; 2]>>> = HashMap::new();

    for (line_id, directions) in routes {
        let mut geometries = Vec::new();

        // Process both inbound and outbound directions
        for route_sequences in directions.values() {
            for sequence in route_sequences {
                for line_string in &sequence.line_strings {
                    // Parse the LineString from GeoJSON format
                    if let Ok(coordinates) = parse_line_string(line_string) {
                        geometries.push(coordinates);
                    }
                }
            }
        }

        // Only add if we have valid geometries
        if !geometries.is_empty() {
            line_geometries.insert(line_id.clone(), geometries);
        }
    }

    line_geometries
}

/// Parse a LineString from a GeoJSON-like format
fn parse_line_string(line_string: &str) -> Result<Vec<[f64; 2]>, String> {
    // The LineString format is like: "[[[-0.335217,51.592268],[-0.31691,51.581756],[-0.308433,51.570232]]]"
    // We need to parse this and extract the coordinates

    // First, remove outer brackets and any whitespace
    let trimmed = line_string
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']');

    // Now parse the inner arrays
    let mut coordinates = Vec::new();

    // Simple parser for this specific format
    // Using regex or a proper JSON parser would be more robust
    let parts: Vec<&str> = trimmed.split("],[").collect();

    for part in parts {
        let clean_part = part.trim_start_matches('[').trim_end_matches(']');
        let coords: Vec<&str> = clean_part.split(',').collect();

        if coords.len() == 2 {
            if let (Ok(lon), Ok(lat)) = (coords[0].parse::<f64>(), coords[1].parse::<f64>()) {
                coordinates.push([lon, lat]);
            }
        }
    }

    if coordinates.is_empty() {
        Err("Failed to parse any coordinates from LineString".to_string())
    } else {
        Ok(coordinates)
    }
}
