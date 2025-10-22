//! Maven command execution and management
//!
//! This module provides functionality for executing Maven commands,
//! detecting Spring Boot capabilities, and managing Maven profiles.

pub(crate) mod command;
pub(crate) mod detection;
pub(crate) mod process;
pub(crate) mod profiles;

// Re-export public APIs
pub use command::{check_maven_availability, execute_maven_command_async_with_options};
pub use detection::{
    LaunchStrategy, build_launch_command, decide_launch_strategy, detect_spring_boot_capabilities,
};
pub use process::{CommandUpdate, kill_process};
pub use profiles::{get_active_profiles, get_profile_xml, get_profiles};

// Tests are in a separate file at the crate level
// See ../maven_tests.rs
