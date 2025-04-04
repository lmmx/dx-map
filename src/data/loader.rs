use super::model::{Platform, PlatformsResponse, Station, StationsResponse};
use crate::utils::log::{self, LogCategory};
use dioxus::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Response;

// Define asset paths for our data files
const STATIONS_JSON_PATH: Asset = asset!("/assets/data/stations.json");
const PLATFORMS_JSON_PATH: Asset = asset!("/assets/data/platforms.json");

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
            !station.Lat.is_nan()
                && !station.Lon.is_nan()
                && station.Lat != 0.0
                && station.Lon != 0.0
        })
        .collect()
}

/// Group platforms by station
pub fn group_platforms_by_station(
    platforms: Vec<Platform>,
) -> std::collections::HashMap<String, Vec<Platform>> {
    let mut map = std::collections::HashMap::new();

    for platform in platforms {
        map.entry(platform.StationUniqueId.clone())
            .or_insert_with(Vec::new)
            .push(platform);
    }

    map
}
