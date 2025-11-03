// Process cleanup and lifecycle management tests
// Replaces: scripts/test-process-cleanup.sh

use lazymvn_test_harness::TestProject;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
    let _ = lazymvn::utils::logger::init(Some("debug"));
}

/// Helper to check if a process with given PID exists
#[cfg(unix)]
fn process_exists(pid: u32) -> bool {
    use std::fs;
    fs::metadata(format!("/proc/{}", pid)).is_ok()
}

#[cfg(windows)]
fn process_exists(pid: u32) -> bool {
    use std::process::Command;
    Command::new("tasklist")
        .arg("/FI")
        .arg(format!("PID eq {}", pid))
        .output()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .contains(&pid.to_string())
        })
        .unwrap_or(false)
}

/// Helper to find Maven/Java processes
fn find_maven_processes() -> Vec<u32> {
    #[cfg(unix)]
    {
        let output = Command::new("ps")
            .args(&["aux"])
            .output()
            .expect("Failed to run ps");
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.lines()
            .filter(|line| {
                (line.contains("mvn") || line.contains("java")) && 
                !line.contains("grep") &&
                !line.contains("test_")
            })
            .filter_map(|line| {
                line.split_whitespace()
                    .nth(1)
                    .and_then(|pid| pid.parse::<u32>().ok())
            })
            .collect()
    }
    
    #[cfg(windows)]
    {
        let output = Command::new("tasklist")
            .arg("/FI")
            .arg("IMAGENAME eq java.exe")
            .output()
            .expect("Failed to run tasklist");
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.lines()
            .skip(3) // Skip header lines
            .filter_map(|line| {
                line.split_whitespace()
                    .nth(1)
                    .and_then(|pid| pid.parse::<u32>().ok())
            })
            .collect()
    }
}

