use std::thread::Scope;
use dioxus::prelude::*;
use web_sys::HtmlCanvasElement;

// For logging and better errors in WASM
use log::Level;
use console_log;
use console_error_panic_hook;

use maplibre::render::settings::WgpuSettings;
use maplibre::{
    io::scheduler::Scheduler,
    io::source_client::HttpSourceClient,
    map::Map,
    style::Style,
};

// If you have images or CSS as assets, define them with Dioxus' asset! macro
const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

// Entry point for Dioxus
fn main() {
    // If you are building a web app, typically you'd do:
    // dioxus_web::launch(App)
    // or use the dioxus-cli "serve" command
    //
    // For demonstration, just do the normal launch (which also can run in a browser with the CLI).
    console_error_panic_hook::set_once();
    console_log::init_with_level(Level::Info).expect("error initializing logger");

    launch(app)
}

// Your top-level Dioxus component
#[component]
fn app() -> Element {
    // Create a NodeRef so we can get the underlying <canvas> HTML element in Rust
    // let canvas_ref = use_node_ref(cx);

    // // Once the component is mounted, initialize the map exactly once
    // use_effect(cx, (), move |_| {
    //     let canvas_ref = canvas_ref.clone();

    //     async move {
    //         // Grab the <canvas> as an HtmlCanvasElement
    //         if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
    //             // Kick off the map initialization
    //             if let Err(e) = init_map(canvas).await {
    //                 log::error!("Map init failed: {:?}", e);
    //             }
    //         }
    //     }
    // });

    rsx! {
        // You can keep your existing "Hero" or other components
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        header {
            h1 { "Hello World Map" }
            nav {
                ul {

                }
            }
        }

        // Your "hero" section
        div {
            id: "hero",
            img { src: HEADER_SVG, id: "header" }
        }

        // Here is the canvas for the Map
        // We’ll set a fixed size just for demonstration
        // canvas {
        //     ref: canvas_ref,
        //     width: "800",
        //     height: "600",
        //     style: "border: 1px solid black;",
        // }
    }
}

// /// Initialize the MapLibre-rs map inside the given HtmlCanvasElement.
// /// This runs entirely in Rust, compiled to WASM + WebGL.
// async fn init_map(canvas: HtmlCanvasElement) -> Result<(), Box<dyn std::error::Error>> {
//     log::info!("Initializing map...");
// 
//     // Create some style.  You can use a local style or a remote style JSON. 
//     let style_url = "https://demotiles.maplibre.org/style.json";
//     let style = Style::from_url(style_url.to_string());
// 
//     // Prepare a simple HTTP tile client
//     let scheduler = Scheduler::new();
//     let http_client = HttpSourceClient::new(None);
// 
//     // Construct the map
//     let mut map = Map::new(
//         canvas,
//         scheduler,
//         http_client,
//         WgpuSettings::default(), // For WebGL usage on the web
//     )
//     .await?;
// 
//     // Centre on London
//     map.set_center(maplibre::coords::LatLon::new(51.5074, -0.1278));
//     map.set_zoom(11.0);
// 
//     // Start the main map rendering/event loop
//     log::info!("Running the map event loop...");
//     map.run().await;
// 
//     Ok(())
// }
