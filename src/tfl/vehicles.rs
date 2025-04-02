// src/tfl/vehicles.rs
use js_sys::{Array, Math, Object, Reflect};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use web_sys::console;

// Vehicle types
#[derive(Clone, Debug, PartialEq)]
pub enum VehicleType {
    Train,
    Bus,
}

// Vehicle structure for simulation
#[derive(Clone, Debug)]
pub struct Vehicle {
    pub id: usize,
    pub vehicle_type: VehicleType,
    pub route_name: String,
    pub current_position: RefCell<(f64, f64)>, // (longitude, latitude)
    pub start_position: (f64, f64),
    pub end_position: (f64, f64),
    pub progress: RefCell<f64>, // 0.0 to 1.0
    pub speed: f64,             // movement per tick (0.0 to 0.1)
    pub direction: RefCell<i8>, // 1 for forward, -1 for reverse
}

impl Vehicle {
    // Create a new vehicle
    pub fn new(
        id: usize,
        vehicle_type: VehicleType,
        route_name: String,
        start_position: (f64, f64),
        end_position: (f64, f64),
    ) -> Self {
        // Start at a random position along the route
        let progress = Math::random();

        // Calculate initial position
        let current_position = Self::interpolate_position(start_position, end_position, progress);

        // Random speed between 0.001 and 0.01
        let speed = 0.001 + (Math::random() * 0.009);

        // Random direction
        let direction = if Math::random() > 0.5 { 1 } else { -1 };

        Self {
            id,
            vehicle_type,
            route_name,
            current_position: RefCell::new(current_position),
            start_position,
            end_position,
            progress: RefCell::new(progress),
            speed,
            direction: RefCell::new(direction),
        }
    }

    // Update vehicle position
    pub fn update(&mut self) {
        // Update progress based on speed and direction
        let mut progress = self.progress.borrow_mut();
        *progress += self.speed * (*self.direction.borrow() as f64);

        // Check if we've reached the end or beginning
        if *progress >= 1.0 {
            *progress = 1.0;
            *self.direction.borrow_mut() = -1;
        } else if *progress <= 0.0 {
            *progress = 0.0;
            *self.direction.borrow_mut() = 1;
        }

        // Update current position
        *self.current_position.borrow_mut() =
            Self::interpolate_position(self.start_position, self.end_position, *progress);
    }

    // Helper to interpolate between two positions
    fn interpolate_position(start: (f64, f64), end: (f64, f64), progress: f64) -> (f64, f64) {
        let lng = start.0 + (end.0 - start.0) * progress;
        let lat = start.1 + (end.1 - start.1) * progress;
        (lng, lat)
    }
}

// Controller for the vehicle simulation
#[derive(Clone)]
pub struct SimulationController {
    vehicles: Vec<Vehicle>,
    is_running: RefCell<bool>,
    animation_frame_id: RefCell<Option<i32>>,
}

impl SimulationController {
    // Create a new simulation controller
    pub fn new(vehicles: Vec<Vehicle>) -> Self {
        Self {
            vehicles,
            is_running: RefCell::new(false),
            animation_frame_id: RefCell::new(None),
        }
    }

    // Start the simulation
    pub fn start(&self) -> Result<(), JsValue> {
        *self.is_running.borrow_mut() = true;
        self.request_animation_frame()
    }

    // Pause the simulation
    pub fn pause(&self) -> Result<(), JsValue> {
        *self.is_running.borrow_mut() = false;

        // Cancel animation frame if it exists
        if let Some(frame_id) = *self.animation_frame_id.borrow() {
            let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window found"))?;
            window.cancel_animation_frame(frame_id)?;
            *self.animation_frame_id.borrow_mut() = None;
        }

        Ok(())
    }

    /// Reset the simulation
    pub fn reset(&self) -> Result<(), JsValue> {
        // Pause first
        self.pause()?;

        // Reset all vehicles to random starting positions
        for vehicle in &self.vehicles {
            // We need to cast to mutable - a bit of a hack since we're using clones
            let new_progress = Math::random();
            *vehicle.progress.borrow_mut() = new_progress;

            *vehicle.current_position.borrow_mut() = Vehicle::interpolate_position(
                vehicle.start_position,
                vehicle.end_position,
                new_progress,
            );

            *vehicle.direction.borrow_mut() = if Math::random() > 0.5 { 1 } else { -1 };
        }

        // Restart the simulation
        self.start()
    }

