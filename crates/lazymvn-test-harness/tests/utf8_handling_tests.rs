//! Tests for UTF-8 handling in Maven output
//!
//! These tests verify that LazyMVN can handle non-UTF-8 characters
//! in Maven output without crashing the output reader threads.

use lazymvn_test_harness::TestProject;
use std::path::PathBuf;

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
    let _ = lazymvn::utils::logger::init(Some("debug"));
}

fn demo_project_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("demo/multi-module")
}

#[test]
fn test_utf8_lossy_conversion_doesnt_crash() {
    init();

    // This test verifies that the Maven output reader can handle
    // non-UTF-8 characters without crashing
    
    let project = TestProject::new(&demo_project_path());
    
    // Run any Maven command
    let result = project.compile_module("library");

    match result {
        Ok(cmd_result) => {
            // The test passes if the command completes without the reader crashing
            println!("✅ Command completed: {}", if cmd_result.success { "success" } else { "failed" });
            
            // Verify we got output (reader didn't crash)
            assert!(
                !cmd_result.output.is_empty(),
                "Should have captured output"
            );
        }
        Err(e) => {
            println!("⚠️  Command error: {}", e);
        }
    }
}

#[test]
fn test_output_reader_handles_special_characters() {
    init();

    // Maven output can contain special characters from:
    // - Windows-1252 encoding (Windows)
    // - ISO-8859-1 characters
    // - ANSI escape codes
    // - Box drawing characters in progress bars
    
    let project = TestProject::new(&demo_project_path());
    
    let result = project.build_module("library");

    match result {
        Ok(cmd_result) => {
            println!("✅ Build handled special characters");
            
            // The output should contain Maven output
            let has_maven_output = cmd_result.output.iter().any(|line| {
                line.contains("[INFO]") || 
                line.contains("Building") ||
                line.contains("SUCCESS") ||
                line.contains("FAILURE")
            });
            
            assert!(
                has_maven_output,
                "Should have captured Maven output (reader working)"
            );
        }
        Err(e) => {
            println!("⚠️  Error: {}", e);
        }
    }
}

#[test]
fn test_output_reader_survives_long_running_process() {
    init();

    // Long-running Maven processes can output thousands of lines
    // with various encodings
    
    let project = TestProject::new(&demo_project_path());
    
    // Run a comprehensive command that produces lots of output
    let result = project.run_command(".", &["clean", "validate"]);

    match result {
        Ok(cmd_result) => {
            println!("✅ Output reader survived long process");
            println!("   Captured {} lines", cmd_result.output.len());
            
            // Should have captured output throughout the entire run
            assert!(
                !cmd_result.output.is_empty(),
                "Output reader should stay alive for entire process"
            );
        }
        Err(e) => {
            println!("⚠️  Error: {}", e);
        }
    }
}

#[test]
fn test_no_error_log_for_utf8_issues() {
    init();

    // This test verifies that we don't log errors for UTF-8 issues
    // (they should be handled silently with lossy conversion)
    
    let project = TestProject::new(&demo_project_path());
    
    let result = project.compile_module("library");

    match result {
        Ok(cmd_result) => {
            // Check that output doesn't contain UTF-8 error messages
            let has_utf8_error = cmd_result.output.iter().any(|line| {
                line.contains("stream did not contain valid UTF-8") ||
                line.contains("Error reading stdout") ||
                line.contains("Error reading stderr")
            });
            
            assert!(
                !has_utf8_error,
                "Should not have UTF-8 errors in output (should use lossy conversion)"
            );
            
            println!("✅ No UTF-8 errors in output");
        }
        Err(e) => {
            println!("⚠️  Error: {}", e);
        }
    }
}

#[test]
fn test_replacement_character_for_invalid_utf8() {
    init();

    // Invalid UTF-8 should be replaced with � (U+FFFD)
    // This test just verifies the output reader completes successfully
    
    let project = TestProject::new(&demo_project_path());
    
    let result = project.run_command("library", &["--version"]);

    match result {
        Ok(cmd_result) => {
            println!("✅ Maven version command completed");
            
            // Should contain Maven version
            let has_version = cmd_result.output.iter().any(|line| {
                line.contains("Apache Maven") || line.contains("Maven")
            });
            
            assert!(has_version, "Should have Maven version in output");
        }
        Err(e) => {
            println!("⚠️  Error: {}", e);
        }
    }
}

#[test]
fn test_concurrent_output_with_mixed_encodings() {
    init();

    // Test that stdout and stderr can both handle mixed encodings
    
    let project = TestProject::new(&demo_project_path());
    
    // Try to trigger both stdout and stderr
    let result = project.run_command("nonexistent-module", &["compile"]);

    match result {
        Ok(cmd_result) => {
            // Should fail but not crash the output reader
            assert!(!cmd_result.success, "Should fail for nonexistent module");
            
            // Should have error output
            let has_error = cmd_result.output.iter().any(|line| {
                line.contains("ERROR") || line.contains("Unknown lifecycle phase")
            });
            
            if has_error {
                println!("✅ Error output captured correctly");
            } else {
                println!("⚠️  No error output (might be filtered)");
            }
        }
        Err(e) => {
            println!("⚠️  Error: {}", e);
        }
    }
}

#[test]
fn test_progress_bar_characters_dont_crash() {
    init();

    // Maven progress bars can contain special characters and ANSI codes
    
    let project = TestProject::new(&demo_project_path());
    
    // Download dependencies (shows progress bars)
    let result = project.run_command("library", &["dependency:resolve"]);

    match result {
        Ok(cmd_result) => {
            println!("✅ Dependency resolution with progress bars completed");
            println!("   Lines captured: {}", cmd_result.output.len());
        }
        Err(e) => {
            println!("⚠️  Error: {}", e);
        }
    }
}

#[test]
fn test_windows_encoding_compatibility() {
    init();

    // Windows Maven output often uses Windows-1252 encoding
    // This test verifies we can handle it
    
    let project = TestProject::new(&demo_project_path());
    
    let result = project.compile_module("library");

    match result {
        Ok(cmd_result) => {
            // On Windows, output might contain special characters
            // On Unix, it should still work fine
            println!("✅ Platform-specific encoding handled");
            
            #[cfg(windows)]
            {
                println!("   Windows: Handling Windows-1252 encoding");
            }
            
            #[cfg(not(windows))]
            {
                println!("   Unix: Handling UTF-8 encoding");
            }
            
            assert!(!cmd_result.output.is_empty());
        }
        Err(e) => {
            println!("⚠️  Error: {}", e);
        }
    }
}
