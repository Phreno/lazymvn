// Configuration management module
//
// This module handles all configuration-related functionality for LazyMVN,
// including config file loading, TOML parsing, logging configuration, and
// project preferences management.

mod io;
mod logging;
mod types;

// Re-export main types
pub use types::{
    Config, LaunchMode, ModulePreferences, ProjectPreferences, RecentProjects, WatchConfig,
};

pub use logging::{LoggingConfig, PackageLogLevel};

// Re-export main functions
pub use io::{
    create_project_config, get_project_config_path, has_project_config, load_config,
};
