//! Maven command execution and management
//!
//! This module provides functionality for executing Maven commands,
//! detecting Spring Boot capabilities, and managing Maven profiles.

pub(crate) mod process;
pub(crate) mod detection;
pub(crate) mod command;
pub(crate) mod profiles;

// Re-export public APIs
pub use process::{CommandUpdate, kill_process};
pub use detection::{
    LaunchStrategy,
    detect_spring_boot_capabilities, decide_launch_strategy,
    build_launch_command,
};
pub use command::{
    check_maven_availability,
    execute_maven_command_async_with_options,
};
pub use profiles::{
    get_profiles, get_active_profiles, get_profile_xml,
};

// Tests are in a separate file at the crate level
// See ../maven_tests.rs

