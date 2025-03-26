use crate::maplibre::bindings::*;
use wasm_bindgen::prelude::*;
use js_sys::{Array, Object, Reflect};
use web_sys::window;
use std::collections::HashMap;

// Helper to create a MapLibre map configuration
pub fn create_map_options(container_id: &str) -> Result<JsValue, JsValue> {
    let options = Object::new();
    
    // Set required properties
    Reflect::set(&options, &JsValue::from_str("container"), &JsValue::from_str(container_id))?;
    Reflect::set(&options, &JsValue::from_str("style"), &JsValue::from_str("https://tiles.openfreemap.org/styles/bright"))?;
    
    // Set London center
    let center = Array::new();
    center.push(&JsValue::from_f64(-0.1276));
    center.push(&JsValue::from_f64(51.5072));
    Reflect::set(&options, &JsValue::from_str("center"), &center)?;
    
    // Set zoom level
    Reflect::set(&options, &JsValue::from_str("zoom"), &JsValue::from_f64(12.0))?;
    
    // Set max bounds
    let bounds = Array::new();
    let sw = Array::new();
    sw.push(&JsValue::from_f64(-0.489));
    sw.push(&JsValue::from_f64(51.28));
    bounds.push(&sw);
    
    let ne = Array::new();
    ne.push(&JsValue::from_f64(0.236));
    ne.push(&JsValue::from_f64(51.686));
    bounds.push(&ne);
    
    Reflect::set(&options, &JsValue::from_str("maxBounds"), &bounds)?;
    
    Ok(options.into())
}

// Create layer configuration
pub fn create_layer_groups() -> Result<JsValue, JsValue> {
    let layer_groups = Array::new();
    
    // Background group
    {
        let labels_layer = Layer::new("labels", "Map Labels", "place-", true);
        let background_layers = Array::new();
        background_layers.push(&labels_layer);
        
        let background_group = LayerGroup::new("Background", &background_layers);
        layer_groups.push(&background_group);
    }
    
    // Transport group
    {
        let transport_layers = Array::new();
        transport_layers.push(&Layer::new("tube-central", "Central Line", "central-line-layer", true));
        transport_layers.push(&Layer::new("tube-northern", "Northern Line", "northern-line-layer", true));
        transport_layers.push(&Layer::new("overground", "Overground", "overground-line-layer", true));
        
        let transport_group = LayerGroup::new("Transport", &transport_layers);
        layer_groups.push(&transport_group);
    }
    
    // Infrastructure group
    {
        let infrastructure_layers = Array::new();
        infrastructure_layers.push(&Layer::new("stations", "Stations", "stations-layer", true));
        infrastructure_layers.push(&Layer::new("station-labels", "Station Labels", "station-labels", true));
        
        let infrastructure_group = LayerGroup::new("Infrastructure", &infrastructure_layers);
        layer_groups.push(&infrastructure_group);
    }
    
    Ok(layer_groups.into())
}

// Helper to create a GeoJSON source
pub fn create_geojson_line_source(coordinates: &[(f64, f64)]) -> Result<JsValue, JsValue> {
    let source = Object::new();
    Reflect::set(&source, &JsValue::from_str("type"), &JsValue::from_str("geojson"))?;
    
    let data = Object::new();
    Reflect::set(&data, &JsValue::from_str("type"), &JsValue::from_str("Feature"))?;
    Reflect::set(&data, &JsValue::from_str("properties"), &Object::new())?;
    
    let geometry = Object::new();
    Reflect::set(&geometry, &JsValue::from_str("type"), &JsValue::from_str("LineString"))?;
    
    let coords = Array::new();
    for &(lng, lat) in coordinates {
        let point = Array::new();
        point.push(&JsValue::from_f64(lng));
        point.push(&JsValue::from_f64(lat));
        coords.push(&point);
    }
    
    Reflect::set(&geometry, &JsValue::from_str("coordinates"), &coords)?;
    Reflect::set(&data, &JsValue::from_str("geometry"), &geometry)?;
    Reflect::set(&source, &JsValue::from_str("data"), &data)?;
    
    Ok(source.into())
}

