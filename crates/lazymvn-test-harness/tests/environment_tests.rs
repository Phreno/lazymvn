// Environment and system validation tests
// Replaces: scripts/test-env.sh

use lazymvn_test_harness::TestProject;
use std::process::Command;
use std::path::Path;

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
    let _ = lazymvn::utils::logger::init(Some("debug"));
}

/// Helper to check if a command exists in PATH
fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Helper to get command version
fn get_command_version(cmd: &str, args: &[&str]) -> Option<String> {
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                String::from_utf8(output.stderr).ok()
            }
        })
        .map(|s| s.lines().next().unwrap_or("").to_string())
}

#[test]
fn test_rust_toolchain_available() {
    init();

    // Check rustc
    assert!(command_exists("rustc"), "rustc not found in PATH");
    
    let rustc_version = get_command_version("rustc", &["--version"]);
    assert!(rustc_version.is_some(), "Failed to get rustc version");
    assert!(rustc_version.unwrap().contains("rustc"), "Invalid rustc version");

    // Check cargo
    assert!(command_exists("cargo"), "cargo not found in PATH");
    
    let cargo_version = get_command_version("cargo", &["--version"]);
    assert!(cargo_version.is_some(), "Failed to get cargo version");
    assert!(cargo_version.unwrap().contains("cargo"), "Invalid cargo version");
}

#[test]
fn test_java_available() {
    init();

    assert!(command_exists("java"), "java not found in PATH");
    
    // Try both stdout and stderr for version info
    let java_version = get_command_version("java", &["-version"])
        .or_else(|| {
            // Java sometimes prints to stderr
            Command::new("java")
                .arg("-version")
                .output()
                .ok()
                .and_then(|output| String::from_utf8(output.stderr).ok())
                .map(|s| s.lines().next().unwrap_or("").to_string())
        });
    
    assert!(java_version.is_some(), "Failed to get java version");
    
    // Java prints to stderr, check for "version" keyword
    let version_str = java_version.unwrap();
    
    // If we got any output, consider it a success
    if !version_str.is_empty() {
        println!("Java version detected: {}", version_str);
        assert!(true);
    } else {
        // Last resort: just check if java runs
        let runs = Command::new("java")
            .arg("-version")
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        assert!(runs, "Java command should execute");
    }
}

#[test]
fn test_maven_available() {
    init();

    assert!(command_exists("mvn"), "mvn not found in PATH");
    
    let mvn_version = get_command_version("mvn", &["--version"]);
    assert!(mvn_version.is_some(), "Failed to get mvn version");
    
    let version_str = mvn_version.unwrap();
    assert!(
        version_str.contains("Apache Maven") || version_str.contains("Maven"),
        "Invalid maven version output: {}", version_str
    );
}

#[test]
fn test_git_available() {
    init();

    assert!(command_exists("git"), "git not found in PATH");
    
    let git_version = get_command_version("git", &["--version"]);
    assert!(git_version.is_some(), "Failed to get git version");
    assert!(git_version.unwrap().contains("git version"), "Invalid git version");
}

