# Development

Your new bare-bones project includes minimal organization with a single `main.rs` file and a few assets.

### Serving Your App

Run the following command in the root of your project to start developing with the default platform:

```bash
dx serve
```

To run for a different platform, use the `--platform platform` flag. E.g.
```bash
dx serve --platform desktop
```


## Requirements

All the requirements for building Dioxus projects

```bash
sudo apt install libgdk3.0-cil libatk1.0-dev libcairo2-dev libpango1.0-dev libgdk-pixbuf2.0-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev libwebkit2gtk-4.1-dev libxdo-dev -y
```

plus protoc for the geozero dependency of maplibre-rs crate

```bash
sudo apt-get install protobuf-compiler -y
```