    /// Request an animation frame
    fn request_animation_frame(&self) -> Result<(), JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window found"))?;

        // Create a clone of self for the closure
        let mut controller_clone = self.clone();

        // Create the animation frame callback
        let callback = Closure::wrap(Box::new(move || {
            // Only proceed if we're still running
            if *controller_clone.is_running.borrow() {
                // Update all vehicles
                for vehicle in &mut controller_clone.vehicles {
                    vehicle.update();
                }

                // Update the map with new vehicle positions
                if let Err(err) = controller_clone.update_vehicle_layer() {
                    console::error_1(&format!("Error updating vehicle layer: {:?}", err).into());
                }

                // Request next frame
                if let Err(err) = controller_clone.request_animation_frame() {
                    console::error_1(
                        &format!("Error requesting animation frame: {:?}", err).into(),
                    );
                }
            }
        }) as Box<dyn FnMut()>);

        // Request the animation frame
        let frame_id = window.request_animation_frame(callback.as_ref().unchecked_ref())?;
        *self.animation_frame_id.borrow_mut() = Some(frame_id);

        // Leak the closure to keep it alive
        callback.forget();

        Ok(())
    }

    /// Update the vehicle layer on the map
    fn update_vehicle_layer(&self) -> Result<(), JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window found"))?;

        // Check if mapInstance exists
        let map_instance = Reflect::get(&window, &JsValue::from_str("mapInstance"))?;
        if map_instance.is_undefined() {
            return Err(JsValue::from_str("Map instance not found"));
        }

        // Create GeoJSON features for vehicles
        let features = Array::new();

        for vehicle in &self.vehicles {
            let feature = Object::new();
            Reflect::set(
                &feature,
                &JsValue::from_str("type"),
                &JsValue::from_str("Feature"),
            )?;

            // Properties
            let properties = Object::new();
            Reflect::set(
                &properties,
                &JsValue::from_str("id"),
                &JsValue::from_f64(vehicle.id as f64),
            )?;

            let vehicle_type = match vehicle.vehicle_type {
                VehicleType::Train => "Train",
                VehicleType::Bus => "Bus",
            };

            Reflect::set(
                &properties,
                &JsValue::from_str("vehicleType"),
                &JsValue::from_str(vehicle_type),
            )?;

            Reflect::set(
                &properties,
                &JsValue::from_str("routeName"),
                &JsValue::from_str(&vehicle.route_name),
            )?;

            Reflect::set(&feature, &JsValue::from_str("properties"), &properties)?;

            // Geometry
            let geometry = Object::new();
            Reflect::set(
                &geometry,
                &JsValue::from_str("type"),
                &JsValue::from_str("Point"),
            )?;

            let coordinates = Array::new();
            let current_position = vehicle.current_position.borrow();
            coordinates.push(&JsValue::from_f64(current_position.0));
            coordinates.push(&JsValue::from_f64(current_position.1));

            Reflect::set(&geometry, &JsValue::from_str("coordinates"), &coordinates)?;
            Reflect::set(&feature, &JsValue::from_str("geometry"), &geometry)?;

            features.push(&feature);
        }

        // Create the GeoJSON object
        let geojson = Object::new();
        Reflect::set(
            &geojson,
            &JsValue::from_str("type"),
            &JsValue::from_str("FeatureCollection"),
        )?;
        Reflect::set(&geojson, &JsValue::from_str("features"), &features)?;

        let geojson_string = js_sys::JSON::stringify(&geojson)
            .unwrap_or_else(|_| JsValue::from_str("{}").into())
            .as_string()
            .unwrap_or_default();

        // Update the source if it exists
        let js_code = format!(
            r#"
            if (window.mapInstance && window.mapInstance.getSource('vehicles-source')) {{
                try {{
                    const data = {};
                    window.mapInstance.getSource('vehicles-source').setData(data);
                }} catch (e) {{
                    console.error("Error updating vehicles source:", e);
                }}
            }}
            "#,
            geojson_string
        );

        js_sys::eval(&js_code)?;

        Ok(())
    }
}
