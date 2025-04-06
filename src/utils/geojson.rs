// src/utils/geojson.rs
use serde::Serialize;
use wasm_bindgen::{JsError, JsValue};

/// GeoJSON source specification
#[derive(Debug, Serialize)]
pub struct GeoJsonSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub data: FeatureCollection,
}

/// GeoJSON FeatureCollection
#[derive(Debug, Serialize)]
pub struct FeatureCollection {
    #[serde(rename = "type")]
    pub collection_type: String,
    pub features: Vec<Feature>,
}

/// GeoJSON Feature
#[derive(Debug, Serialize)]
pub struct Feature {
    #[serde(rename = "type")]
    pub feature_type: String,
    pub geometry: Geometry,
    pub properties: serde_json::Value,
}

/// GeoJSON Geometry (union type)
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum Geometry {
    #[serde(rename = "Point")]
    Point { coordinates: [f64; 2] },
    
    #[serde(rename = "LineString")]
    LineString { coordinates: Vec<[f64; 2]> },
    
    // Add other geometry types as needed
}

/// Serialize GeoJSON to JsValue
pub fn to_js_value<T: Serialize>(value: &T) -> Result<JsValue, JsError> {
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    
    match value.serialize(&serializer) {
        Ok(js_value) => Ok(js_value),
        Err(err) => Err(JsError::new(&format!(
            "Failed to serialize GeoJSON: {:?}",
            err
        ))),
    }
}