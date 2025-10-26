//! Core functionality module
//!
//! This module contains the core application logic:
//! - `config`: Configuration file management (lazymvn.toml)
//! - `project`: Maven project discovery and module parsing

pub mod config;
pub mod project;

// Re-export main types for convenience (used by lib.rs public API)
#[allow(unused_imports)]
pub use config::{Config, LaunchMode, RecentProjects};
#[allow(unused_imports)]
pub use project::{get_project_modules, get_project_modules_for_path};
