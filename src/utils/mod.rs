pub mod log;

// Re-export commonly used logging functions to make them easier to import
pub use log::{LogCategory, LogLevel, get_log_level, set_category_enabled, set_log_level};
