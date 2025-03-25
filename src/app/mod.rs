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
            h1 { "Hello World Map" }
            nav {
                ul {

                }
            }
        }

        Canvas { }
    }
}
