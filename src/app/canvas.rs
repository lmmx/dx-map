use dioxus::prelude::*;

/// Here is the canvas for the Map
/// We’ll set a fixed size just for demonstration
#[component]
pub fn Canvas() -> Element {
    rsx! {
        canvas {
            width: "800",
            height: "600",
            style: "border: 2px solid white;",
        }
    }
}
