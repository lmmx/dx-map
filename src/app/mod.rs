use dioxus::prelude::*;

mod canvas;
use canvas::Canvas;

// If you have images or CSS as assets, define them with Dioxus' asset! macro
const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

#[component]
pub fn app() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        header {
            class: "app-header",
            h1 { "MapLibre GL JS with Dioxus" }
            p { "Interactive map powered by MapLibre GL" }
        }

        main {
            class: "app-content",
            Canvas { }
        }

        footer {
            class: "app-footer",
            p { "Built with Dioxus and MapLibre GL JS" }
        }
    }
}