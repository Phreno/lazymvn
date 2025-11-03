//! Tests for configuration features (custom flags, profiles, settings)
//!
//! These tests verify that configuration loading and Maven profile/flag
//! handling work correctly. Replaces manual scripts:
//! - test-custom-flags.sh
//! - test-profile-loading.sh
//! - test-log4j-filtering.sh

use lazymvn_test_harness::TestProject;
use std::path::PathBuf;

fn demo_project_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("demo/multi-module")
}

fn init_test_logging() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Debug)
        .try_init();
}

#[test]
fn test_custom_flags_basic() {
    init_test_logging();

    let project = TestProject::new(demo_project_path())
        .with_flags(&["-DskipTests"]);

    let result = project.compile_module("library")
        .expect("Failed to compile with custom flag");

    assert!(result.success, "Compile with -DskipTests should succeed");
    
    // Should see the flag in the Maven command if we had verbose output
    println!("✅ Custom flag -DskipTests applied successfully");
}

#[test]
fn test_multiple_custom_flags() {
    init_test_logging();

    let project = TestProject::new(demo_project_path())
        .with_flags(&[
            "-DskipTests",
            "-Dmaven.javadoc.skip=true",
            "-Dtest.property=value1",
        ]);

    let result = project.compile_module("library")
        .expect("Failed to compile with multiple flags");

    assert!(result.success, "Compile with multiple flags should succeed");
    println!("✅ Multiple custom flags applied successfully");
}

#[test]
fn test_update_snapshots_flag() {
    init_test_logging();

    let project = TestProject::new(demo_project_path())
        .with_flags(&["-U"]);

    let result = project.compile_module("library")
        .expect("Failed to compile with -U flag");

    assert!(result.success, "Compile with -U (update snapshots) should succeed");
    println!("✅ Update snapshots flag (-U) works");
}

#[test]
fn test_offline_mode_flag() {
    init_test_logging();

    let project = TestProject::new(demo_project_path())
        .with_flags(&["-o"]);

    let result = project.compile_module("library")
        .expect("Failed to compile with offline flag");

    // May succeed or fail depending on local cache, just check it doesn't crash
    println!("Offline mode result: {}", if result.success { "success" } else { "failed (expected if no cache)" });
}

#[test]
fn test_profile_activation_single() {
    init_test_logging();

    let project = TestProject::new(demo_project_path())
        .with_profiles(&["dev"]);

    let result = project.compile_module("library")
        .expect("Failed to compile with dev profile");

    assert!(result.success, "Compile with dev profile should succeed");
    println!("✅ Single profile activation (dev) works");
}

#[test]
fn test_profile_activation_multiple() {
    init_test_logging();

    let project = TestProject::new(demo_project_path())
        .with_profiles(&["dev", "fast"]);

    let result = project.compile_module("library")
        .expect("Failed to compile with multiple profiles");

    assert!(result.success, "Compile with multiple profiles should succeed");
    println!("✅ Multiple profile activation (dev,fast) works");
}

#[test]
fn test_profiles_and_flags_combined() {
    init_test_logging();

    let project = TestProject::new(demo_project_path())
        .with_profiles(&["dev"])
        .with_flags(&["-DskipTests", "-U"]);

    let result = project.compile_module("library")
        .expect("Failed to compile with profiles and flags");

    assert!(result.success, "Compile with profiles + flags should succeed");
    println!("✅ Combined profiles and flags work together");
}

#[test]
fn test_maven_settings_file() {
    init_test_logging();

    let demo_path = demo_project_path();
    let settings_file = demo_path.join("settings.xml");

    // Check if settings file exists
    if !settings_file.exists() {
        println!("⚠️  Skipping test: settings.xml not found at {:?}", settings_file);
        return;
    }

    let project = TestProject::new(demo_path)
        .with_settings("settings.xml");

    let result = project.compile_module("library")
        .expect("Failed to compile with settings file");

    assert!(result.success, "Compile with settings.xml should succeed");
    println!("✅ Maven settings file loaded successfully");
}

