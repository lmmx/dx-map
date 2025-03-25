// This file is included in the Canvas component for map initialization
// It's not actually used directly but shows what JS is being executed

function initMap(containerId) {
    if (typeof maplibregl === 'undefined') {
        console.error('MapLibre GL JS not loaded');
        return;
    }
    
    try {
        const map = new maplibregl.Map({
            container: containerId,
            style: 'https://demotiles.maplibre.org/style.json', // Default demo style
            center: [0, 0],
            zoom: 1
        });
        
        map.addControl(new maplibregl.NavigationControl());
        
        // Store map instance for later access if needed
        window.mapInstance = map;
        
        return map;
    } catch (e) {
        console.error('Failed to initialize MapLibre map:', e);
        return null;
    }
}