// Helper to create a layer definition
pub fn create_line_layer(id: &str, source: &str, color: &str, width: f64) -> Result<JsValue, JsValue> {
    let layer = Object::new();
    
    Reflect::set(&layer, &JsValue::from_str("id"), &JsValue::from_str(id))?;
    Reflect::set(&layer, &JsValue::from_str("type"), &JsValue::from_str("line"))?;
    Reflect::set(&layer, &JsValue::from_str("source"), &JsValue::from_str(source))?;
    
    let layout = Object::new();
    Reflect::set(&layout, &JsValue::from_str("line-join"), &JsValue::from_str("round"))?;
    Reflect::set(&layout, &JsValue::from_str("line-cap"), &JsValue::from_str("round"))?;
    Reflect::set(&layer, &JsValue::from_str("layout"), &layout)?;
    
    let paint = Object::new();
    Reflect::set(&paint, &JsValue::from_str("line-color"), &JsValue::from_str(color))?;
    Reflect::set(&paint, &JsValue::from_str("line-width"), &JsValue::from_f64(width))?;
    Reflect::set(&layer, &JsValue::from_str("paint"), &paint)?;
    
    Ok(layer.into())
}

// Helper to create a GeoJSON point collection source
pub fn create_geojson_points_source(points: &[HashMap<String, JsValue>]) -> Result<JsValue, JsValue> {
    let source = Object::new();
    Reflect::set(&source, &JsValue::from_str("type"), &JsValue::from_str("geojson"))?;
    
    let data = Object::new();
    Reflect::set(&data, &JsValue::from_str("type"), &JsValue::from_str("FeatureCollection"))?;
    
    let features = Array::new();
    for point in points {
        let feature = Object::new();
        Reflect::set(&feature, &JsValue::from_str("type"), &JsValue::from_str("Feature"))?;
        
        let properties = Object::new();
        if let Some(name) = point.get("name") {
            Reflect::set(&properties, &JsValue::from_str("name"), name)?;
        }
        Reflect::set(&feature, &JsValue::from_str("properties"), &properties)?;
        
        let geometry = Object::new();
        Reflect::set(&geometry, &JsValue::from_str("type"), &JsValue::from_str("Point"))?;
        
        if let (Some(lng), Some(lat)) = (point.get("lng"), point.get("lat")) {
            let coords = Array::new();
            coords.push(lng);
            coords.push(lat);
            Reflect::set(&geometry, &JsValue::from_str("coordinates"), &coords)?;
        }
        
        Reflect::set(&feature, &JsValue::from_str("geometry"), &geometry)?;
        features.push(&feature);
    }
    
    Reflect::set(&data, &JsValue::from_str("features"), &features)?;
    Reflect::set(&source, &JsValue::from_str("data"), &data)?;
    
    Ok(source.into())
}

// Helper to create circle layer for stations
pub fn create_circle_layer(id: &str, source: &str) -> Result<JsValue, JsValue> {
    let layer = Object::new();
    
    Reflect::set(&layer, &JsValue::from_str("id"), &JsValue::from_str(id))?;
    Reflect::set(&layer, &JsValue::from_str("type"), &JsValue::from_str("circle"))?;
    Reflect::set(&layer, &JsValue::from_str("source"), &JsValue::from_str(source))?;
    
    let paint = Object::new();
    Reflect::set(&paint, &JsValue::from_str("circle-radius"), &JsValue::from_f64(6.0))?;
    Reflect::set(&paint, &JsValue::from_str("circle-color"), &JsValue::from_str("#ffffff"))?;
    Reflect::set(&paint, &JsValue::from_str("circle-stroke-color"), &JsValue::from_str("#000000"))?;
    Reflect::set(&paint, &JsValue::from_str("circle-stroke-width"), &JsValue::from_f64(2.0))?;
    Reflect::set(&layer, &JsValue::from_str("paint"), &paint)?;
    
    Ok(layer.into())
}

