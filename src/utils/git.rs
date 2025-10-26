//! Git-related utilities

use std::process::Command;

/// Get the current Git branch name for a project
/// Returns None if not a Git repository or if branch cannot be determined
pub fn get_git_branch(project_root: &std::path::Path) -> Option<String> {
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
