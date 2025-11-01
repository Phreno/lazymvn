use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::core::config::logging::LoggingConfig;

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct Config {
    pub maven_settings: Option<String>,
    pub launch_mode: Option<LaunchMode>,
    pub notifications_enabled: Option<bool>,
    pub watch: Option<WatchConfig>,
    pub output: Option<OutputConfig>,
    pub logging: Option<LoggingConfig>,
    pub spring: Option<SpringConfig>,
    pub maven: Option<MavenConfig>,
}

/// Maven configuration for custom arguments
#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
pub struct MavenConfig {
    /// List of custom Maven flags/arguments
    #[serde(default)]
    pub custom_flags: Vec<CustomFlag>,
    
    /// List of custom Maven goals (e.g., plugins with full coordinates)
    #[serde(default)]
    pub custom_goals: Vec<CustomGoal>,
}

/// Custom Maven flag that can be toggled
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct CustomFlag {
    /// Display name for the flag (e.g., "Custom property")
    pub name: String,
    /// The actual Maven flag (e.g., "-Dmy.property=value")
    pub flag: String,
    /// Whether the flag is enabled by default (optional, default: false)
    #[serde(default)]
    pub enabled: bool,
}

/// Custom Maven goal for quick execution
/// These are typically plugin invocations that don't fit well as flags
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct CustomGoal {
    /// Display name for the goal (e.g., "Format code")
    pub name: String,
    /// The Maven goal to execute (e.g., "net.revelc.code.formatter:formatter-maven-plugin:2.23.0:format")
    pub goal: String,
}

/// Spring Boot configuration overrides
#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
pub struct SpringConfig {
    /// List of Spring Boot properties to override
    #[serde(default)]
    pub properties: Vec<SpringProperty>,

    /// Active profiles (alternative to -Dspring.profiles.active)
    #[serde(default)]
    pub active_profiles: Vec<String>,
}

/// Spring Boot property override
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct SpringProperty {
    /// Property name (e.g., "server.port")
    pub name: String,
    /// Property value (e.g., "8081")
    pub value: String,
}

/// Output buffer configuration
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct OutputConfig {
    /// Maximum number of lines to keep in output buffer (default: 10000)
    #[serde(default = "default_max_lines")]
    pub max_lines: usize,

    /// Maximum number of updates to process per poll cycle (default: 100)
    #[serde(default = "default_max_updates_per_poll")]
    pub max_updates_per_poll: usize,
}

fn default_max_lines() -> usize {
    10_000
}

fn default_max_updates_per_poll() -> usize {
    100
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            max_lines: default_max_lines(),
            max_updates_per_poll: default_max_updates_per_poll(),
        }
    }
}

/// File watching configuration
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct WatchConfig {
    /// Enable file watching (default: false)
    #[serde(default)]
    pub enabled: bool,

    /// Commands that should trigger auto-reload on file changes
    /// Default: ["test", "start"]
    #[serde(default = "default_watch_commands")]
    pub commands: Vec<String>,

    /// File patterns to watch (glob patterns)
    /// Default: ["src/**/*.java", "src/**/*.properties", "src/**/*.xml"]
    #[serde(default = "default_watch_patterns")]
    pub patterns: Vec<String>,

    /// Debounce delay in milliseconds (default: 500ms)
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,
}

fn default_watch_commands() -> Vec<String> {
    vec!["test".to_string(), "start".to_string()]
}

fn default_watch_patterns() -> Vec<String> {
    vec![
        "src/**/*.java".to_string(),
        "src/**/*.properties".to_string(),
        "src/**/*.xml".to_string(),
    ]
}

fn default_debounce_ms() -> u64 {
    500
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            commands: default_watch_commands(),
            patterns: default_watch_patterns(),
            debounce_ms: default_debounce_ms(),
        }
    }
}

/// Launch mode for running Maven applications
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LaunchMode {
    /// Auto-detect: use spring-boot:run if available, fallback to exec:java
    #[default]
    Auto,
    /// Always use spring-boot:run
    ForceRun,
    /// Always use exec:java
    ForceExec,
}
