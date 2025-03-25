use dioxus::prelude::*;
use log::info;
use super::TflLayers;

#[component]
pub fn Canvas(layers: Signal<TflLayers>) -> Element {
    let map_id = "maplibre-canvas";
    let mut prev_layers = use_signal(|| None::<TflLayers>);

    // Create a use_effect to initialize the map after the component mounts
    use_effect(move || {
        info!("Canvas mounted, initializing TfL map");
        
        // Initialize the map with London coordinates
        initialize_map_libre(&map_id);

        // Check if layers changed
        let current = *layers.read();
        let mut prev = prev_layers.write();
        
        // If we have previous layers saved and they're different
        if let Some(old_layers) = *prev {
            if old_layers != current {
                info!("Layers state changed");
                update_layer_visibility(&current);
            }
        } else {
            // First time - initialize
            update_layer_visibility(&current);
        }
        
        // Save current layers for next comparison
        *prev = Some(current);
    });

    rsx! {
        div {
            id: "map-container",
            style: "width: 100%; height: 100vh; position: relative;",
            
            div {
                id: map_id,
                style: "position: absolute; top: 0; bottom: 0; width: 100%; height: 100%;"
            }
        }
    }
}

#[cfg(feature = "web")]
fn update_layer_visibility(layers: &TflLayers) {
    use wasm_bindgen::JsValue;
    
    // Convert our layers to a JavaScript object to pass to the map
    let js_code = format!(r#"
        if (window.mapInstance) {{
            // Update tube layer visibility
            if (window.mapInstance.getLayer('central-line-layer')) {{
                window.mapInstance.setLayoutProperty(
                    'central-line-layer', 
                    'visibility', 
                    {} ? 'visible' : 'none'
                );
            }}
            if (window.mapInstance.getLayer('northern-line-layer')) {{
                window.mapInstance.setLayoutProperty(
                    'northern-line-layer', 
                    'visibility', 
                    {} ? 'visible' : 'none'
                );
            }}
            
            // Update overground layer visibility
            if (window.mapInstance.getLayer('overground-line-layer')) {{
                window.mapInstance.setLayoutProperty(
                    'overground-line-layer', 
                    'visibility', 
                    {} ? 'visible' : 'none'
                );
            }}
            
            // Update stations layer visibility
            if (window.mapInstance.getLayer('stations-layer')) {{
                window.mapInstance.setLayoutProperty(
                    'stations-layer', 
                    'visibility', 
                    {} ? 'visible' : 'none'
                );
            }}
            if (window.mapInstance.getLayer('station-labels')) {{
                window.mapInstance.setLayoutProperty(
                    'station-labels', 
                    'visibility', 
                    {} ? 'visible' : 'none'
                );
            }}
        }}
    "#, 
        layers.tube, 
        layers.tube, 
        layers.overground, 
        layers.stations,
        layers.stations
    );
    
    let _ = js_sys::eval(&js_code);
}

#[cfg(not(feature = "web"))]
fn update_layer_visibility(_layers: &TflLayers) {
    // Do nothing on non-web targets
}

#[cfg(feature = "web")]
fn initialize_map_libre(map_id: &str) {
    use wasm_bindgen::{JsCast, JsValue};

    let window = web_sys::window().expect("no global window exists");
    let document = window.document().expect("no document exists on window");
    
    // Add MapLibre CSS
    let head = document.head().expect("document should have head");
    let link = document.create_element("link").expect("could not create link element");
    link.set_attribute("rel", "stylesheet").expect("could not set attribute");
    link.set_attribute("href", "https://unpkg.com/maplibre-gl@3.6.2/dist/maplibre-gl.css").expect("could not set attribute");
    head.append_child(&link).expect("could not append child");
    
    // Add MapLibre script and initialize map
    let script = document.create_element("script").expect("could not create script element");
    script.set_attribute("src", "https://unpkg.com/maplibre-gl@3.6.2/dist/maplibre-gl.js").expect("could not set attribute");
    
    // Set up an onload handler
    let map_container_id = map_id.to_string();
    let onload_handler = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
        let init_code = format!(include_str!("./js/map_init.js"), map_container_id);
        
        let _ = js_sys::eval(&init_code);
    }) as Box<dyn FnMut()>);
    
    script.set_attribute("onload", "window.onloadHandler()").expect("could not set onload");
    // Fix: Use Reflect.set instead of the global().set method
    js_sys::Reflect::set(
        &js_sys::global(),
        &JsValue::from_str("onloadHandler"),
        &onload_handler.as_ref().unchecked_ref()
    ).expect("could not set global onload handler");
    onload_handler.forget(); // Prevent the closure from being dropped
    
    document.body().expect("document should have body")
        .append_child(&script)
        .expect("could not append script to body");
}

#[cfg(not(feature = "web"))]
fn initialize_map_libre(_map_id: &str) {
    // Do nothing on non-web targets
}
