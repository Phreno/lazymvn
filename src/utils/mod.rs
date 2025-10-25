//! Utility modules
//!
//! This module provides various utility functions:
//! - `text`: Text processing (colorization, ANSI stripping, XML formatting)
//! - `logger`: Logging configuration
//! - `watcher`: File watching for live reload
//! - `loading`: Loading animations
//! - `git`: Git repository operations

pub mod git;
pub mod loading;
pub mod logger;
pub mod text;
pub mod watcher;

// Re-export commonly used functions for convenience
pub use text::{clean_log_line, colorize_log_line, colorize_xml_line};
pub use git::get_git_branch;
