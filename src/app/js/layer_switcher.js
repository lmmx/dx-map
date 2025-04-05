/**
 * Implementation of the LayerSwitcher class for MapLibre GL JS
 * Based on https://github.com/russss/maplibregl-layer-switcher
 */
class LayerSwitcher {
  constructor(layers, title = 'Layers') {
    this._layers = layers;
    this._layerIndex = {};

    // Index all layers for quick lookup
    for (let layer of this.getLayers()) {
      if (this._layerIndex[layer.id]) {
        console.error(`Duplicate layer ID "${layer.id}". Layer IDs must be unique.`);
      }
      this._layerIndex[layer.id] = layer;
    }

    // Initialize visible layers based on their enabled status
    this._visible = this._default_visible = Object.values(this._layerIndex)
      .filter(layer => layer.enabled)
      .map(layer => layer.id);

    // Create DOM elements
    this._layerList = document.createElement('ul');
    this._container = document.createElement('div');
    this._container.className = 'layer-switcher-list';

    const titleElement = document.createElement('h3');
    titleElement.textContent = title;

    this._container.appendChild(titleElement);
    this._container.appendChild(this._layerList);

    // Append to document body for positioning
    document.body.appendChild(this._container);
    // Store instance for retrieval
    LayerSwitcher._instance = this;
  }

  // Add a static method to get the instance
  static getInstance() {
    return LayerSwitcher._instance;
  }

  // Extract flat list of layers from layer groups
  getLayers() {
    const layers = [];
    for (let item of this._layers) {
      if (item instanceof LayerGroup) {
        layers.push(...item.layers);
      } else if (item instanceof Layer) {
        layers.push(item);
      }
    }
    return layers;
  }

  // Set visibility of a specific layer
  setVisibility(layerId, visible) {
    if (visible) {
      if (!this._visible.includes(layerId)) {
        this._visible.push(layerId);
      }
    } else {
      this._visible = this._visible.filter(id => id !== layerId);
    }

    this._updateVisibility();
    this._updateList();
  }

  // Update visibility of all layers in the map
  _updateVisibility() {
    if (!this._map) {
      return;
    }

    const layers = this._map.getStyle().layers;
    for (let layer of layers) {
      const layerId = layer.id;

      for (let configLayerId in this._layerIndex) {
        const prefix = this._layerIndex[configLayerId].prefix;
        if (layerId.startsWith(prefix)) {
          const visibility = this._visible.includes(configLayerId) ? 'visible' : 'none';
          this._map.setLayoutProperty(layerId, 'visibility', visibility);
        }
      }
    }
  }

  // Set initial visibility in the style before the map is created
  setInitialVisibility(style) {
    for (let layer of style.layers) {
      for (let configLayerId in this._layerIndex) {
        const prefix = this._layerIndex[configLayerId].prefix;
        if (layer.id.startsWith(prefix) && !this._visible.includes(configLayerId)) {
          if (!layer.layout) {
            layer.layout = {};
          }
          layer.layout.visibility = 'none';
        }
      }
    }
    this._updateList();
  }

  // MapLibre IControl implementation
  onAdd(map) {
    this._map = map;

    // Initialize visibility when the style is loaded
    if (map.isStyleLoaded()) {
      console.log("LAYER SWITCHER ADDING AS MAP STYLE LOADED")
      this._updateVisibility();
    } else {
      map.on('load', () => {
        console.log("LAYER SWITCHER ADDING AS MAP LOADED")
        this._updateVisibility();
      });
    }

    // Create the control button
    const button = document.createElement('button');
    button.className = 'layer-switcher-button';
    button.setAttribute('aria-label', 'Layer Switcher');

    // Set up event listeners
    button.addEventListener('click', () => {
      this._container.classList.toggle('active');
      this._updateList();
    });

    // Hide the panel when clicking outside
    document.addEventListener('click', (e) => {
      if (!this._container.contains(e.target) &&
          !e.target.classList.contains('layer-switcher-button')) {
        this._container.classList.remove('active');
      }
    });

    // Create container for the button
    const controlContainer = document.createElement('div');
    controlContainer.className = 'maplibregl-ctrl maplibregl-ctrl-group layer-switcher';
    controlContainer.appendChild(button);

    this._updateList();

    return controlContainer;
  }

  onRemove() {
    if (this._container.parentNode) {
      this._container.parentNode.removeChild(this._container);
    }
    this._map = undefined;
  }

  // Create a DOM element for a layer
  _getLayerElement(item) {
    if (item instanceof Layer) {
      const li = document.createElement('li');
      li.className = 'layer-item';

      const checkbox = document.createElement('input');
      checkbox.type = 'checkbox';
      checkbox.id = `layer-${item.id}`;
      checkbox.checked = this._visible.includes(item.id);

      checkbox.addEventListener('change', (e) => {
        this.setVisibility(item.id, e.target.checked);
      });

      const label = document.createElement('label');
      label.htmlFor = `layer-${item.id}`;
      label.textContent = item.title;

      li.appendChild(checkbox);
      li.appendChild(label);

      return li;
    } else if (item instanceof LayerGroup) {
      const li = document.createElement('li');
      li.className = 'layer-group';

      const heading = document.createElement('h4');
      heading.textContent = item.title;
      li.appendChild(heading);

      const ul = document.createElement('ul');
      for (let layer of item.layers) {
        ul.appendChild(this._getLayerElement(layer));
      }

      li.appendChild(ul);
      return li;
    } else {
      console.error('Unknown item type:', item);
      return document.createElement('li');
    }
  }

  // Update the layer list display
  _updateList() {
    // Clear existing items
    while (this._layerList.firstChild) {
      this._layerList.removeChild(this._layerList.firstChild);
    }

    // Add new items
    for (let item of this._layers) {
      this._layerList.appendChild(this._getLayerElement(item));
    }
  }
}

// Layer class for individual layers
class Layer {
  constructor(id, title, prefix, enabled = false) {
    this.id = id;
    this.title = title;
    this.prefix = prefix;
    this.enabled = enabled;
  }
}

// LayerGroup class for groups of layers
class LayerGroup {
  constructor(title, layers) {
    this.title = title;
    this.layers = layers;
  }
}

LayerSwitcher._instance = null;

// Export the control constructor and helper classes
window.LayerSwitcher = LayerSwitcher;
window.Layer = Layer;
window.LayerGroup = LayerGroup;
