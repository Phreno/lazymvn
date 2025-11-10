//! Maven command building and execution
//!
//! This module provides functionality for:
//! - Building Maven command strings
//! - Detecting Maven wrapper (mvnw)
//! - Executing Maven commands synchronously
//! - Executing Maven commands asynchronously with streaming output
//! - Log4j configuration extraction and injection

mod builder;
#[cfg(test)]
mod builder_tests;
mod executor;
mod helpers;
mod log4j_config;

// Re-export public API
pub use builder::{
    check_maven_availability,
    get_maven_command,
};

pub use executor::{
    execute_maven_command,
    execute_maven_command_async_with_options,
    execute_maven_command_with_options,
};

// Helpers are used internally and by profiles module
pub(crate) use helpers::{
    parse_active_profile_from_line,
    parse_profile_id_from_line,
};
