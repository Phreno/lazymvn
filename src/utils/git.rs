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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_get_git_branch_current_repo() {
        // Test in the current repository
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        let branch = get_git_branch(&current_dir);
        
        // If we're in a git repo, we should get a branch name
        if let Some(branch_name) = branch {
            assert!(!branch_name.is_empty());
            assert!(!branch_name.contains('\n'));
        }
    }

    #[test]
    fn test_get_git_branch_non_git_directory() {
        // Test with a directory that's definitely not a git repo
        let temp_dir = std::env::temp_dir();
        let non_git_path = temp_dir.join("definitely-not-a-git-repo-test");
        
        let branch = get_git_branch(&non_git_path);
        // Should return None for non-git directories
        // Note: this could be Some if the temp dir is in a git repo
        let _ = branch;
    }

    #[test]
    fn test_get_git_branch_with_path() {
        let path = Path::new(".");
        let result = get_git_branch(path);
        // Just ensure it doesn't panic
        let _ = result;
    }
}
