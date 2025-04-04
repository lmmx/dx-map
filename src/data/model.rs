use serde::{Deserialize, Serialize};

// From the TfL topology data model, as recorded here:
// https://github.com/lmmx/tubeulator/blob/a8fc10becac3ea04cf16b91b0c24be944df692a5/src/tubeulator/topology/data_model.py

/// Represents a TfL station with its location and metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Station {
    /// Unique identifier for the station
    #[serde(rename = "StationUniqueId")]
    pub station_unique_id: String,
    /// Human-readable name of the station
    #[serde(rename = "StationName")]
    pub station_name: String,
    /// Fare zones the station belongs to (comma-separated)
    #[serde(rename = "FareZones")]
    pub fare_zones: String,
    /// Optional hub Naptan code for interchanges
    #[serde(rename = "HubNaptanCode")]
    #[serde(default)]
    pub hub_naptan_code: Option<String>,
    /// Whether the station has Wi-Fi
    #[serde(rename = "Wifi")]
    #[serde(default)]
    pub wifi: bool,
    /// Unique ID for outside of the station
    #[serde(rename = "OutsideStationUniqueId")]
    pub outside_station_unique_id: String,
    /// Latitude coordinate of the station
    #[serde(rename = "Lat")]
    pub lat: f64,
    /// Longitude coordinate of the station
    #[serde(rename = "Lon")]
    pub lon: f64,
    /// List of component station codes that make up this station
    #[serde(rename = "ComponentStations")]
    #[serde(default)]
    pub component_stations: Vec<String>,
}

/// Represents a platform at a TfL station
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Platform {
    /// Unique identifier for this platform
    #[serde(rename = "PlatformUniqueId")]
    pub platform_unique_id: String,
    /// Station this platform belongs to
    #[serde(rename = "StationUniqueId")]
    pub station_unique_id: String,
    /// Platform number (as string to handle complex numbering)
    #[serde(rename = "PlatformNumber")]
    #[serde(default)]
    pub platform_number: Option<String>,
    /// Direction of travel (Northbound, Southbound, etc.)
    #[serde(rename = "CardinalDirection")]
    #[serde(default)]
    pub cardinal_direction: Option<String>,
    /// Optional platform Naptan code
    #[serde(rename = "PlatformNaptanCode")]
    #[serde(default)]
    pub platform_naptan_code: Option<String>,
    /// Human-readable name for the platform
    #[serde(rename = "PlatformFriendlyName")]
    pub platform_friendly_name: String,
    /// Whether the platform is accessible to customers
    #[serde(rename = "IsCustomerFacing")]
    pub is_customer_facing: bool,
    /// Whether the platform has service interchange
    #[serde(rename = "HasServiceInterchange")]
    pub has_service_interchange: bool,
    /// Name of the station this platform is in
    #[serde(rename = "StationName")]
    pub station_name: String,
    /// Fare zones for this station
    #[serde(rename = "FareZones")]
    pub fare_zones: String,
    /// Hub Naptan code if applicable
    #[serde(rename = "HubNaptanCode")]
    #[serde(default)]
    pub hub_naptan_code: Option<String>,
    /// Whether the station has Wi-Fi
    #[serde(rename = "Wifi")]
    #[serde(default)]
    pub wifi: bool,
    /// Outside station unique ID
    #[serde(rename = "OutsideStationUniqueId")]
    pub outside_station_unique_id: String,
    /// Stop area Naptan code
    #[serde(rename = "StopAreaNaptanCode")]
    pub stop_area_naptan_code: String,
    /// Line this platform serves (e.g., "central", "district")
    #[serde(rename = "Line")]
    pub line: String,
    /// Direction this platform heads toward
    #[serde(rename = "DirectionTowards")]
    #[serde(default)]
    pub direction_towards: Option<String>,
    /// Platform service group name if applicable
    #[serde(rename = "PlatformServiceGroupName")]
    #[serde(default)]
    pub platform_service_group_name: Option<String>,
}

/// Response structure from the stations API
#[derive(Debug, Deserialize)]
pub struct StationsResponse {
    pub context: ResponseContext,
    pub success: bool,
    pub results: Vec<Station>,
}

/// Response structure from the platforms API
#[derive(Debug, Deserialize)]
pub struct PlatformsResponse {
    pub context: ResponseContext,
    pub success: bool,
    pub results: Vec<Platform>,
}

/// Context information included in API responses
#[derive(Debug, Deserialize)]
pub struct ResponseContext {
    pub request_time: String,
    pub response_time: String,
    pub response_latency: f64,
    pub query: String,
}
