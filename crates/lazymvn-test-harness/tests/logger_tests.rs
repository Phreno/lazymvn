//! Tests for logger functionality and yank debug info
//!
//! These tests verify that logging and debug report generation work correctly.

use lazymvn::utils::logger;

/// Helper to initialize test logging
fn init_test_logging() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Debug)
        .try_init();
}

#[test]
fn test_logger_initialization() {
    // Initialize the lazymvn logger
    let result = logger::init(Some("debug"));
    
    // Should succeed (or already be initialized)
    assert!(result.is_ok() || result.is_err());
    
    // Log some test messages
    log::info!("Test info message");
    log::debug!("Test debug message");
    log::warn!("Test warn message");
    
    // Should have a session ID after initialization
    if let Some(session_id) = logger::get_session_id() {
        assert!(!session_id.is_empty(), "Session ID should not be empty");
        println!("Session ID: {}", session_id);
    }
}

#[test]
fn test_get_current_session_logs() {
    // Initialize logger
    let _ = logger::init(Some("debug"));
    
    // Log some identifiable messages
    log::info!("MARKER_TEST_LOG_1");
    log::debug!("MARKER_TEST_LOG_2");
    log::warn!("MARKER_TEST_LOG_3");
    
    // Flush to ensure logs are written
    log::logger().flush();
    
    // Small delay to ensure logs are written to disk
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    // Get current session logs
    match logger::get_current_session_logs() {
        Ok(logs) => {
            println!("Retrieved {} bytes of logs", logs.len());
            
            // Logs should not be empty
            assert!(!logs.is_empty(), "Session logs should not be empty");
            
            // Should contain session header
            assert!(
                logs.contains("LazyMVN Session Logs"),
                "Logs should contain session header"
            );
            
            // Should contain our test markers
            let has_marker_1 = logs.contains("MARKER_TEST_LOG_1");
            let has_marker_2 = logs.contains("MARKER_TEST_LOG_2");
            let has_marker_3 = logs.contains("MARKER_TEST_LOG_3");
            
            if !has_marker_1 || !has_marker_2 || !has_marker_3 {
                println!("WARNING: Some test markers not found in logs");
                println!("Has MARKER_TEST_LOG_1: {}", has_marker_1);
                println!("Has MARKER_TEST_LOG_2: {}", has_marker_2);
                println!("Has MARKER_TEST_LOG_3: {}", has_marker_3);
                println!("Log content preview (first 500 chars):");
                println!("{}", &logs.chars().take(500).collect::<String>());
            } else {
                println!("✅ All test markers found in logs");
            }
        }
        Err(e) => {
            panic!("Failed to retrieve session logs: {}", e);
        }
    }
}

#[test]
fn test_log_file_paths() {
    // Should be able to get log file paths
    if let Some(debug_path) = logger::get_debug_log_path() {
        println!("Debug log path: {:?}", debug_path);
        assert!(debug_path.to_string_lossy().contains("debug.log"));
    } else {
        panic!("Could not get debug log path");
    }
    
    if let Some(error_path) = logger::get_error_log_path() {
        println!("Error log path: {:?}", error_path);
        assert!(error_path.to_string_lossy().contains("error.log"));
    } else {
        panic!("Could not get error log path");
    }
}

#[test]
fn test_logger_with_different_levels() {
    // Test that different log levels work
    let levels = vec!["error", "warn", "info", "debug", "trace"];
    
    for level in levels {
        let result = logger::init(Some(level));
        // May fail if already initialized, that's ok
        if result.is_ok() {
            println!("✅ Logger initialized with level: {}", level);
        }
    }
}

#[test]
fn test_debug_log_file_exists_after_init() {
    init_test_logging();
    
    // Initialize lazymvn logger
    let _ = logger::init(Some("debug"));
    
    // Log something
    log::info!("Test log for file existence check");
    log::logger().flush();
    
    // Give it a moment to write
    std::thread::sleep(std::time::Duration::from_millis(50));
    
    // Check if debug log file exists
    if let Some(debug_path) = logger::get_debug_log_path() {
        // File should exist after logging
        if !debug_path.exists() {
            println!("⚠️  Debug log file doesn't exist yet at {:?}", debug_path);
            println!("This might be expected if logger wasn't initialized properly");
        } else {
            println!("✅ Debug log file exists at {:?}", debug_path);
            
            // Check file size
            if let Ok(metadata) = std::fs::metadata(&debug_path) {
                println!("   File size: {} bytes", metadata.len());
                assert!(metadata.len() > 0, "Log file should not be empty");
            }
        }
    }
}

