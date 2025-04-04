use super::model::{Station, Platform};
use crate::utils::log::{self, LogCategory};
use js_sys::{Array, Object, Reflect};
use std::collections::HashMap;
use wasm_bindgen::JsValue;

/// Convert a list of stations into a format suitable for MapLibre GeoJSON
pub fn stations_to_geojson(stations: &[Station]) -> Result<JsValue, JsValue> {
    log::info_with_category(
        LogCategory::Map,
        &format!("Converting {} stations to GeoJSON", stations.len()),
    );
    
    // Create the GeoJSON structure
    let geojson = Object::new();
    Reflect::set(
        &geojson,
        &JsValue::from_str("type"),
        &JsValue::from_str("FeatureCollection"),
    )?;
    
    // Create the features array
    let features = Array::new();
    
    for station in stations {
        // Skip stations with invalid coordinates
        if station.Lat.is_nan() || station.Lon.is_nan() {
            continue;
        }
        
        // Create a feature for this station
        let feature = Object::new();
        Reflect::set(
            &feature,
            &JsValue::from_str("type"),
            &JsValue::from_str("Feature"),
        )?;
        
        // Set the geometry
        let geometry = Object::new();
        Reflect::set(
            &geometry,
            &JsValue::from_str("type"),
            &JsValue::from_str("Point"),
        )?;
        
        let coordinates = Array::new();
        coordinates.push(&JsValue::from_f64(station.Lon)); // Note: GeoJSON is [lng, lat]
        coordinates.push(&JsValue::from_f64(station.Lat));
        
        Reflect::set(
            &geometry,
            &JsValue::from_str("coordinates"),
            &coordinates,
        )?;
        
        Reflect::set(
            &feature,
            &JsValue::from_str("geometry"),
            &geometry,
        )?;
        
        // Set the properties
        let properties = Object::new();
        Reflect::set(
            &properties,
            &JsValue::from_str("id"),
            &JsValue::from_str(&station.StationUniqueId),
        )?;
        Reflect::set(
            &properties,
            &JsValue::from_str("name"),
            &JsValue::from_str(&station.StationName),
        )?;
        Reflect::set(
            &properties,
            &JsValue::from_str("fareZones"),
            &JsValue::from_str(&station.FareZones),
        )?;
        Reflect::set(
            &properties,
            &JsValue::from_str("wifi"),
            &JsValue::from_bool(station.Wifi),
        )?;
        
        Reflect::set(
            &feature,
            &JsValue::from_str("properties"),
            &properties,
        )?;
        
        // Add this feature to the features array
        features.push(&feature);
    }
    
    // Set the features array on the GeoJSON object
    Reflect::set(&geojson, &JsValue::from_str("features"), &features)?;
    
    log::debug_with_category(
        LogCategory::Map,
        &format!("Created GeoJSON with {} features", features.length()),
    );
    
    Ok(geojson.into())
}

/// Create a mapping of line names to their corresponding stations
pub fn create_line_stations_map(platforms: &[Platform]) -> HashMap<String, Vec<String>> {
    let mut map = HashMap::new();
    
    for platform in platforms {
        // Skip non-TfL lines or special cases
        if platform.Line == "national-rail" || platform.Line.is_empty() {
            continue;
        }
        
        map.entry(platform.Line.clone())
            .or_insert_with(Vec::new)
            .push(platform.StationUniqueId.clone());
    }
    
    // Deduplicate station IDs for each line
    for stations in map.values_mut() {
        stations.sort();
        stations.dedup();
    }
    
    map
}

