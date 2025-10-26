//! LazyMVN - A Terminal UI for Maven Projects
//!
//! LazyMVN provides an interactive terminal interface for managing Maven multi-module projects.
//! It simplifies common Maven tasks like building, testing, and running specific modules
//! with support for profiles, build flags, and Spring Boot applications.
//!
//! # Architecture
//!
//! The codebase is organized into logical modules:
//!
//! ## Core Modules
//! - [`core`]: Configuration and project management
//!   - `config`: Configuration file handling (lazymvn.toml)
//!   - `project`: Maven project discovery and POM parsing
//!
//! ## Maven Integration
//! - [`maven`]: Maven command execution and detection
//!   - `command`: Maven command building and execution
//!   - `detection`: Spring Boot and exec:java detection
//!   - `profiles`: Maven profile loading and activation
//!   - `process`: Process management
//!   - `log4j`: Log4j configuration override
//!   - `spring`: Spring Boot properties override
//!
//! ## User Interface
//! - [`ui`]: Terminal UI components
//!   - `state`: Application state management
//!   - `keybindings`: Keyboard event handling
//!   - `panes`: UI rendering (layouts, module list, output, etc.)
//!   - `theme`: Color schemes and styles
//!   - `search`: Search functionality
//!
//! ## Features
//! - [`features`]: Optional enhancement features
//!   - `favorites`: Save and load favorite configurations
//!   - `history`: Command execution history
//!   - `starters`: Spring Boot starter management
//!
//! ## Utilities
//! - [`utils`]: Shared utility functions
//!   - `text`: Text processing (colorization, ANSI stripping)
//!   - `logger`: Logging system
//!   - `watcher`: File watching for live reload
//!   - `loading`: Loading screen animations
//!   - `git`: Git repository operations
//!
//! # Usage as a Library
//!
//! While LazyMVN is primarily a binary application, it can be used as a library
//! for building custom Maven tooling:
//!
//! ```rust,no_run
//! use lazymvn::core::project;
//!
//! // Load a Maven project
//! let (modules, project_root) = project::get_project_modules().unwrap();
//!
//! // Load configuration
//! let config = lazymvn::core::config::load_config(&project_root);
//! ```

// Public modules
pub mod core;
pub mod features;
pub mod maven;
pub mod ui;
pub mod utils;

// Re-export commonly used types for convenience
pub use core::{Config, LaunchMode};

#[cfg(test)]
pub mod test_utils {
    use std::sync::{Mutex, OnceLock};

    pub fn fs_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }
}
