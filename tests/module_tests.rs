// Module selection and -f flag tests
use lazymvn::maven::{execute_maven_command, execute_maven_command_with_options};
use lazymvn::utils;
use tempfile::tempdir;

mod common;

#[test]
#[cfg(unix)] // Shell script execution not supported on Windows
fn execute_maven_command_scopes_to_module() {
    let _guard = common::test_lock().lock().unwrap();
    let dir = tempdir().unwrap();
    let project_root = dir.path();

    let mvnw_path = project_root.join("mvnw");
    common::write_script(&mvnw_path, "#!/bin/sh\necho $@\n");

    let output: Vec<String> =
        execute_maven_command(project_root, Some("module-a"), &["test"], &[], None, &[])
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
    assert_eq!(maven_output, vec!["-pl module-a test"]);
}

#[test]
#[cfg(unix)] // Shell script execution not supported on Windows
fn execute_maven_command_without_pl_for_root_module() {
    let _guard = common::test_lock().lock().unwrap();
    let dir = tempdir().unwrap();
    let project_root = dir.path();

    let mvnw_path = project_root.join("mvnw");
    common::write_script(&mvnw_path, "#!/bin/sh\necho $@\n");

    let output: Vec<String> =
        execute_maven_command(project_root, Some("."), &["test"], &[], None, &[])
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
    assert_eq!(maven_output, vec!["test"]);
}

#[test]
#[cfg(unix)]
fn test_exec_java_with_file_flag_adds_also_make() {
    let _guard = common::test_lock().lock().unwrap();
    let dir = tempdir().unwrap();
    let project_root = dir.path();

    let mvnw_path = project_root.join("mvnw");
    common::write_script(&mvnw_path, "#!/bin/sh\necho $@\n");

    let output: Vec<String> = execute_maven_command_with_options(
        project_root,
        Some("my-module"),
        &["exec:java"],
        &[],
        None,
        &[],
        true, // use_file_flag = true
        None, // no logging config
    )
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

    // Should contain -f flag, --also-make, and exec:java
    let command_output = maven_output.join(" ");
    assert!(command_output.contains("-f"));
    assert!(command_output.contains("--also-make"));
    assert!(command_output.contains("exec:java"));
}

#[test]
#[cfg(unix)]
fn test_exec_java_with_file_flag_preserves_existing_also_make() {
    let _guard = common::test_lock().lock().unwrap();
    let dir = tempdir().unwrap();
    let project_root = dir.path();

    let mvnw_path = project_root.join("mvnw");
    common::write_script(&mvnw_path, "#!/bin/sh\necho $@\n");

    let flags = vec!["--also-make-dependents".to_string()];
    let output: Vec<String> = execute_maven_command_with_options(
        project_root,
        Some("my-module"),
        &["exec:java"],
        &[],
        None,
        &flags,
        true, // use_file_flag = true
        None, // no logging config
    )
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

    let command_output = maven_output.join(" ");
    // Should contain existing flag but not auto-add --also-make
    assert!(command_output.contains("--also-make-dependents"));
    // Should have only one occurrence of "also-make" (from the existing flag)
    assert_eq!(command_output.matches("also-make").count(), 1);
}
