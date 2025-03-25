use dioxus::prelude::*;

#[component]
pub fn KeyPanel(
    visible: bool,
    on_close: EventHandler<()>
) -> Element {
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
                    tr {
                        td { "Bakerloo" }
                        td { 
                            div {
                                class: "color-line bakerloo"
                            }
                        }
                    }
                    tr {
                        td { "Central" }
                        td { 
                            div {
                                class: "color-line central"
                            }
                        }
                    }
                    tr {
                        td { "Circle" }
                        td { 
                            div {
                                class: "color-line circle"
                            }
                        }
                    }
                    tr {
                        td { "District" }
                        td { 
                            div {
                                class: "color-line district"
                            }
                        }
                    }
                    tr {
                        td { "Hammersmith & City" }
                        td { 
                            div {
                                class: "color-line hammersmith"
                            }
                        }
                    }
                    tr {
                        td { "Jubilee" }
                        td { 
                            div {
                                class: "color-line jubilee"
                            }
                        }
                    }
                    tr {
                        td { "Metropolitan" }
                        td { 
                            div {
                                class: "color-line metropolitan"
                            }
                        }
                    }
                    tr {
                        td { "Northern" }
                        td { 
                            div {
                                class: "color-line northern"
                            }
                        }
                    }
                    tr {
                        td { "Piccadilly" }
                        td { 
                            div {
                                class: "color-line piccadilly"
                            }
                        }
                    }
                    tr {
                        td { "Victoria" }
                        td { 
                            div {
                                class: "color-line victoria"
                            }
                        }
                    }
                    tr {
                        td { "Waterloo & City" }
                        td { 
                            div {
                                class: "color-line waterloo"
                            }
                        }
                    }
                }
                
                h3 { "Other Rail" }
                table {
                    tr {
                        td { "Overground" }
                        td { 
                            div {
                                class: "color-line overground"
                            }
                        }
                    }
                    tr {
                        td { "DLR" }
                        td { 
                            div {
                                class: "color-line dlr"
                            }
                        }
                    }
                    tr {
                        td { "Elizabeth Line" }
                        td { 
                            div {
                                class: "color-line elizabeth"
                            }
                        }
                    }
                    tr {
                        td { "Trams" }
                        td { 
                            div {
                                class: "color-line tram"
                            }
                        }
                    }
                    tr {
                        td { "Cable Car" }
                        td { 
                            div {
                                class: "color-line cablecar"
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