/// Convert line stations to GeoJSON LineString format
pub fn line_to_geojson(
    line_name: &str,
    station_ids: &[String],
    stations_by_id: &HashMap<String, Station>,
) -> Result<JsValue, JsValue> {
    log::info_with_category(
        LogCategory::Map,
        &format!("Creating GeoJSON for {} line with {} stations", line_name, station_ids.len()),
    );
    
    // Get coordinates for all stations on this line
    let mut coordinates = Vec::new();
    for station_id in station_ids {
        if let Some(station) = stations_by_id.get(station_id) {
            coordinates.push((station.Lon, station.Lat));
        }
    }
    
    // We need at least 2 points to form a line
    if coordinates.len() < 2 {
        return Err(JsValue::from_str(&format!(
            "Not enough stations with valid coordinates for {} line",
            line_name
        )));
    }
    
    // Create the GeoJSON
    let source = Object::new();
    Reflect::set(
        &source,
        &JsValue::from_str("type"),
        &JsValue::from_str("geojson"),
    )?;
    
    let data = Object::new();
    Reflect::set(
        &data,
        &JsValue::from_str("type"),
        &JsValue::from_str("Feature"),
    )?;
    
    // Set properties
    let properties = Object::new();
    Reflect::set(
        &properties,
        &JsValue::from_str("name"),
        &JsValue::from_str(line_name),
    )?;
    Reflect::set(&data, &JsValue::from_str("properties"), &properties)?;
    
    // Set geometry
    let geometry = Object::new();
    Reflect::set(
        &geometry,
        &JsValue::from_str("type"),
        &JsValue::from_str("LineString"),
    )?;
    
    let coords_array = Array::new();
    for &(lng, lat) in &coordinates {
        let point = Array::new();
        point.push(&JsValue::from_f64(lng));
        point.push(&JsValue::from_f64(lat));
        coords_array.push(&point);
    }
    
    Reflect::set(&geometry, &JsValue::from_str("coordinates"), &coords_array)?;
    Reflect::set(&data, &JsValue::from_str("geometry"), &geometry)?;
    Reflect::set(&source, &JsValue::from_str("data"), &data)?;
    
    log::debug_with_category(
        LogCategory::Map,
        &format!("Created GeoJSON LineString for {} line with {} points", line_name, coords_array.length()),
    );
    
    Ok(source.into())
}

/// Get the color for a specific TfL line
pub fn get_line_color(line_name: &str) -> &'static str {
    match line_name {
        "bakerloo" => "#B36305",
        "central" => "#E32017",
        "circle" => "#FFD300",
        "district" => "#00782A",
        "dlr" => "#00A4A7",
        "elizabeth" => "#6950A1",
        "hammersmith-city" => "#F3A9BB",
        "jubilee" => "#A0A5A9",
        "metropolitan" => "#9B0056",
        "northern" => "#000000",
        "overground" => "#EE7C0E",
        "piccadilly" => "#003688",
        "tram" => "#84B817",
        "victoria" => "#0098D4",
        "waterloo-city" => "#95CDBA",
        _ => "#999999", // Default gray for unknown lines
    }
}

/// Generate all line data for MapLibre
pub fn generate_all_line_data(
    repository: &super::TflDataRepository,
) -> Result<Vec<(String, JsValue, String)>, JsValue> {
    log::info_with_category(LogCategory::Map, "Generating data for all TfL lines");
    
    // Collect all platforms from repository into a single Vec<Platform>
    let platforms: Vec<Platform> = repository.platforms_by_station.values()
        .flat_map(|v| v.clone())
        .collect();
    
    // Create map of line names to station IDs
    let line_stations = create_line_stations_map(&platforms);
    
    // Generate GeoJSON for each line
    let mut result = Vec::new();
    
    for (line_name, station_ids) in line_stations {
        // Skip lines with too few stations
        if station_ids.len() < 2 {
            log::debug_with_category(
                LogCategory::Map,
                &format!("Skipping {} line with only {} stations", line_name, station_ids.len()),
            );
            continue;
        }
        
        match line_to_geojson(&line_name, &station_ids, &repository.station_by_id) {
            Ok(geojson) => {
                let color = get_line_color(&line_name);
                result.push((line_name, geojson, color.to_string()));
            }
            Err(e) => {
                log::error_with_category(
                    LogCategory::Map,
                    &format!("Failed to generate GeoJSON for {} line: {:?}", line_name, e),
                );
            }
        }
    }
    
    log::info_with_category(
        LogCategory::Map, 
        &format!("Generated data for {} TfL lines", result.len()),
    );
    
    Ok(result)
}