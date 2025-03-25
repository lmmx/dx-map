use dioxus::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use log::info;

#[component]
pub fn Canvas() -> Element {
    let map_id = "maplibre-canvas";

    // Create a use_effect to initialize the map after the component mounts
    use_effect(move || {
        info!("Canvas mounted, initializing map");
        
        // Run initialization code after the component has mounted
        let init_script = include_str!("map_init.js");
        
        initialize_map_libre(&map_id);

        // // Cleanup function
        // || {
        //     info!("Canvas unmounting, cleanup would go here");
        // }
    });

    rsx! {
        div {
            id: "map-container",
            style: "width: 100%; height: 600px; position: relative;",
            
            div {
                id: map_id,
                style: "position: absolute; top: 0; bottom: 0; width: 100%; height: 100%;"
            }
        }
    }
}
#[cfg(feature = "web")]
fn initialize_map_libre(map_id: &str) {
    use wasm_bindgen::JsCast;

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
        let init_code = format!(r#"
            try {{
                const map = new maplibregl.Map({{
                    container: '{}',
                    style: 'https://demotiles.maplibre.org/style.json',
                    center: [0, 0],
                    zoom: 1
                }});
                map.addControl(new maplibregl.NavigationControl());
                window.mapInstance = map;
            }} catch(e) {{
                console.error('Failed to initialize MapLibre map:', e);
            }}
        "#, map_container_id);
        
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
