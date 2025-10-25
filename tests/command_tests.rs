// Maven command execution tests
use lazymvn::maven::{execute_maven_command, get_maven_command, get_profiles};
use lazymvn::utils;
use std::fs;
use tempfile::tempdir;

mod common;

#[test]
fn get_maven_command_returns_mvnw_if_present() {
    let dir = tempdir().unwrap();
    let project_root = dir.path();

    // Test with mvnw present
    #[cfg(unix)]
    {
        let mvnw_path = project_root.join("mvnw");
        fs::File::create(&mvnw_path).unwrap();
        assert_eq!(get_maven_command(project_root), "./mvnw");
        std::fs::remove_file(&mvnw_path).unwrap();
    }

    #[cfg(windows)]
    {
        let mvnw_path = project_root.join("mvnw.bat");
        fs::File::create(&mvnw_path).unwrap();
        assert_eq!(get_maven_command(project_root), "mvnw.bat");
        std::fs::remove_file(&mvnw_path).unwrap();
    }

    // Test without mvnw present
    #[cfg(windows)]
    {
        assert_eq!(get_maven_command(project_root), "mvn.cmd");
    }
    #[cfg(not(windows))]
    {
        assert_eq!(get_maven_command(project_root), "mvn");
    }
}

#[test]
#[cfg(unix)] // Shell script execution not supported on Windows
fn execute_maven_command_captures_output() {
    let _guard = common::test_lock().lock().unwrap();
    let dir = tempdir().unwrap();
    let project_root = dir.path();

    // Create a mock mvnw script
    let mvnw_path = project_root.join("mvnw");
    common::write_script(&mvnw_path, "#!/bin/sh\necho 'line 1'\necho 'line 2'\n");

    let output: Vec<String> = execute_maven_command(project_root, None, &["test"], &[], None, &[])
        .unwrap()
        .iter()
        .filter_map(|line| utils::clean_log_line(line))
        .collect();

    // Output now includes command line at the start
    // Skip the command line to check actual Maven output
    let maven_output: Vec<String> = output
        .iter()
        .skip_while(|line| line.starts_with("$ "))
        .cloned()
        .collect();
    assert_eq!(maven_output, vec!["line 1", "line 2"]);
}

#[test]
#[cfg(unix)] // Shell script execution not supported on Windows
fn execute_maven_command_captures_stderr() {
    let _guard = common::test_lock().lock().unwrap();
    let dir = tempdir().unwrap();
    let project_root = dir.path();

    let mvnw_path = project_root.join("mvnw");
    common::write_script(
        &mvnw_path,
        "#!/bin/sh\necho 'line 1'\n>&2 echo 'warn message'\n",
    );

    let output: Vec<String> = execute_maven_command(project_root, None, &["test"], &[], None, &[])
        .unwrap()
        .iter()
        .filter_map(|line| utils::clean_log_line(line))
        .collect();

    // Skip command line header
    let maven_output: Vec<String> = output
        .iter()
        .skip_while(|line| line.starts_with("$ "))
        .cloned()
        .collect();
    assert!(
        maven_output.contains(&"line 1".to_string()),
        "stdout line should be present"
    );
    assert!(
        maven_output.contains(&"[ERR] warn message".to_string()),
        "stderr line should be tagged"
    );
}

#[test]
#[cfg(unix)] // Shell script execution not supported on Windows
fn execute_maven_command_with_profiles() {
    let _guard = common::test_lock().lock().unwrap();
    let dir = tempdir().unwrap();
    let project_root = dir.path();

    // Create a mock mvnw script
    let mvnw_path = project_root.join("mvnw");
    common::write_script(&mvnw_path, "#!/bin/sh\necho $@\n");

    let profiles = vec!["p1".to_string(), "p2".to_string()];
    let output: Vec<String> =
        execute_maven_command(project_root, None, &["test"], &profiles, None, &[])
            .unwrap()
            .iter()
            .filter_map(|line| utils::clean_log_line(line))
            .collect();

    // Skip command line header and check actual Maven output
    let maven_output: Vec<String> = output
        .iter()
        .skip_while(|line| line.starts_with("$ "))
        .cloned()
        .collect();
    assert_eq!(maven_output, vec!["-P p1,p2 test"]);
}

#[test]
#[cfg(unix)] // Shell script execution not supported on Windows
fn test_get_profiles() {
    let _guard = common::test_lock().lock().unwrap();
    let dir = tempdir().unwrap();
    let project_root = dir.path();

    // Create a mock mvnw script that simulates Maven's help:all-profiles output
    let mvnw_path = project_root.join("mvnw");
    common::write_script(
        &mvnw_path,
        "#!/bin/sh\necho '  Profile Id: profile-1 (Active: false, Source: pom)'\necho '  Profile Id: profile-2 (Active: true, Source: pom)'\n",
    );

    let profiles = get_profiles(project_root).unwrap();
    assert_eq!(profiles, vec!["profile-1", "profile-2"]);
}

#[test]
#[cfg(unix)] // Shell script execution not supported on Windows
fn test_get_profiles_deduplicates_and_sorts() {
    let _guard = common::test_lock().lock().unwrap();
    let dir = tempdir().unwrap();
    let project_root = dir.path();

    // Create a mock mvnw script that simulates Maven's help:all-profiles output
    // with duplicates (as would happen in multi-module projects without -N)
    let mvnw_path = project_root.join("mvnw");
    common::write_script(
        &mvnw_path,
        "#!/bin/sh\necho '  Profile Id: profile-2 (Active: false, Source: pom)'\necho '  Profile Id: profile-1 (Active: false, Source: pom)'\necho '  Profile Id: profile-2 (Active: false, Source: pom)'\necho '  Profile Id: child-profile (Active: false, Source: pom)'\n",
    );

    let profiles = get_profiles(project_root).unwrap();
    // Should be deduplicated and sorted
    assert_eq!(profiles, vec!["child-profile", "profile-1", "profile-2"]);
}
