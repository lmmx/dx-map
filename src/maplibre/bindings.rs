use wasm_bindgen::prelude::*;
use web_sys::{HtmlElement, Element};

// Define TypeScript-like bindings for MapLibre GL JS

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = maplibregl, js_name = Map)]
    pub type MapLibreMap;

    #[wasm_bindgen(constructor, js_namespace = maplibregl, js_name = Map)]
    pub fn new(options: &JsValue) -> MapLibreMap;

    #[wasm_bindgen(method, js_name = getContainer)]
    pub fn get_container(this: &MapLibreMap) -> HtmlElement;

    #[wasm_bindgen(method, js_name = getCanvas)]
    pub fn get_canvas(this: &MapLibreMap) -> Element;

    #[wasm_bindgen(method)]
    pub fn addControl(this: &MapLibreMap, control: &JsValue, position: Option<&str>) -> MapLibreMap;

    #[wasm_bindgen(method, js_name = setLayoutProperty)]
    pub fn set_layout_property(
        this: &MapLibreMap,
        layer_id: &str,
        name: &str,
        value: &JsValue,
    ) -> MapLibreMap;

    #[wasm_bindgen(method, js_name = getLayer)]
    pub fn get_layer_raw(this: &MapLibreMap, id: &str) -> JsValue;

    #[wasm_bindgen(method, js_name = addSource)]
    pub fn add_source(this: &MapLibreMap, id: &str, source: &JsValue) -> MapLibreMap;

    #[wasm_bindgen(method, js_name = addLayer)]
    pub fn add_layer(this: &MapLibreMap, layer: &JsValue) -> MapLibreMap;

    #[wasm_bindgen(method)]
    pub fn on(this: &MapLibreMap, event: &str, handler: &Closure<dyn FnMut()>) -> MapLibreMap;

    #[wasm_bindgen(method)]
    pub fn off(this: &MapLibreMap, event: &str, handler: &Closure<dyn FnMut()>) -> MapLibreMap;

    // MapLibre Controls
    #[wasm_bindgen(js_namespace = maplibregl, js_name = NavigationControl)]
    pub type NavigationControl;

    #[wasm_bindgen(constructor, js_namespace = maplibregl, js_name = NavigationControl)]
    pub fn new() -> NavigationControl;

    #[wasm_bindgen(js_namespace = maplibregl, js_name = ScaleControl)]
    pub type ScaleControl;

    #[wasm_bindgen(constructor, js_namespace = maplibregl, js_name = ScaleControl)]
    pub fn new(options: &JsValue) -> ScaleControl;

    // Our custom controls
    #[wasm_bindgen(js_namespace = window, js_name = KeyControl)]
    pub type KeyControl;

    #[wasm_bindgen(constructor, js_namespace = window, js_name = KeyControl)]
    pub fn new() -> KeyControl;

    #[wasm_bindgen(js_namespace = window, js_name = LayerSwitcher)]
    pub type LayerSwitcher;

    #[wasm_bindgen(constructor, js_namespace = window, js_name = LayerSwitcher)]
    pub fn new(layers: &JsValue, title: &str) -> LayerSwitcher;

    // Layer/Group classes
    #[wasm_bindgen(js_namespace = window, js_name = Layer)]
    pub type MapLayer;

    #[wasm_bindgen(constructor, js_namespace = window, js_name = Layer)]
    pub fn new(id: &str, title: &str, prefix: &str, enabled: bool) -> MapLayer;

    #[wasm_bindgen(js_namespace = window, js_name = LayerGroup)]
    pub type MapLayerGroup;

    #[wasm_bindgen(constructor, js_namespace = window, js_name = LayerGroup)]
    pub fn new(title: &str, layers: &JsValue) -> MapLayerGroup;
}

impl MapLibreMap {
    pub fn get_layer(&self, id: &str) -> Option<JsValue> {
        let raw = self.get_layer_raw(id);
        if raw.is_null() || raw.is_undefined() {
            None
        } else {
            Some(raw)
        }
    }
}

// Helper to access the global MapLibre instance
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window)]
    pub static mapInstance: Option<MapLibreMap>;
}
