//! Utility functions for LazyMVN
//!
//! This module re-exports all utility functions from the utils/ submodule
//! and provides Git-related functionality.

// Re-export all utility modules
pub use crate::utils::loading;
pub use crate::utils::logger;
pub use crate::utils::text;
pub use crate::utils::watcher;

// Re-export commonly used functions for convenience
pub use text::{clean_log_line, colorize_log_line, colorize_xml_line};

/// Get the current Git branch name for a project
/// Returns None if not a Git repository or if branch cannot be determined
pub fn get_git_branch(project_root: &std::path::Path) -> Option<String> {
/// Get the current Git branch name for a project
/// Returns None if not a Git repository or if branch cannot be determined
pub fn get_git_branch(project_root: &std::path::Path) -> Option<String> {
    use std::process::Command;

    let output = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .arg("branch")
        .arg("--show-current")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let branch = String::from_utf8(output.stdout).ok()?;
    let branch = branch.trim();

    if branch.is_empty() {
        None
    } else {
        Some(branch.to_string())
    }
}

    use std::process::Command;

    let output = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .arg("branch")
        .arg("--show-current")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let branch = String::from_utf8(output.stdout).ok()?;
    let branch = branch.trim();

    if branch.is_empty() {
        None
    } else {
        Some(branch.to_string())
    }
}
