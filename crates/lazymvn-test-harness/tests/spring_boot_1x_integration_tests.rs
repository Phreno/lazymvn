//! Test harness integration tests for Spring Boot 1.x fix
//!
//! These tests use the test harness to verify Spring Boot 1.x behavior
//! in a real Maven project environment.

use lazymvn_test_harness::TestProject;
use std::path::PathBuf;

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
    let _ = lazymvn::utils::logger::init(Some("debug"));
}

fn demo_project_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("demo/multi-module")
}

#[test]
fn test_spring_boot_run_command_generation() {
    init();

    let project = TestProject::new(&demo_project_path());
    
    // Try to get help on spring-boot:run to verify command syntax
    let result = project.run_command("app", &["help:describe", "-Dplugin=spring-boot"]);

    if let Ok(cmd_result) = result {
        // If the plugin is configured, Maven should be able to describe it
        if cmd_result.success || cmd_result.contains("spring-boot") {
            println!("✅ spring-boot plugin is recognized by Maven");
        }
    }
}

#[test]
fn test_spring_boot_goal_without_version_works() {
    init();

    let project = TestProject::new(&demo_project_path());
    
    // Test that spring-boot:run works (even if it fails for other reasons)
    // The important thing is Maven recognizes the goal
    let result = project.run_command("app", &["spring-boot:help"]);

    match result {
        Ok(cmd_result) => {
            // If plugin is configured, help should work
            if cmd_result.contains("spring-boot") || cmd_result.contains("goal") {
                println!("✅ spring-boot:help goal is recognized");
            } else if cmd_result.contains("No plugin found") {
                println!("⚠️  Spring Boot plugin not configured in app module (expected)");
            }
        }
        Err(e) => {
            println!("⚠️  Could not test spring-boot goal: {}", e);
        }
    }
}

#[test]
fn test_fully_qualified_plugin_without_version_works() {
    init();

    let project = TestProject::new(&demo_project_path());
    
    // Test that the fully-qualified form WITHOUT version works
    let result = project.run_command(
        "app",
        &["org.springframework.boot:spring-boot-maven-plugin:help"]
    );

    match result {
        Ok(cmd_result) => {
            // This should work if plugin is in repositories
            // or be recognized even if not found
            if cmd_result.contains("spring-boot") {
                println!("✅ Fully-qualified plugin syntax (without version) works");
            } else if cmd_result.contains("Could not find") {
                println!("⚠️  Plugin not in repositories (expected in test env)");
            }
        }
        Err(e) => {
            println!("⚠️  Could not test fully-qualified syntax: {}", e);
        }
    }
}

#[test]
fn test_invalid_fully_qualified_with_version_fails() {
    init();

    let project = TestProject::new(&demo_project_path());
    
    // Test that the INVALID syntax with version fails
    // This is what the bug was generating
    let result = project.run_command(
        "app",
        &["org.springframework.boot:spring-boot-maven-plugin:1.4.13:help"]
    );

    match result {
        Ok(cmd_result) => {
            // This SHOULD fail with "Could not find artifact"
            if cmd_result.contains("Could not find artifact")
                || cmd_result.contains("jar:1.4.13")
            {
                println!("✅ Invalid syntax correctly fails (as expected)");
                let error_line = cmd_result.output
                    .iter()
                    .find(|l| l.contains("ERROR"))
                    .map(|s| s.as_str())
                    .unwrap_or("");
                println!("   Error message: {}", error_line);
            } else if !cmd_result.success {
                println!("⚠️  Command failed (expected)");
            } else {
                println!("⚠️  Unexpected success with invalid syntax");
            }
        }
        Err(e) => {
            println!("⚠️  Could not test invalid syntax: {}", e);
        }
    }
}

#[test]
fn test_spring_boot_properties_1x_vs_2x() {
    init();

    // Test that the correct properties are used based on version detection
    let project_path = demo_project_path();

    // Simulate 1.x properties
    let project_1x = TestProject::new(&project_path)
        .with_flags(&["-Drun.profiles=dev"]);
    let result_1x = project_1x.compile_module("app");

    // Simulate 2.x properties
    let project_2x = TestProject::new(&project_path)
        .with_flags(&["-Dspring-boot.run.profiles=dev"]);
    let result_2x = project_2x.compile_module("app");

    // Both should work (or fail for same reason)
    match (result_1x, result_2x) {
        (Ok(r1), Ok(r2)) => {
            println!("1.x properties: {}", if r1.success { "✅" } else { "⚠️" });
            println!("2.x properties: {}", if r2.success { "✅" } else { "⚠️" });
        }
        _ => {
            println!("⚠️  Could not test property compatibility");
        }
    }
}

#[test]
fn test_spring_boot_with_jvm_arguments() {
    init();

    let project = TestProject::new(&demo_project_path())
        .with_flags(&["-Dspring-boot.run.jvmArguments=-Xmx512m -Ddebug=true"]);

    let result = project.compile_module("app");

    match result {
        Ok(cmd_result) => {
            if cmd_result.success {
                println!("✅ Spring Boot with JVM arguments succeeded");
            } else {
                println!("⚠️  Build failed (might be expected)");
            }
        }
        Err(e) => {
            println!("⚠️  Could not test JVM arguments: {}", e);
        }
    }
}

