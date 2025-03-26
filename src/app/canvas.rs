use dioxus::prelude::*;
use log::info;

use crate::maplibre::MapLibreManager;
use super::TflLayers;

#[component]
pub fn Canvas(layers: Signal<TflLayers>) -> Element {
    let map_id = "maplibre-canvas";
    let mut prev_layers = use_signal(|| None::<TflLayers>);
    
    // Create a manager reference that can be shared
    let map_manager = use_ref(|| None::<MapLibreManager>);
    
    // First-time initialization
    use_effect(move || {
        info!("Canvas mounted, initializing TfL map");
        
        // Only initialize if not already done
        if map_manager.read().is_none() {
            let mut manager = MapLibreManager::new();
            if let Err(err) = manager.initialize(map_id) {
                info!("Error initializing map: {:?}", err);
                return;
            }
            
            *map_manager.write() = Some(manager);
            info!("MapLibre manager initialized");
        }
    });
    
    // Effect to handle layer changes
    use_effect(move || {
        let current = *layers.read();
        let mut prev = prev_layers.write();
        
        // If we have previous layers saved and they're different
        if let Some(old_layers) = *prev {
            if old_layers != current {
                info!("Layers state changed");
                
                // Update the map via the manager
                map_manager.with_mut(|manager| {
                    if let Some(manager) = manager {
                        if let Err(err) = manager.update_layer_visibility(&current) {
                            info!("Error updating layer visibility: {:?}", err);
                        }
                    }
                });
            }
        } else {
            // First time - initialize layers when map is ready
            map_manager.with_mut(|manager| {
                if let Some(manager) = manager {
                    if let Err(err) = manager.update_layer_visibility(&current) {
                        info!("Error initializing layer visibility: {:?}", err);
                    }
                }
            });
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