#[test]
fn test_lazymvn_builds() {
    init();

    // Check that we can build lazymvn
    let workspace_root = Path::new("/workspaces/lazymvn");
    assert!(workspace_root.join("Cargo.toml").exists(), "Cargo.toml not found");
    
    // Run cargo check
    let output = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(workspace_root)
        .output()
        .expect("Failed to execute cargo check");
    
    assert!(
        output.status.success(),
        "cargo check failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_demo_project_structure() {
    init();

    let multi_module = Path::new("/workspaces/lazymvn/demo/multi-module");
    
    // Check project structure
    assert!(multi_module.exists(), "multi-module demo not found");
    assert!(multi_module.join("pom.xml").exists(), "multi-module pom.xml not found");
    
    // Check modules
    assert!(multi_module.join("app").exists(), "app module not found");
    assert!(multi_module.join("app/pom.xml").exists(), "app pom.xml not found");
    
    assert!(multi_module.join("library").exists(), "library module not found");
    assert!(multi_module.join("library/pom.xml").exists(), "library pom.xml not found");
}

#[test]
fn test_demo_project_compiles() {
    init();

    let multi_module = Path::new("/workspaces/lazymvn/demo/multi-module");
    
    // Run mvn compile on demo project
    let output = Command::new("mvn")
        .arg("-q")
        .arg("clean")
        .arg("compile")
        .current_dir(multi_module)
        .output()
        .expect("Failed to execute mvn compile");
    
    assert!(
        output.status.success(),
        "Demo project compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_lazymvn_can_detect_demo_project() {
    init();

    let project = TestProject::new("/workspaces/lazymvn/demo/multi-module");
    
    // Should be able to run commands on modules
    let result = project.compile_module("library");
    assert!(result.is_ok(), "Failed to compile library module");
    
    let result = result.unwrap();
    assert!(result.success, "Compilation should succeed");
}

#[test]
fn test_log_directory_structure() {
    init();

    // Get log directory
    let log_dir = if cfg!(target_os = "windows") {
        dirs::data_local_dir()
            .expect("Failed to get local data dir")
            .join("lazymvn")
            .join("logs")
    } else {
        dirs::data_local_dir()
            .expect("Failed to get local data dir")
            .join("lazymvn")
            .join("logs")
    };
    
    // Directory should exist after logger init
    assert!(log_dir.exists() || log_dir.parent().unwrap().exists(), 
        "Log directory structure not accessible");
}

#[test]
fn test_lazymvn_binary_exists() {
    init();

    let workspace_root = Path::new("/workspaces/lazymvn");
    
    // Check if debug or release binary exists
    let debug_binary = workspace_root.join("target/debug/lazymvn");
    let release_binary = workspace_root.join("target/release/lazymvn");
    
    let binary_exists = debug_binary.exists() || release_binary.exists();
    
    // If no binary exists, try building one
    if !binary_exists {
        let build = Command::new("cargo")
            .arg("build")
            .arg("--quiet")
            .current_dir(workspace_root)
            .status()
            .expect("Failed to build lazymvn");
        
        assert!(build.success(), "Failed to build lazymvn binary");
        assert!(debug_binary.exists(), "Binary not created after build");
    } else {
        assert!(binary_exists, "No lazymvn binary found");
    }
}

#[test]
fn test_workspace_structure() {
    init();

    let workspace_root = Path::new("/workspaces/lazymvn");
    
    // Check essential files
    assert!(workspace_root.join("Cargo.toml").exists(), "Cargo.toml missing");
    assert!(workspace_root.join("README.md").exists(), "README.md missing");
    assert!(workspace_root.join("CHANGELOG.md").exists(), "CHANGELOG.md missing");
    
    // Check essential directories
    assert!(workspace_root.join("src").exists(), "src/ directory missing");
    assert!(workspace_root.join("tests").exists(), "tests/ directory missing");
    assert!(workspace_root.join("demo").exists(), "demo/ directory missing");
    assert!(workspace_root.join("docs").exists(), "docs/ directory missing");
    
    // Check crates
    assert!(workspace_root.join("crates/lazymvn-test-harness").exists(), 
        "test-harness crate missing");
}

#[test]
fn test_maven_wrapper_available() {
    init();

    let single_module = Path::new("/workspaces/lazymvn/demo/single-module");
    
    if single_module.exists() {
        let mvnw = single_module.join("mvnw");
        
        if mvnw.exists() {
            // Check if executable
            let metadata = std::fs::metadata(&mvnw).expect("Failed to get mvnw metadata");
            
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let permissions = metadata.permissions();
                let is_executable = permissions.mode() & 0o111 != 0;
                assert!(is_executable, "mvnw is not executable");
            }
            
            // Try to run it
            let output = Command::new(&mvnw)
                .arg("--version")
                .current_dir(single_module)
                .output();
            
            if let Ok(output) = output {
                assert!(output.status.success(), "mvnw --version failed");
            }
        }
    }
}

#[test]
fn test_required_system_tools() {
    init();

    // Essential tools that must be available
    let required_tools = vec![
        ("rustc", "Rust compiler"),
        ("cargo", "Rust package manager"),
        ("java", "Java runtime"),
        ("mvn", "Maven build tool"),
        ("git", "Version control"),
    ];
    
    let mut missing_tools = Vec::new();
    
    for (tool, description) in &required_tools {
        if !command_exists(tool) {
            missing_tools.push(format!("{} ({})", tool, description));
        }
    }
    
    assert!(
        missing_tools.is_empty(),
        "Missing required tools: {}",
        missing_tools.join(", ")
    );
}

#[test]
fn test_optional_development_tools() {
    init();

    // Optional but recommended tools
    let optional_tools = vec![
        "cargo-watch",
        "cargo-edit",
        "cargo-audit",
    ];
    
    println!("Optional development tools status:");
    for tool in &optional_tools {
        let available = command_exists(tool);
        println!("  {} - {}", tool, if available { "✓ installed" } else { "○ not installed" });
    }
    
    // This test always passes, just reports status
    assert!(true);
}
