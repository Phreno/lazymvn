//! # Maven Log Analyzer
//!
//! A library for analyzing Maven build logs - extract packages, exceptions, and build metrics.
//!
//! ## Features
//!
//! - **Package Detection**: Extract Java package names from log lines with high precision
//! - **Exception Detection**: Identify Java exceptions in logs
//! - **Stack Trace Parsing**: Parse and extract information from Java stack traces
//! - **Log Normalization**: Clean ANSI escape sequences and normalize log content
//! - **Statistical Analysis**: Extract unique packages for build statistics
//!
//! ## Example
//!
//! ```rust
//! use maven_log_analyzer::{analysis, parser};
//!
//! let log_line = "[INFO] com.example.service.UserService - Processing request";
//! let cleaned = parser::clean_log_line(log_line).unwrap();
//!
//! if let Some((_start, _end, package)) = analysis::extract_package_from_log_line(&cleaned, "[%p] %c - %m%n") {
//!     println!("Found package: {}", package);
//! }
//! ```
//!
//! ## Modules
//!
//! - `patterns`: Regex patterns for detecting Java packages, exceptions, and stack traces
//! - `analysis`: Functions for extracting and analyzing package names from logs
//! - `parser`: Utilities for cleaning and normalizing log content

pub mod analysis;
pub mod parser;
pub mod patterns;

// Re-export commonly used items
pub use analysis::{extract_package_from_log_line, extract_unique_packages, is_false_positive};
pub use parser::clean_log_line;
pub use patterns::{
    EXCEPTION_PATTERN, PACKAGE_PATTERN_GENERIC, PACKAGE_PATTERN_PERMISSIVE,
    PACKAGE_PATTERN_WITH_PREFIX, STACKTRACE_PATTERN,
};
