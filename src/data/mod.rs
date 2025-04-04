pub mod model;
pub mod loader;
pub mod map_helpers;

// Re-export commonly used items
pub use model::{Station, Platform};
pub use loader::{load_stations, load_platforms, filter_valid_stations, group_platforms_by_station};
pub use map_helpers::{stations_to_geojson, line_to_geojson, get_line_color, generate_all_line_data};

use crate::utils::log::{self, LogCategory};
use std::collections::HashMap;

/// A consolidated data repository for TfL data
#[derive(Clone)]
pub struct TflDataRepository {
    /// All stations with valid coordinates
    pub stations: Vec<model::Station>,
    /// All platforms grouped by station ID
    pub platforms_by_station: HashMap<String, Vec<model::Platform>>,
    /// Stations by their unique ID for quick lookup
    pub station_by_id: HashMap<String, model::Station>,
    /// Indicates if the repository has been loaded
    pub is_loaded: bool,
}

impl Default for TflDataRepository {
    fn default() -> Self {
        Self {
            stations: Vec::new(),
            platforms_by_station: HashMap::new(),
            station_by_id: HashMap::new(),
            is_loaded: false,
        }
    }
}

impl TflDataRepository {
    /// Initialize the data repository by loading all data
    pub async fn initialize() -> Result<Self, String> {
        log::info_with_category(LogCategory::App, "Initializing TFL data repository");
        
        // Load and process stations
        let stations = loader::load_stations().await?;
        let valid_stations = loader::filter_valid_stations(stations);
        
        // Create lookup map for stations
        let station_by_id = valid_stations.iter()
            .map(|s| (s.StationUniqueId.clone(), s.clone()))
            .collect();
        
        // Load and process platforms
        let platforms = loader::load_platforms().await?;
        let platforms_by_station = loader::group_platforms_by_station(platforms);
        
        log::info_with_category(
            LogCategory::App, 
            &format!("TFL data repository initialized with {} stations", valid_stations.len())
        );
        
        Ok(Self {
            stations: valid_stations,
            platforms_by_station,
            station_by_id,
            is_loaded: true,
        })
    }
    
    /// Get a station by its unique ID
    pub fn get_station(&self, station_id: &str) -> Option<&model::Station> {
        self.station_by_id.get(station_id)
    }
    
    /// Get platforms for a specific station
    pub fn get_platforms_for_station(&self, station_id: &str) -> Vec<&model::Platform> {
        match self.platforms_by_station.get(station_id) {
            Some(platforms) => platforms.iter().collect(),
            None => Vec::new(),
        }
    }
    
    /// Get all stations for a specific line
    pub fn get_stations_for_line(&self, line_name: &str) -> Vec<&model::Station> {
        let mut result = Vec::new();
        
        // Check each station's platforms to see if any serve this line
        for (station_id, platforms) in &self.platforms_by_station {
            let serves_line = platforms.iter().any(|p| p.Line == line_name);
            
            if serves_line {
                if let Some(station) = self.station_by_id.get(station_id) {
                    result.push(station);
                }
            }
        }
        
        result
    }
}