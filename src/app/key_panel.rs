use dioxus::prelude::*;
use crate::data::line_definitions::{get_underground_lines, get_other_rail_lines};

#[component]
pub fn KeyPanel(visible: bool, on_close: EventHandler<()>) -> Element {
    let underground_lines = get_underground_lines();
    let other_rail_lines = get_other_rail_lines();

    rsx! {
        div {
            class: if visible { "oim-key-panel visible" } else { "oim-key-panel" },

            div {
                class: "oim-key-header",
                h2 { "Key" }
                button {
                    class: "oim-key-close",
                    onclick: move |_| on_close.call(()),
                    "×"
                }
            }

            div {
                class: "oim-key-body",

                h3 { "Underground Lines" }
                table {
                    // Dynamically generate rows for underground lines
                    for line in &underground_lines {
                        tr {
                            td { "{line.name}" }
                            td {
                                div {
                                    class: format_args!("color-line {}", line.id)
                                }
                            }
                        }
                    }
                }

                h3 { "Other Rail" }
                table {
                    // Dynamically generate rows for other rail lines
                    for line in &other_rail_lines {
                        tr {
                            td { "{line.name}" }
                            td {
                                div {
                                    class: format_args!("color-line {}", line.id)
                                }
                            }
                        }
                    }
                }

                h3 { "Other Features" }
                table {
                    tr {
                        td { "Station" }
                        td {
                            div {
                                class: "map-symbol station"
                            }
                        }
                    }
                    tr {
                        td { "Interchange" }
                        td {
                            div {
                                class: "map-symbol interchange"
                            }
                        }
                    }
                    tr {
                        td { "Depot" }
                        td {
                            div {
                                class: "map-symbol depot"
                            }
                        }
                    }
                }
            }
        }
    }
}