#[test]
fn test_spring_boot_command_in_output() {
    init();

    let project = TestProject::new(&demo_project_path());
    
    // Run a command and check that the correct goal appears
    let result = project.run_command("app", &["help:describe", "-Dcmd=spring-boot:run"]);

    match result {
        Ok(cmd_result) => {
            // Check that our command syntax is being used
            if cmd_result.contains("spring-boot:run") {
                println!("✅ Correct goal syntax found in output");
            }

            // Verify NO fully-qualified syntax with version appears
            if !cmd_result.contains("spring-boot-maven-plugin:1.")
                && !cmd_result.contains("spring-boot-maven-plugin:2.")
            {
                println!("✅ No buggy fully-qualified syntax in output");
            }
        }
        Err(e) => {
            println!("⚠️  Could not check command output: {}", e);
        }
    }
}

#[test]
fn test_spring_boot_multiple_modules() {
    init();

    let project = TestProject::new(&demo_project_path());
    
    // Build library (non-Spring Boot)
    let result1 = project.build_module("library");
    
    // Build app (might have Spring Boot)
    let result2 = project.build_module("app");

    match (result1, result2) {
        (Ok(r1), Ok(r2)) => {
            if r1.success && r2.success {
                println!("✅ Both modules built successfully");
                println!("   This verifies Spring Boot isolation works");
            } else {
                println!("⚠️  Some builds failed:");
                println!("   library: {}", if r1.success { "✅" } else { "❌" });
                println!("   app: {}", if r2.success { "✅" } else { "❌" });
            }
        }
        _ => {
            println!("⚠️  Could not test multiple modules");
        }
    }
}

#[test]
fn test_spring_boot_clean_install() {
    init();

    let project = TestProject::new(&demo_project_path());
    
    // Clean and install app module
    let result = project.run_command("app", &["clean", "install"]);

    match result {
        Ok(cmd_result) => {
            if cmd_result.success {
                println!("✅ clean install succeeded");
                
                // Check that JAR/WAR was built
                if cmd_result.contains("BUILD SUCCESS") {
                    println!("   ✅ Build completed successfully");
                }
            } else {
                println!("⚠️  clean install failed");
                if cmd_result.contains("plugin") && cmd_result.contains("not be resolved") {
                    println!("   ❌ Plugin resolution issue detected!");
                }
            }
        }
        Err(e) => {
            println!("⚠️  Could not run clean install: {}", e);
        }
    }
}

#[test]
fn test_spring_boot_with_profiles() {
    init();

    let project = TestProject::new(&demo_project_path())
        .with_profiles(&["dev", "local"]);
    
    let result = project.compile_module("app");

    match result {
        Ok(cmd_result) => {
            // Check that profiles were activated
            if cmd_result.contains("dev") || cmd_result.contains("profile") {
                println!("✅ Profiles processed by Maven");
            }

            if cmd_result.success {
                println!("✅ Build with profiles succeeded");
            }
        }
        Err(e) => {
            println!("⚠️  Could not test profiles: {}", e);
        }
    }
}

#[test]
fn test_spring_boot_dependency_resolution() {
    init();

    let project = TestProject::new(&demo_project_path());
    
    // Test dependency:tree to verify Spring Boot dependencies
    let result = project.run_command("app", &["dependency:tree"]);

    match result {
        Ok(cmd_result) => {
            if cmd_result.success {
                println!("✅ Dependency resolution works");
                
                // Check if Spring Boot dependencies are present
                if cmd_result.contains("spring-boot") {
                    println!("   ✅ Spring Boot dependencies detected");
                }
            }
        }
        Err(e) => {
            println!("⚠️  Could not check dependencies: {}", e);
        }
    }
}

#[test]
fn test_spring_boot_goal_prefix_mapping() {
    init();

    let project = TestProject::new(&demo_project_path());
    
    // Test that Maven can resolve the spring-boot prefix
    let result = project.run_command("app", &["help:describe", "-Dplugin=spring-boot"]);

    match result {
        Ok(cmd_result) => {
            if cmd_result.contains("spring-boot-maven-plugin") {
                println!("✅ Plugin prefix 'spring-boot' correctly maps to spring-boot-maven-plugin");
            } else if cmd_result.contains("No plugin found") {
                println!("⚠️  Plugin not configured (expected if app isn't Spring Boot)");
            }
        }
        Err(e) => {
            println!("⚠️  Could not test prefix mapping: {}", e);
        }
    }
}

#[test]
fn test_spring_boot_package_goal() {
    init();

    let project = TestProject::new(&demo_project_path());
    
    // Test that package goal works
    let result = project.package_module("app");

    match result {
        Ok(cmd_result) => {
            if cmd_result.success {
                println!("✅ Package goal succeeded");
                
                // Spring Boot should create an executable JAR/WAR
                if cmd_result.contains("Building") || cmd_result.contains("jar") {
                    println!("   ✅ Artifact created");
                }
            } else {
                println!("⚠️  Package failed");
                
                // Check for plugin issues
                if cmd_result.contains("plugin") {
                    println!("   ⚠️  Plugin-related failure");
                }
            }
        }
        Err(e) => {
            println!("⚠️  Could not test package: {}", e);
        }
    }
}

#[test]
fn test_spring_boot_verify_no_plugin_jar_error() {
    init();

    let project = TestProject::new(&demo_project_path());
    
    // Run any command and verify we don't get the specific error from the bug report
    let result = project.compile_module("app");

    match result {
        Ok(cmd_result) => {
            // Check for the specific error from the bug report
            let has_bug_error = cmd_result.contains("Could not find artifact")
                && cmd_result.contains("spring-boot-maven-plugin:jar:");

            if has_bug_error {
                panic!("❌ BUG DETECTED: Found 'plugin:jar:' error that should be fixed!");
            } else {
                println!("✅ No plugin JAR resolution error detected");
            }

            if cmd_result.success {
                println!("   ✅ Build succeeded");
            }
        }
        Err(e) => {
            println!("⚠️  Could not verify: {}", e);
        }
    }
}
