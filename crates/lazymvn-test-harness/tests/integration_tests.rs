//! Integration tests using the test harness
//!
//! These tests verify that lazymvn's core operations work correctly
//! without requiring manual TUI interaction.

use lazymvn_test_harness::TestProject;
use std::path::PathBuf;

/// Helper to initialize logging for tests
fn init_test_logging() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Debug)
        .try_init();
}

/// Get the path to the demo project from workspace root
fn demo_project_path() -> PathBuf {
    // Tests run from crates/lazymvn-test-harness, so go up to workspace root
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("demo/multi-module")
}

#[test]
fn test_build_library_module() {
    init_test_logging();

    let project = TestProject::new(demo_project_path());
    let result = project.build_module("library")
        .expect("Failed to execute build command");

    // Should complete successfully
    assert!(result.success, "Build should succeed");
    assert_eq!(result.exit_code, Some(0), "Exit code should be 0");

    // Should have Maven output
    assert!(result.line_count() > 0, "Should have output lines");

    // Should contain expected build messages
    assert!(
        result.contains("BUILD SUCCESS"),
        "Output should contain BUILD SUCCESS"
    );
}

#[test]
fn test_build_app_module() {
    init_test_logging();

    let project = TestProject::new(demo_project_path());
    let result = project.build_module("app")
        .expect("Failed to execute build command");

    assert!(result.success, "App build should succeed");
    assert!(result.contains("BUILD SUCCESS"));
}

#[test]
fn test_compile_only() {
    init_test_logging();

    let project = TestProject::new(demo_project_path());
    let result = project.compile_module("library")
        .expect("Failed to execute compile command");

    assert!(result.success, "Compile should succeed");
    assert!(
        result.contains("Compiling") || result.contains("BUILD SUCCESS"),
        "Should show compilation output"
    );
}

#[test]
fn test_clean_module() {
    init_test_logging();

    let project = TestProject::new(demo_project_path());
    let result = project.clean_module("library")
        .expect("Failed to execute clean command");

    assert!(result.success, "Clean should succeed");
}

#[test]
fn test_package_module() {
    init_test_logging();

    let project = TestProject::new(demo_project_path());
    let result = project.package_module("library")
        .expect("Failed to execute package command");

    assert!(result.success, "Package should succeed");
    assert!(result.contains("BUILD SUCCESS"));
}

#[test]
fn test_build_with_profile() {
    init_test_logging();

    let project = TestProject::new(demo_project_path())
        .with_profiles(&["dev"]);

    let result = project.build_module("library")
        .expect("Failed to execute build with profile");

    assert!(result.success, "Build with profile should succeed");
}

#[test]
fn test_build_with_flags() {
    init_test_logging();

    let project = TestProject::new(demo_project_path())
        .with_flags(&["-U", "-DskipTests"]);

    let result = project.build_module("library")
        .expect("Failed to execute build with flags");

    assert!(result.success, "Build with flags should succeed");
}

#[test]
fn test_build_all_modules() {
    init_test_logging();

    let project = TestProject::new(demo_project_path());
    let result = project.build_all()
        .expect("Failed to execute build all");

    assert!(result.success, "Build all should succeed");
    assert!(result.contains("BUILD SUCCESS"));
}

#[test]
#[ignore] // This test requires Spring Boot app to be properly configured
fn test_start_spring_boot_app() {
    init_test_logging();

    let project = TestProject::new(demo_project_path());
    
    // First build to ensure dependencies are ready
    let build_result = project.build_module("app")
        .expect("Failed to build app");
    assert!(build_result.success, "Pre-build should succeed");

    // Then try to start (this will timeout/fail if app isn't configured for tests)
    let start_result = project.start_module("app")
        .expect("Failed to start app");

    // For now, just check that the command was attempted
    assert!(
        start_result.contains("spring-boot:run") || start_result.line_count() > 0,
        "Should attempt to run Spring Boot"
    );
}

/// Test that verifies Maven output is captured correctly
#[test]
fn test_maven_output_captured() {
    init_test_logging();

    let project = TestProject::new(demo_project_path());
    let result = project.compile_module("library")
        .expect("Failed to compile");

    // Critical: This test verifies the regression bug "logs perdus"
    // If this fails, Maven output is not being captured properly
    assert!(
        result.line_count() > 5,
        "Should capture substantial Maven output (got {} lines)",
        result.line_count()
    );

    // Should see Maven version or build info
    let has_maven_output = result.contains("Maven") 
        || result.contains("Building") 
        || result.contains("Compiling")
        || result.contains("[INFO]");

    assert!(
        has_maven_output,
        "Should capture Maven output messages. Got:\n{}",
        result.output.join("\n")
    );
}

/// Test for exit code detection (regression: build failures not detected)
#[test]
fn test_build_failure_detected() {
    init_test_logging();

    let project = TestProject::new(demo_project_path());
    
    // Try to build a non-existent module
    let result = project.run_command("nonexistent-module", &["compile"]);

    // Should fail
    match result {
        Ok(cmd_result) => {
            assert!(
                !cmd_result.success,
                "Building non-existent module should fail"
            );
            assert!(
                cmd_result.exit_code != Some(0),
                "Exit code should not be 0"
            );
        }
        Err(_) => {
            // It's OK if the command fails to execute
        }
    }
}

/// Test for logging configuration (regression: logs filtering)
#[test]
fn test_logging_levels_work() {
    init_test_logging();

    // This test would require logging config support in TestProject
    // For now, just verify basic build works
    let project = TestProject::new(demo_project_path());
    let result = project.build_module("library")
        .expect("Failed to build");

    assert!(result.success);
    
    // If logging config works, we should see [INFO] tags
    let has_log_tags = result.output.iter()
        .any(|line| line.contains("[INFO]") || line.contains("[DEBUG]") || line.contains("[WARNING]"));
    
    assert!(
        has_log_tags || result.contains("Building"),
        "Should capture Maven log output"
    );
}