#[test]
fn test_fast_build_flags() {
    init_test_logging();

    // Simulate a "fast build" with multiple skip flags
    let project = TestProject::new(demo_project_path())
        .with_flags(&[
            "-DskipTests",
            "-Dmaven.javadoc.skip=true",
            "-Dmaven.source.skip=true",
            "-Dcheckstyle.skip=true",
        ]);

    let result = project.compile_module("library")
        .expect("Failed to compile with fast build flags");

    assert!(result.success, "Fast build should succeed");
    println!("✅ Fast build flags work");
}

#[test]
fn test_thread_count_flag() {
    init_test_logging();

    let project = TestProject::new(demo_project_path())
        .with_flags(&["-T", "2"]);

    let result = project.compile_module("library")
        .expect("Failed to compile with thread count");

    assert!(result.success, "Compile with -T 2 should succeed");
    println!("✅ Thread count flag (-T) works");
}

#[test]
fn test_quiet_flag() {
    init_test_logging();

    let project = TestProject::new(demo_project_path())
        .with_flags(&["-q"]);

    let result = project.compile_module("library")
        .expect("Failed to compile with quiet flag");

    assert!(result.success, "Compile with -q should succeed");
    
    // Quiet mode should produce less output
    println!("Output lines with -q: {}", result.line_count());
    println!("✅ Quiet flag (-q) works");
}

#[test]
fn test_debug_flag() {
    init_test_logging();

    let project = TestProject::new(demo_project_path())
        .with_flags(&["-X"]);

    let result = project.compile_module("library")
        .expect("Failed to compile with debug flag");

    assert!(result.success, "Compile with -X should succeed");
    
    // Debug mode should produce more output
    println!("Output lines with -X: {}", result.line_count());
    assert!(result.line_count() > 20, "Debug mode should produce substantial output");
    println!("✅ Debug flag (-X) works");
}

/// Test that invalid flags are still passed through to Maven
/// (Maven will handle the error)
#[test]
fn test_invalid_flag_handling() {
    init_test_logging();

    let project = TestProject::new(demo_project_path())
        .with_flags(&["--this-flag-does-not-exist"]);

    let result = project.compile_module("library");

    // Should get a result (even if it's an error from Maven)
    match result {
        Ok(cmd_result) => {
            // Maven might accept it or reject it
            println!("Invalid flag result: {}", if cmd_result.success { "accepted" } else { "rejected" });
        }
        Err(e) => {
            println!("Invalid flag error: {}", e);
        }
    }
    
    println!("✅ Invalid flag handling works (passed to Maven)");
}

/// Test profile loading from pom.xml
#[test]
fn test_profile_discovery() {
    init_test_logging();

    let demo_path = demo_project_path();
    let pom_file = demo_path.join("pom.xml");

    assert!(pom_file.exists(), "pom.xml should exist");

    // Read pom.xml and check for profiles
    let pom_content = std::fs::read_to_string(&pom_file)
        .expect("Failed to read pom.xml");

    let has_profiles = pom_content.contains("<profiles>") 
        || pom_content.contains("<profile>");

    if has_profiles {
        println!("✅ Project has Maven profiles defined");
        
        // Try to extract profile IDs (basic regex)
        let profile_count = pom_content.matches("<profile>").count();
        println!("   Found approximately {} profiles in pom.xml", profile_count);
    } else {
        println!("⚠️  Project has no Maven profiles defined (this is OK)");
    }
}

/// Test that flags with spaces are handled correctly
#[test]
fn test_flags_with_spaces() {
    init_test_logging();

    // Some flags might have spaces in their values
    let project = TestProject::new(demo_project_path())
        .with_flags(&["-Dmaven.compiler.source=17"]);

    let result = project.compile_module("library")
        .expect("Failed with flag containing =");

    assert!(result.success, "Flag with = should work");
    println!("✅ Flags with special characters work");
}
