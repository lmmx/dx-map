// TfL Infrastructure Map initialization

function initMap(containerId) {
    if (typeof maplibregl === 'undefined') {
        console.error('MapLibre GL JS not loaded');
        return;
    }
    
    try {
        const map = new maplibregl.Map({
            container: containerId,
            style: 'https://demotiles.maplibre.org/style.json',
            center: [-0.1276, 51.5072], // London
            zoom: 12,
            maxBounds: [
                [-0.489, 51.28], // SW
                [0.236, 51.686]  // NE
            ]
        });
        
        // Add navigation controls
        map.addControl(new maplibregl.NavigationControl());
        
        // Add scale control
        map.addControl(new maplibregl.ScaleControl({
            maxWidth: 100,
            unit: 'metric'
        }), 'bottom-left');
        
        // Store map instance for later access
        window.mapInstance = map;
        
        // Add TfL layers when map loads
        map.on('load', function() {
            // Add placeholder for Central line
            map.addSource('central-line', {
                'type': 'geojson',
                'data': {
                    'type': 'Feature',
                    'properties': {},
                    'geometry': {
                        'type': 'LineString',
                        'coordinates': [
                            [-0.22, 51.51],
                            [-0.18, 51.52],
                            [-0.14, 51.515],
                            [-0.10, 51.52],
                            [-0.05, 51.52]
                        ]
                    }
                }
            });
            
            map.addLayer({
                'id': 'central-line-layer',
                'type': 'line',
                'source': 'central-line',
                'layout': {
                    'line-join': 'round',
                    'line-cap': 'round'
                },
                'paint': {
                    'line-color': '#DC241F',
                    'line-width': 4
                }
            });
            
            // Add placeholder for Northern line
            map.addSource('northern-line', {
                'type': 'geojson',
                'data': {
                    'type': 'Feature',
                    'properties': {},
                    'geometry': {
                        'type': 'LineString',
                        'coordinates': [
                            [-0.15, 51.48],
                            [-0.12, 51.50],
                            [-0.12, 51.53],
                            [-0.14, 51.55]
                        ]
                    }
                }
            });
            
            map.addLayer({
                'id': 'northern-line-layer',
                'type': 'line',
                'source': 'northern-line',
                'layout': {
                    'line-join': 'round',
                    'line-cap': 'round'
                },
                'paint': {
                    'line-color': '#000000',
                    'line-width': 4
                }
            });
            
            // Add placeholder for Overground
            map.addSource('overground-line', {
                'type': 'geojson',
                'data': {
                    'type': 'Feature',
                    'properties': {},
                    'geometry': {
                        'type': 'LineString',
                        'coordinates': [
                            [-0.20, 51.53],
                            [-0.16, 51.54],
                            [-0.10, 51.54],
                            [-0.05, 51.55]
                        ]
                    }
                }
            });
            
            map.addLayer({
                'id': 'overground-line-layer',
                'type': 'line',
                'source': 'overground-line',
                'layout': {
                    'line-join': 'round',
                    'line-cap': 'round'
                },
                'paint': {
                    'line-color': '#EE7C0E',
                    'line-width': 4
                }
            });
            
            // Add placeholder stations
            map.addSource('stations', {
                'type': 'geojson',
                'data': {
                    'type': 'FeatureCollection',
                    'features': [
                        {
                            'type': 'Feature',
                            'properties': { 'name': 'Oxford Circus' },
                            'geometry': {
                                'type': 'Point',
                                'coordinates': [-0.1418, 51.5152]
                            }
                        },
                        {
                            'type': 'Feature',
                            'properties': { 'name': 'Kings Cross' },
                            'geometry': {
                                'type': 'Point',
                                'coordinates': [-0.1234, 51.5308]
                            }
                        },
                        {
                            'type': 'Feature',
                            'properties': { 'name': 'Liverpool Street' },
                            'geometry': {
                                'type': 'Point',
                                'coordinates': [-0.0827, 51.5178]
                            }
                        }
                    ]
                }
            });
            
            map.addLayer({
                'id': 'stations-layer',
                'type': 'circle',
                'source': 'stations',
                'paint': {
                    'circle-radius': 6,
                    'circle-color': '#ffffff',
                    'circle-stroke-color': '#000000',
                    'circle-stroke-width': 2
                }
            });
            
            map.addLayer({
                'id': 'station-labels',
                'type': 'symbol',
                'source': 'stations',
                'layout': {
                    'text-field': ['get', 'name'],
                    'text-font': ['Open Sans Regular'],
                    'text-offset': [0, 1.5],
                    'text-anchor': 'top'
                },
                'paint': {
                    'text-color': '#000000',
                    'text-halo-color': '#ffffff',
                    'text-halo-width': 2
                }
            });
        });
        
        return map;
    } catch (e) {
        console.error('Failed to initialize TfL map:', e);
        return null;
    }
}

// Function to update layer visibility - can be called from Rust
function updateLayerVisibility(layers) {
    if (!window.mapInstance) return;
    
    const map = window.mapInstance;
    
    // Update Underground layers
    if (map.getLayer('central-line-layer')) {
        map.setLayoutProperty(
            'central-line-layer',
            'visibility',
            layers.tube ? 'visible' : 'none'
        );
    }
    
    if (map.getLayer('northern-line-layer')) {
        map.setLayoutProperty(
            'northern-line-layer',
            'visibility',
            layers.tube ? 'visible' : 'none'
        );
    }
    
    // Update Overground layers
    if (map.getLayer('overground-line-layer')) {
        map.setLayoutProperty(
            'overground-line-layer',
            'visibility',
            layers.overground ? 'visible' : 'none'
        );
    }
    
    // Update station layers
    if (map.getLayer('stations-layer')) {
        map.setLayoutProperty(
            'stations-layer',
            'visibility',
            layers.stations ? 'visible' : 'none'
        );
    }
    
    if (map.getLayer('station-labels')) {
        map.setLayoutProperty(
            'station-labels',
            'visibility',
            layers.stations ? 'visible' : 'none'
        );
    }
}