// Helper to create a text label layer
pub fn create_label_layer(id: &str, source: &str) -> Result<JsValue, JsValue> {
    let layer = Object::new();
    
    Reflect::set(&layer, &JsValue::from_str("id"), &JsValue::from_str(id))?;
    Reflect::set(&layer, &JsValue::from_str("type"), &JsValue::from_str("symbol"))?;
    Reflect::set(&layer, &JsValue::from_str("source"), &JsValue::from_str(source))?;
    
    let layout = Object::new();
    let text_field = Array::new();
    text_field.push(&JsValue::from_str("get"));
    text_field.push(&JsValue::from_str("name"));
    
    Reflect::set(&layout, &JsValue::from_str("text-field"), &text_field)?;
    
    let font = Array::new();
    font.push(&JsValue::from_str("Noto Sans Regular"));
    Reflect::set(&layout, &JsValue::from_str("text-font"), &font)?;
    
    let offset = Array::new();
    offset.push(&JsValue::from_f64(0.0));
    offset.push(&JsValue::from_f64(1.5));
    Reflect::set(&layout, &JsValue::from_str("text-offset"), &offset)?;
    
    Reflect::set(&layout, &JsValue::from_str("text-anchor"), &JsValue::from_str("top"))?;
    Reflect::set(&layer, &JsValue::from_str("layout"), &layout)?;
    
    let paint = Object::new();
    Reflect::set(&paint, &JsValue::from_str("text-color"), &JsValue::from_str("#000000"))?;
    Reflect::set(&paint, &JsValue::from_str("text-halo-color"), &JsValue::from_str("#ffffff"))?;
    Reflect::set(&paint, &JsValue::from_str("text-halo-width"), &JsValue::from_f64(2.0))?;
    Reflect::set(&layer, &JsValue::from_str("paint"), &paint)?;
    
    Ok(layer.into())
}

// Create a ScaleControl configuration
pub fn create_scale_control_options() -> Result<JsValue, JsValue> {
    let options = Object::new();
    Reflect::set(&options, &JsValue::from_str("maxWidth"), &JsValue::from_f64(100.0))?;
    Reflect::set(&options, &JsValue::from_str("unit"), &JsValue::from_str("metric"))?;
    Ok(options.into())
}

/// Helper to add scripts to the page programmatically
pub fn load_script(src: &str, on_load: Option<Closure<dyn FnMut()>>) -> Result<(), JsValue> {
    let window = window().ok_or_else(|| JsValue::from_str("No global window exists"))?;
    let document = window.document().ok_or_else(|| JsValue::from_str("No document exists on window"))?;
    
    let script = document.create_element("script")?;
    script.set_attribute("src", src)?;
    
    if let Some(callback) = on_load {
        script.set_attribute("onload", "this.onload_callback()")?;
        
        // Store callback in the script element
        js_sys::Reflect::set(&script, &JsValue::from_str("onload_callback"), callback.as_ref())?;
        
        // IMPORTANT: Forget the closure to prevent it from being dropped
        // The callback will remain alive until the page is unloaded
        callback.forget();
    }
    
    document.head()
        .ok_or_else(|| JsValue::from_str("document should have head"))?
        .append_child(&script)?;
    
    Ok(())
}

// Helper to add CSS to the page programmatically
pub fn load_css(href: &str) -> Result<(), JsValue> {
    let window = window().ok_or_else(|| JsValue::from_str("No global window exists"))?;
    let document = window.document().ok_or_else(|| JsValue::from_str("No document exists on window"))?;
    
    let link = document.create_element("link")?;
    link.set_attribute("rel", "stylesheet")?;
    link.set_attribute("href", href)?;
    
    document.head()
        .ok_or_else(|| JsValue::from_str("document should have head"))?
        .append_child(&link)?;
    
    Ok(())
}

// Helper to add inline script to the page
pub fn add_inline_script(content: &str) -> Result<(), JsValue> {
    let window = window().ok_or_else(|| JsValue::from_str("No global window exists"))?;
    let document = window.document().ok_or_else(|| JsValue::from_str("No document exists on window"))?;
    
    let script = document.create_element("script")?;
    script.set_inner_html(content);
    
    document.head()
        .ok_or_else(|| JsValue::from_str("document should have head"))?
        .append_child(&script)?;
    
    Ok(())
}
