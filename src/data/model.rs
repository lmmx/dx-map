use serde::{Deserialize, Serialize};

/// Represents a TfL station with its location and metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Station {
    /// Unique identifier for the station
    pub StationUniqueId: String,
    /// Human-readable name of the station
    pub StationName: String,
    /// Fare zones the station belongs to (comma-separated)
    pub FareZones: String,
    /// Optional hub Naptan code for interchanges
    #[serde(default)]
    pub HubNaptanCode: Option<String>,
    /// Whether the station has Wi-Fi
    #[serde(default)]
    pub Wifi: bool,
    /// Unique ID for outside of the station
    pub OutsideStationUniqueId: String,
    /// Latitude coordinate of the station
    pub Lat: f64,
    /// Longitude coordinate of the station
    pub Lon: f64,
    /// List of component station codes that make up this station
    #[serde(default)]
    pub ComponentStations: Vec<String>,
}

/// Represents a platform at a TfL station
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Platform {
    /// Unique identifier for this platform
    pub PlatformUniqueId: String,
    /// Station this platform belongs to
    pub StationUniqueId: String,
    /// Platform number (as string to handle complex numbering)
    pub PlatformNumber: String,
    /// Direction of travel (Northbound, Southbound, etc.)
    pub CardinalDirection: String,
    /// Optional platform Naptan code
    #[serde(default)]
    pub PlatformNaptanCode: Option<String>,
    /// Human-readable name for the platform
    pub PlatformFriendlyName: String,
    /// Whether the platform is accessible to customers
    pub IsCustomerFacing: bool,
    /// Whether the platform has service interchange
    pub HasServiceInterchange: bool,
    /// Name of the station this platform is in
    pub StationName: String,
    /// Fare zones for this station
    pub FareZones: String,
    /// Hub Naptan code if applicable
    #[serde(default)]
    pub HubNaptanCode: Option<String>,
    /// Whether the station has Wi-Fi
    #[serde(default)]
    pub Wifi: bool,
    /// Outside station unique ID
    pub OutsideStationUniqueId: String,
    /// Stop area Naptan code
    pub StopAreaNaptanCode: String,
    /// Line this platform serves (e.g., "central", "district")
    pub Line: String,
    /// Direction this platform heads toward
    #[serde(default)]
    pub DirectionTowards: Option<String>,
    /// Platform service group name if applicable
    #[serde(default)]
    pub PlatformServiceGroupName: Option<String>,
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