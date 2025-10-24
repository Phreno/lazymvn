//! Maven command execution and management
//!
//! This module provides functionality for executing Maven commands,
//! detecting Spring Boot capabilities, and managing Maven profiles.

pub(crate) mod command;
pub(crate) mod detection;
pub(crate) mod process;
pub(crate) mod profiles;
pub(crate) mod log4j;
pub(crate) mod spring;

// Re-export public APIs
pub use command::{check_maven_availability, execute_maven_command_async_with_options};
pub use detection::{
    LaunchStrategy, build_launch_command, decide_launch_strategy, detect_spring_boot_capabilities,
};
pub use process::{CommandUpdate, kill_process};
pub use profiles::{get_active_profiles, get_profile_xml, get_profiles};
pub use log4j::generate_log4j_config;
pub use spring::generate_spring_properties;

// Tests are in a separate file at the crate level
// See ../maven_tests.rs
