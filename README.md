# dx-map

A simple integration of MapLibre GL JS with Dioxus - a reactive UI framework for Rust.

![MapLibre GL JS with Dioxus](https://github.com/user-attachments/assets/df55c43f-053c-4535-ba5d-0762b0c06055)

## Overview

This project demonstrates how to integrate MapLibre GL JS with Dioxus to create an interactive map application compiled to WebAssembly. It serves as a starting point for developers looking to build map-based applications using Rust for the web.

## Features

- 🌍 Interactive world map with navigation controls
- 🦀 Written in Rust, compiled to WebAssembly
- 🔄 Reactive UI components with Dioxus

## Getting Started

### Prerequisites

You'll need the following installed:

- [Rust](https://www.rust-lang.org/tools/install) (stable)
- [Dioxus CLI](https://dioxuslabs.com/docs/0.4/guide/en/installation.html)

For building Dioxus projects:
```bash
sudo apt install libgdk3.0-cil libatk1.0-dev libcairo2-dev libpango1.0-dev libgdk-pixbuf2.0-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev libwebkit2gtk-4.1-dev libxdo-dev -y
```

### Running the Application

Run the development server:

```bash
dx serve --platform web
```

This will build the project and serve it at http://localhost:8080 by default.

## Project Structure

```
dx-map/
├── Cargo.toml           # Dependencies and project configuration
├── Dioxus.toml          # Dioxus configuration
├── src/
│   ├── main.rs          # Application entry point
│   ├── app/
│   │   ├── mod.rs       # Main application component
│   │   ├── canvas.rs    # MapLibre canvas component
│   │   └── map_init.js  # JavaScript helper for map initialization
│   └── assets/          # Static assets like CSS and favicons
└── README.md
```

## How It Works

The application works by:

1. Setting up a Dioxus component structure
2. Creating a container for the MapLibre GL map
3. Loading MapLibre GL JS and CSS dynamically
4. Initializing the map when the component mounts
5. Using wasm-bindgen to interact between Rust and JavaScript

The key integration happens in the `Canvas` component, which:
- Creates a properly sized container for the map
- Loads MapLibre's required CSS and JavaScript
- Uses JavaScript interop to initialize the map
- Sets up event handlers between Rust and JavaScript

## Customization

### Changing Map Style

You can modify the map style by changing the `style` URL in the initialization code:

```rust
// Inside the initialize_map_libre function
let init_code = format!(r#"
    try {{
        const map = new maplibregl.Map({{
            container: '{}',
            // Change the style URL below to use a different map style
            style: 'https://demotiles.maplibre.org/style.json',
            center: [0, 0],
            zoom: 1
        }});
        // ...
    }}
"#, map_container_id);
```

### Adding Map Features

You can extend the map functionality by adding more code to the initialization function:

```rust
// To add markers, layers, or other MapLibre features
let extended_code = format!(r#"
    map.on('load', function() {{
        // Add markers, layers, or custom controls here
        new maplibregl.Marker()
            .setLngLat([0, 0])
            .addTo(map);
    }});
"#);
```

## Troubleshooting

### Common Issues

1. **Map container not visible**
   - Ensure the container has proper dimensions (width and height)
   - Check browser console for JavaScript errors

2. **Map library not loading**
   - Verify network connections to CDN resources
   - Check if MapLibre JS/CSS URLs are correct

3. **Rust compilation errors**
   - Make sure you have all required features enabled in web-sys and js-sys

## License

This project is available under the MIT License.

## Acknowledgments

- [Dioxus](https://dioxuslabs.com) - Reactive UI framework for Rust
- [MapLibre GL JS](https://maplibre.org) - Free and open-source map rendering library
- [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) - Bridging Rust and JavaScript
