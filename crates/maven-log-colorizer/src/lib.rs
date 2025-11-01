//! # Maven Log Colorizer
//!
//! A library for colorizing Maven build logs with syntax highlighting for:
//! - Log levels (INFO, DEBUG, WARNING, ERROR)
//! - Package names (Java class paths)
//! - Exception names (NullPointerException, IOException, etc.)
//! - Stack traces (at com.example.Class.method)
//! - Command lines ($ mvn clean install)
//!
//! ## Features
//!
//! - **Format-aware**: Extracts and highlights package names based on log4j patterns
//! - **Exception highlighting**: Automatically detects and highlights Java exceptions
//! - **Stack trace coloring**: Beautiful syntax highlighting for stack traces
//! - **Ratatui integration**: Returns `Line<'static>` for direct use in ratatui UIs
//!
//! ## Example
//!
//! ```rust
//! use maven_log_colorizer::colorize_log_line;
//! use ratatui::text::Line;
//!
//! let log_line = "[INFO] Building project";
//! let colored: Line<'static> = colorize_log_line(log_line);
//! // Use `colored` directly in your ratatui Paragraph widget
//! ```
//!
//! ## With Log Format Pattern
//!
//! ```rust
//! use maven_log_colorizer::colorize_log_line_with_format;
//!
//! let log_line = "[INFO] com.example.MyClass - Processing data";
//! let log_format = "[%p] %c - %m%n";
//! let colored = colorize_log_line_with_format(log_line, Some(log_format));
//! // Package name "com.example.MyClass" will be highlighted in cyan
//! ```

mod colorizer;

pub use colorizer::{colorize_log_line, colorize_log_line_with_format};

// Re-export clean_log_line from maven-log-analyzer for convenience
pub use maven_log_analyzer::parser::clean_log_line;
