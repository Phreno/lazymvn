//! Utility modules
//!
//! This module provides various utility functions:
//! - `text`: Text processing (colorization, ANSI stripping, XML formatting)
//! - `log_patterns`: Regex patterns for log analysis
//! - `log_analysis`: Log extraction and analysis functions
//! - `logger`: Logging configuration
//! - `watcher`: File watching for live reload
//! - `loading`: Loading animations
//! - `git`: Git repository operations

pub mod git;
pub mod loading;
pub mod log_analysis;
pub mod log_patterns;
pub mod logger;
pub mod text;
pub mod version;
pub mod watcher;

// Re-export commonly used functions for convenience
pub use git::get_git_branch;
pub use log_analysis::{extract_package_from_log_line, extract_unique_packages, is_false_positive};
pub use text::{clean_log_line, colorize_log_line, colorize_log_line_with_format, colorize_xml_line};