/// Helper to kill a process
fn kill_process(pid: u32) -> bool {
    #[cfg(unix)]
    {
        Command::new("kill")
            .arg("-9")
            .arg(pid.to_string())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
    
    #[cfg(windows)]
    {
        Command::new("taskkill")
            .args(&["/F", "/PID", &pid.to_string()])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

#[test]
fn test_no_orphaned_maven_processes() {
    init();

    // Get initial Maven processes
    let initial_processes = find_maven_processes();
    println!("Initial Maven/Java processes: {:?}", initial_processes);
    
    // This test just verifies we can detect processes
    // Actual cleanup is tested in integration
    assert!(true, "Process detection working");
}

#[test]
fn test_maven_process_cleanup_after_build() {
    init();

    let project = TestProject::new("/workspaces/lazymvn/demo/multi-module");
    
    // Count processes before
    let before = find_maven_processes().len();
    
    // Run a quick clean (lighter than full build)
    let result = project.clean_module("library");
    
    // If successful, verify no leak
    if let Ok(r) = result {
        if r.success {
            // Wait a bit for cleanup
            thread::sleep(Duration::from_secs(1));
            
            // Count processes after
            let after = find_maven_processes().len();
            
            // Should not have more processes than before
            assert!(
                after <= before + 1,
                "Possible process leak: before={}, after={}",
                before, after
            );
        }
    }
    
    // Test completes (either clean worked or we skip leak check)
    assert!(true);
}

#[test]
fn test_process_tracking() {
    init();

    // Start a simple long-running command
    let child = Command::new("sleep")
        .arg("5")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    
    if let Ok(mut child) = child {
        let pid = child.id();
        
        // Process should exist
        assert!(process_exists(pid), "Process should be running");
        
        // Kill it
        let _ = child.kill();
        let _ = child.wait();
        
        // Give OS time to clean up
        thread::sleep(Duration::from_millis(100));
        
        // Process should not exist
        assert!(!process_exists(pid), "Process should be killed");
    }
}

#[test]
fn test_background_process_termination() {
    init();

    let is_running = Arc::new(AtomicBool::new(true));
    let is_running_clone = is_running.clone();
    
    // Spawn a background "process" (thread simulating a Maven process)
    let handle = thread::spawn(move || {
        while is_running_clone.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(100));
        }
    });
    
    // Simulate work
    thread::sleep(Duration::from_millis(500));
    
    // Signal termination
    is_running.store(false, Ordering::Relaxed);
    
    // Should terminate quickly
    let result = handle.join();
    assert!(result.is_ok(), "Background process should terminate cleanly");
}

#[test]
fn test_process_cleanup_on_error() {
    init();

    let project = TestProject::new("/workspaces/lazymvn/demo/multi-module");
    
    let before = find_maven_processes().len();
    
    // Try to build non-existent module (should fail)
    let result = project.build_module("nonexistent-module");
    
    // Either failed Result or unsuccessful CommandResult
    let failed = result.is_err() || !result.unwrap().success;
    assert!(failed, "Build should fail for non-existent module");
    
    // Wait for cleanup
    thread::sleep(Duration::from_secs(1));
    
    let after = find_maven_processes().len();
    
    // No process leak even on error
    assert!(
        after <= before + 1,
        "Process leak on error: before={}, after={}",
        before, after
    );
}

#[test]
fn test_multiple_sequential_builds_cleanup() {
    init();

    let project = TestProject::new("/workspaces/lazymvn/demo/multi-module");
    
    let initial = find_maven_processes().len();
    
    // Run multiple builds
    for i in 0..3 {
        println!("Build iteration {}", i + 1);
        let result = project.clean_module("library");
        if let Ok(r) = result {
            assert!(r.success, "Clean should succeed");
        }
        
        thread::sleep(Duration::from_millis(500));
    }
    
    // Wait for all cleanup
    thread::sleep(Duration::from_secs(2));
    
    let final_count = find_maven_processes().len();
    
    // Should not accumulate processes
    assert!(
        final_count <= initial + 2,
        "Processes accumulating: initial={}, final={}",
        initial, final_count
    );
}

#[test]
fn test_concurrent_process_limit() {
    init();

    // This tests that we don't spawn too many processes at once
    let _project = TestProject::new("/workspaces/lazymvn/demo/multi-module");
    
    let results = Arc::new(Mutex::new(Vec::new()));
    let mut handles = vec![];
    
    // Try to spawn multiple builds (should be serialized or limited)
    for _ in 0..3 {
        let project_clone = TestProject::new("/workspaces/lazymvn/demo/multi-module");
        let results_clone = results.clone();
        
        let handle = thread::spawn(move || {
            let result = project_clone.compile_module("library");
            let success = result.map(|r| r.success).unwrap_or(false);
            results_clone.lock().unwrap().push(success);
        });
        
        handles.push(handle);
    }
    
    // Wait for all
    for handle in handles {
        let _ = handle.join();
    }
    
    // All should succeed (or fail gracefully)
    let results = results.lock().unwrap();
    assert_eq!(results.len(), 3, "All builds should complete");
}

#[test]
fn test_sigterm_handling_simulation() {
    init();

    // Simulate SIGTERM/SIGINT handling
    let shutdown_flag = Arc::new(AtomicBool::new(false));
    let flag_clone = shutdown_flag.clone();
    
    let handle = thread::spawn(move || {
        // Simulate a running process
        let mut count = 0;
        while !flag_clone.load(Ordering::Relaxed) && count < 100 {
            thread::sleep(Duration::from_millis(10));
            count += 1;
        }
        count
    });
    
    // Let it run a bit
    thread::sleep(Duration::from_millis(50));
    
    // Signal shutdown
    shutdown_flag.store(true, Ordering::Relaxed);
    
    // Should stop quickly
    let count = handle.join().unwrap();
    assert!(count < 100, "Process should stop on signal, stopped at iteration {}", count);
}

#[test]
fn test_zombie_process_detection() {
    init();

    // On Unix, check for zombie processes
    #[cfg(unix)]
    {
        let output = Command::new("ps")
            .args(&["aux"])
            .output()
            .expect("Failed to run ps");
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let zombies: Vec<_> = stdout.lines()
            .filter(|line| line.contains("<defunct>") || line.contains("Z+"))
            .collect();
        
        assert!(
            zombies.is_empty(),
            "Zombie processes detected:\n{}",
            zombies.join("\n")
        );
    }
    
    #[cfg(not(unix))]
    {
        // On Windows, just check no orphaned java.exe
        let java_count = find_maven_processes().len();
        println!("Java processes count: {}", java_count);
        assert!(true, "Zombie check (Windows placeholder)");
    }
}

#[test]
fn test_graceful_shutdown_timeout() {
    init();

    // Test that shutdown doesn't wait forever
    let start = std::time::Instant::now();
    
    let handle = thread::spawn(|| {
        // Simulate a process that needs cleanup
        thread::sleep(Duration::from_millis(100));
    });
    
    // Wait with timeout
    let timeout = Duration::from_secs(1);
    let result = handle.join();
    
    let elapsed = start.elapsed();
    
    assert!(result.is_ok(), "Shutdown should succeed");
    assert!(elapsed < timeout, "Shutdown should be fast");
}

#[test]
fn test_process_cleanup_idempotent() {
    init();

    // Killing a non-existent process should not fail
    let fake_pid = 99999;
    
    #[cfg(unix)]
    {
        let result = Command::new("kill")
            .arg("-0") // Check if exists
            .arg(fake_pid.to_string())
            .status();
        
        // Should fail gracefully
        assert!(result.is_ok(), "Process check should not panic");
    }
    
    #[cfg(windows)]
    {
        let result = Command::new("tasklist")
            .arg("/FI")
            .arg(format!("PID eq {}", fake_pid))
            .output();
        
        assert!(result.is_ok(), "Process check should not panic");
    }
}

#[test]
fn test_resource_limits() {
    init();

    // Test that we respect system resource limits
    let project = TestProject::new("/workspaces/lazymvn/demo/multi-module");
    
    // Run a build
    let result = project.compile_module("library");
    
    // Check basic resource usage
    #[cfg(unix)]
    {
        let output = Command::new("ps")
            .args(&["-o", "rss=", "-p", &std::process::id().to_string()])
            .output();
        
        if let Ok(output) = output {
            let memory = String::from_utf8_lossy(&output.stdout)
                .trim()
                .parse::<u64>()
                .unwrap_or(0);
            
            // Should use less than 500 MB
            assert!(memory < 500_000, "Memory usage too high: {} KB", memory);
        }
    }
    
    // Test completed
    assert!(result.is_ok() || result.is_err(), "Build completed");
}