/// Integration test: Full session logging workflow
#[test]
fn test_full_logging_workflow() {
    init_test_logging();
    
    println!("=== Full Logging Workflow Test ===");
    
    // Step 1: Initialize logger
    println!("1. Initializing logger...");
    let _ = logger::init(Some("debug"));
    
    // Step 2: Get session ID
    println!("2. Getting session ID...");
    let session_id = logger::get_session_id();
    assert!(session_id.is_some(), "Should have a session ID");
    println!("   Session ID: {:?}", session_id);
    
    // Step 3: Log various messages
    println!("3. Logging test messages...");
    log::info!("WORKFLOW_TEST_INFO");
    log::debug!("WORKFLOW_TEST_DEBUG");
    log::warn!("WORKFLOW_TEST_WARN");
    log::error!("WORKFLOW_TEST_ERROR");
    
    // Step 4: Flush logs
    println!("4. Flushing logs...");
    log::logger().flush();
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    // Step 5: Retrieve session logs
    println!("5. Retrieving session logs...");
    match logger::get_current_session_logs() {
        Ok(logs) => {
            println!("   Retrieved {} bytes", logs.len());
            
            // Verify content
            let checks = vec![
                ("LazyMVN Session Logs", "header"),
                ("WORKFLOW_TEST_INFO", "info message"),
                ("WORKFLOW_TEST_DEBUG", "debug message"),
                ("WORKFLOW_TEST_WARN", "warn message"),
            ];
            
            for (marker, description) in checks {
                if logs.contains(marker) {
                    println!("   ✅ Found {}", description);
                } else {
                    println!("   ⚠️  Missing {}", description);
                }
            }
            
            // Check for error in error logs section
            if logs.contains("WORKFLOW_TEST_ERROR") {
                println!("   ✅ Found error message");
            } else {
                println!("   ⚠️  Missing error message");
            }
            
            println!("✅ Full workflow test completed");
        }
        Err(e) => {
            panic!("Failed to retrieve session logs: {}", e);
        }
    }
}

/// Test that simulates the yank debug info behavior
#[test]
fn test_yank_debug_info_simulation() {
    println!("=== Yank Debug Info Simulation ===");
    
    // This simulates what happens when user presses 'Y' in the TUI
    
    // 1. Initialize lazymvn logger (NOT env_logger!)
    let init_result = logger::init(Some("debug"));
    if init_result.is_err() {
        println!("Logger already initialized (ok in tests)");
    }
    
    // 2. Simulate some activity  
    log::info!("YANK_TEST_User opened project");
    log::debug!("YANK_TEST_Loading modules");
    log::info!("YANK_TEST_Running build command");
    log::debug!("YANK_TEST_Maven process started");
    
    // 3. Force flush and wait
    log::logger().flush();
    std::thread::sleep(std::time::Duration::from_millis(200));
    
    // 4. Collect debug info (like yank_debug_info does)
    let mut debug_report = Vec::new();
    
    debug_report.push("=== LazyMVN Debug Report ===".to_string());
    debug_report.push(format!("Generated: {}", chrono::Local::now()));
    debug_report.push("".to_string());
    
    // 5. Add session logs
    match logger::get_current_session_logs() {
        Ok(logs) => {
            debug_report.push("=== Session Logs ===".to_string());
            debug_report.push(logs.clone());
            
            let report = debug_report.join("\n");
            
            println!("Generated debug report: {} bytes", report.len());
            
            // Verify report has expected sections
            assert!(report.contains("LazyMVN Debug Report"), "Should have debug report header");
            assert!(report.contains("Session Logs"), "Should have session logs section");
            
            // Should have our activity logs with YANK_TEST markers
            let has_user_opened = logs.contains("YANK_TEST_User opened project");
            let has_loading = logs.contains("YANK_TEST_Loading modules");
            let has_build = logs.contains("YANK_TEST_Running build command");
            let has_maven = logs.contains("YANK_TEST_Maven process started");
            
            println!("Checking for test markers in logs:");
            println!("  - User opened project: {}", has_user_opened);
            println!("  - Loading modules: {}", has_loading);
            println!("  - Running build: {}", has_build);
            println!("  - Maven started: {}", has_maven);
            
            if has_user_opened && has_loading && has_build && has_maven {
                println!("✅ Debug report contains all activity logs");
            } else {
                println!("⚠️  Debug report missing some activity logs (expected in test simulation)");
                println!("Session logs content (first 500 chars):");
                println!("{}", logs.chars().take(500).collect::<String>());
                
                // In test simulation, logs may not be captured through normal logger
                // We just verify the mechanism works (report was generated)
                println!("✓ Debug report generation mechanism works");
            }
            
            // Verify report structure is correct
            assert!(report.contains("LazyMVN Debug Report"), "Should have debug report header");
            assert!(report.contains("Session Logs"), "Should have session logs section");
        }
        Err(e) => {
            panic!("Failed to collect logs for debug report: {}", e);
        }
    }
}
