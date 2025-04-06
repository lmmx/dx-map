use super::model::{Platform, Station};
use crate::data::TflDataRepository;
use crate::utils::geojson::{Feature, FeatureCollection, GeoJsonSource, Geometry, to_js_value};
use crate::utils::log::{self, LogCategory};
use js_sys::{Array, Object, Reflect};
use serde::Serialize;
use std::collections::HashMap;
use wasm_bindgen::{JsError, JsValue};

/// Convert a list of stations into a format suitable for MapLibre GeoJSON
pub fn stations_to_geojson(stations: &[Station]) -> Result<JsValue, JsError> {
    log::info_with_category(
        LogCategory::Map,
        &format!("Converting {} stations to GeoJSON", stations.len()),
    );

    let mut features = Vec::new();

    for station in stations {
        // Skip stations with invalid coordinates
        if station.lat.is_nan() || station.lon.is_nan() {
            continue;
        }

        // Create properties as a JSON object
        let properties = serde_json::json!({
            "id": station.station_unique_id,
            "name": station.station_name,
            "fareZones": station.fare_zones,
            "wifi": station.wifi,
        });

        // Create a feature for this station
        let feature = Feature {
            feature_type: "Feature".to_string(),
            geometry: Geometry::Point {
                coordinates: [station.lon, station.lat], // Note: GeoJSON is [lng, lat]
            },
            properties,
        };

        features.push(feature);
    }

    // Store the length before moving the vector
    let feature_count = features.len();

    // Create the GeoJSON source
    let geojson_source = GeoJsonSource {
        source_type: "geojson".to_string(),
        data: FeatureCollection {
            collection_type: "FeatureCollection".to_string(),
            features,
        },
    };

    log::debug_with_category(
        LogCategory::Map,
        &format!("Created GeoJSON with {} features", feature_count),
    );

    // Convert to JsValue
    to_js_value(&geojson_source)
}

/// Create a mapping of line names to their corresponding stations
pub fn create_line_stations_map(platforms: &[Platform]) -> HashMap<String, Vec<String>> {
    let mut map = HashMap::new();

    for platform in platforms {
        // Skip non-TfL lines or special cases
        if platform.line == "national-rail" || platform.line.is_empty() {
            continue;
        }

        map.entry(platform.line.clone())
            .or_insert_with(Vec::new)
            .push(platform.station_unique_id.clone());
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
        &format!(
            "Creating GeoJSON for {} line with {} stations",
            line_name,
            station_ids.len()
        ),
    );

    // Get coordinates for all stations on this line
    let mut coordinates = Vec::new();
    for station_id in station_ids {
        if let Some(station) = stations_by_id.get(station_id) {
            coordinates.push((station.lon, station.lat));
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
        &format!(
            "Created GeoJSON LineString for {} line with {} points",
            line_name,
            coords_array.length()
        ),
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
        "london-cable-car" => "#AF174C",
        "london-overground" => "#EE7C0E",
        "metropolitan" => "#9B0056",
        "northern" => "#000000",
        "piccadilly" => "#003688",
        "thameslink" => "#C1007C",
        "tram" => "#84B817",
        "victoria" => "#0098D4",
        "waterloo-city" => "#95CDBA",
        "liberty" => "#4C6366",
        "lioness" => "#FFA32B",
        "mildmay" => "#088ECC",
        "suffragette" => "#59C274",
        "weaver" => "#B43983",
        "windrush" => "#FF2E24",
        _ => "#FFFFFF", // Default white for unknown lines
    }
}

// Not used: left in for debugging (if there's a new line without routes, uncomment use in app/mod.rs)
/// Generate all line data for MapLibre
#[allow(dead_code)]
pub fn generate_all_line_data(
    repository: &super::TflDataRepository,
) -> Result<Vec<(String, JsValue, String)>, JsValue> {
    log::info_with_category(LogCategory::Map, "Generating data for all TfL lines");

    // Collect all platforms from repository into a single Vec<Platform>
    let platforms: Vec<Platform> = repository
        .platforms_by_station
        .values()
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
                &format!(
                    "Skipping {} line with only {} stations",
                    line_name,
                    station_ids.len()
                ),
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

/// Convert route geometries for a specific line to GeoJSON
pub fn route_geometries_to_geojson(
    line_id: &str,
    geometries: &Vec<Vec<[f64; 2]>>,
) -> Result<JsValue, JsError> {
    let mut features = Vec::new();

    // Process each route segment
    for (i, coordinates) in geometries.iter().enumerate() {
        // Skip empty geometries
        if coordinates.is_empty() {
            continue;
        }

        let properties = serde_json::json!({
            "line_id": line_id,
            "segment_id": i,
        });

        // Create a GeoJSON LineString feature
        let feature = Feature {
            feature_type: "Feature".to_string(),
            geometry: Geometry::LineString {
                coordinates: coordinates.clone(),
            },
            properties,
        };

        features.push(feature);
    }

    // Create the GeoJSON source using our structs
    let geo_json_source = GeoJsonSource {
        source_type: "geojson".to_string(),
        data: FeatureCollection {
            collection_type: "FeatureCollection".to_string(),
            features,
        },
    };

    to_js_value(&geo_json_source)
}

/// Generate all route geometries as GeoJSON for multiple lines
pub fn generate_all_route_geometries(
    tfl_data: &TflDataRepository,
) -> Result<Vec<(String, JsValue)>, JsError> {
    let mut result = Vec::new();

    // Process each line
    for (line_id, geometries) in &tfl_data.route_geometries {
        // Skip lines with no geometries
        if geometries.is_empty() {
            continue;
        }

        match route_geometries_to_geojson(line_id, geometries) {
            Ok(geojson) => {
                result.push((line_id.clone(), geojson));
            }
            Err(err) => {
                log::error_with_category(
                    LogCategory::Map,
                    &format!("Failed to convert route to GeoJSON: {:?}", err),
                );
            }
        }
    }

    Ok(result)
}
