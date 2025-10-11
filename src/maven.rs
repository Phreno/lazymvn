use std::path::Path;
use std::process::{Command, Stdio};
use std::io::{BufReader, BufRead};

pub fn get_maven_command(project_root: &Path) -> String {
    if project_root.join("mvnw").exists() {
        "./mvnw".to_string()
    } else {
        "mvn".to_string()
    }
}

pub fn execute_maven_command(project_root: &Path, args: &[&str]) -> Result<Vec<String>, std::io::Error> {
    let maven_command = get_maven_command(project_root);
    let mut child = Command::new(maven_command)
        .args(args)
        .current_dir(project_root)
        .stdout(Stdio::piped())
        .spawn()?;

    let mut output = Vec::new();
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            output.push(line?);
        }
    }

    child.wait()?;
    Ok(output)
}

pub fn get_profiles(project_root: &Path) -> Result<Vec<String>, std::io::Error> {
    let output = execute_maven_command(project_root, &["help:all-profiles"])?;
    let profiles = output
        .iter()
        .filter_map(|line| {
            if line.contains("Profile Id:") {
                line.split("Profile Id:").last().map(|s| s.trim().to_string())
            } else {
                None
            }
        })
        .collect();
    Ok(profiles)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn get_maven_command_returns_mvnw_if_present() {
        let dir = tempdir().unwrap();
        let project_root = dir.path();

        // Test with mvnw present
        let mvnw_path = project_root.join("mvnw");
        File::create(&mvnw_path).unwrap();
        assert_eq!(get_maven_command(project_root), "./mvnw");

        // Test without mvnw present
        std::fs::remove_file(&mvnw_path).unwrap();
        assert_eq!(get_maven_command(project_root), "mvn");
    }

    #[test]
    fn execute_maven_command_captures_output() {
        let dir = tempdir().unwrap();
        let project_root = dir.path();

        // Create a mock mvnw script
        let mvnw_path = project_root.join("mvnw");
        let mut mvnw_file = File::create(&mvnw_path).unwrap();
        use std::io::Write;
        mvnw_file.write_all(b"#!/bin/sh\necho 'line 1'\necho 'line 2'").unwrap();
        // Make it executable
        use std::os::unix::fs::PermissionsExt;
        let mut perms = mvnw_file.metadata().unwrap().permissions();
        perms.set_mode(0o755);
        mvnw_file.set_permissions(perms).unwrap();
        drop(mvnw_file);

        let output = execute_maven_command(project_root, &["test"]).unwrap();
        assert_eq!(output, vec!["line 1", "line 2"]);
    }

    #[test]
    fn test_get_profiles() {
        let dir = tempdir().unwrap();
        let project_root = dir.path();

        // Create a mock mvnw script
        let mvnw_path = project_root.join("mvnw");
        let mut mvnw_file = File::create(&mvnw_path).unwrap();
        use std::io::Write;
        mvnw_file.write_all(b"#!/bin/sh\necho '[INFO]   Profile Id: profile-1'\necho '[INFO]   Profile Id: profile-2'").unwrap();
        use std::os::unix::fs::PermissionsExt;
        let mut perms = mvnw_file.metadata().unwrap().permissions();
        perms.set_mode(0o755);
        mvnw_file.set_permissions(perms).unwrap();
        drop(mvnw_file);

        let profiles = get_profiles(project_root).unwrap();
        assert_eq!(profiles, vec!["profile-1", "profile-2"]);
    }
}
