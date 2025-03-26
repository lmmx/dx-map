use super::TflLayers;
use dioxus::prelude::*;

#[component]
pub fn LayerPanel(visible: bool, layers: Signal<TflLayers>, on_close: EventHandler<()>) -> Element {
    rsx! {
        div {
            class: if visible { "layer-switcher-list active" } else { "layer-switcher-list" },

            h3 { "Layers" }

            h4 { "Background" }
            div {
                class: "layer-item",
                input {
                    r#type: "checkbox",
                    id: "nighttime_lights",
                    name: "nighttime_lights"
                }
                label {
                    r#for: "nighttime_lights",
                    "Nighttime Lights"
                }
            }
            div {
                class: "layer-item",
                input {
                    r#type: "checkbox",
                    id: "labels",
                    name: "labels",
                    checked: true,
                }
                label {
                    r#for: "labels",
                    "Labels"
                }
            }

            h4 { "Transport" }

            div {
                class: "layer-item",
                input {
                    r#type: "checkbox",
                    id: "tube",
                    name: "tube",
                    checked: layers.read().tube,
                    onchange: move |_| {
                        let mut updated = *layers.read();
                        updated.tube = !updated.tube;
                        layers.set(updated);
                    }
                }
                label {
                    r#for: "tube",
                    "Underground"
                }
            }

            div {
                class: "layer-item",
                input {
                    r#type: "checkbox",
                    id: "overground",
                    name: "overground",
                    checked: layers.read().overground,
                    onchange: move |_| {
                        let mut updated = *layers.read();
                        updated.overground = !updated.overground;
                        layers.set(updated);
                    }
                }
                label {
                    r#for: "overground",
                    "Overground"
                }
            }

            div {
                class: "layer-item",
                input {
                    r#type: "checkbox",
                    id: "dlr",
                    name: "dlr",
                    checked: layers.read().dlr,
                    onchange: move |_| {
                        let mut updated = *layers.read();
                        updated.dlr = !updated.dlr;
                        layers.set(updated);
                    }
                }
                label {
                    r#for: "dlr",
                    "DLR"
                }
            }

            div {
                class: "layer-item",
                input {
                    r#type: "checkbox",
                    id: "elizabeth_line",
                    name: "elizabeth_line",
                    checked: layers.read().elizabeth_line,
                    onchange: move |_| {
                        let mut updated = *layers.read();
                        updated.elizabeth_line = !updated.elizabeth_line;
                        layers.set(updated);
                    }
                }
                label {
                    r#for: "elizabeth_line",
                    "Elizabeth Line"
                }
            }

            div {
                class: "layer-item",
                input {
                    r#type: "checkbox",
                    id: "buses",
                    name: "buses",
                    checked: layers.read().buses,
                    onchange: move |_| {
                        let mut updated = *layers.read();
                        updated.buses = !updated.buses;
                        layers.set(updated);
                    }
                }
                label {
                    r#for: "buses",
                    "Buses"
                }
            }

            div {
                class: "layer-item",
                input {
                    r#type: "checkbox",
                    id: "trams",
                    name: "trams",
                    checked: layers.read().trams,
                    onchange: move |_| {
                        let mut updated = *layers.read();
                        updated.trams = !updated.trams;
                        layers.set(updated);
                    }
                }
                label {
                    r#for: "trams",
                    "Trams"
                }
            }

            div {
                class: "layer-item",
                input {
                    r#type: "checkbox",
                    id: "cable_car",
                    name: "cable_car",
                    checked: layers.read().cable_car,
                    onchange: move |_| {
                        let mut updated = *layers.read();
                        updated.cable_car = !updated.cable_car;
                        layers.set(updated);
                    }
                }
                label {
                    r#for: "cable_car",
                    "Cable Car"
                }
            }

            h4 { "Infrastructure" }

            div {
                class: "layer-item",
                input {
                    r#type: "checkbox",
                    id: "stations",
                    name: "stations",
                    checked: layers.read().stations,
                    onchange: move |_| {
                        let mut updated = *layers.read();
                        updated.stations = !updated.stations;
                        layers.set(updated);
                    }
                }
                label {
                    r#for: "stations",
                    "Stations"
                }
            }

            div {
                class: "layer-item",
                input {
                    r#type: "checkbox",
                    id: "depots",
                    name: "depots",
                    checked: layers.read().depots,
                    onchange: move |_| {
                        let mut updated = *layers.read();
                        updated.depots = !updated.depots;
                        layers.set(updated);
                    }
                }
                label {
                    r#for: "depots",
                    "Depots & Facilities"
                }
            }

            h4 { "Simulation" }

            div {
                class: "layer-item",
                input {
                    r#type: "checkbox",
                    id: "simulation",
                    name: "simulation",
                    checked: layers.read().simulation,
                    onchange: move |_| {
                        let mut updated = *layers.read();
                        updated.simulation = !updated.simulation;
                        layers.set(updated);

                        // Update visibility of simulation layers via JS
                        let js_code = format!(
                            r#"
                            if (window.mapInstance) {{
                                const visibility = {} ? 'visible' : 'none';
                                if (window.mapInstance.getLayer('buses-layer')) {{
                                    window.mapInstance.setLayoutProperty('buses-layer', 'visibility', visibility);
                                }}
                                if (window.mapInstance.getLayer('trains-layer')) {{
                                    window.mapInstance.setLayoutProperty('trains-layer', 'visibility', visibility);
                                }}
                            }}
                            "#,
                            updated.simulation
                        );
                        let _ = js_sys::eval(&js_code);
                    }
                }
                label {
                    r#for: "simulation",
                    "Vehicle Simulation"
                }
            }

            button {
                class: "close-button",
                onclick: move |_| on_close.call(()),
                "Close"
            }
        }
    }
}
