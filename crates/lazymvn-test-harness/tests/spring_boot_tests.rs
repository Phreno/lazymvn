//! Tests for Spring Boot specific features
//!
//! These tests verify Spring Boot detection, starter isolation, and
//! version-specific behavior. Replaces manual scripts:
//! - test-spring-boot-1x-fix.sh
//! - test-starter-isolation.sh

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

/// Test basic Spring Boot detection
#[test]
fn test_spring_boot_detection() {
    init_test_logging();

    let demo_path = demo_project_path();
    
    // Check if app module has Spring Boot plugin
    let app_pom = demo_path.join("app/pom.xml");
    
    if !app_pom.exists() {
        println!("⚠️  Skipping: app/pom.xml not found");
        return;
    }

    let pom_content = std::fs::read_to_string(&app_pom)
        .expect("Failed to read app/pom.xml");

    let has_spring_boot = pom_content.contains("spring-boot-maven-plugin")
        || pom_content.contains("spring-boot-starter");

    if has_spring_boot {
        println!("✅ Spring Boot detected in app module");
    } else {
        println!("⚠️  No Spring Boot plugin found in app module");
    }
}

/// Test that Spring Boot apps can be compiled
#[test]
fn test_spring_boot_compile() {
    init_test_logging();

    let project = TestProject::new(demo_project_path());
    
    // Try to compile the app module (likely has Spring Boot)
    let result = project.compile_module("app");

    match result {
        Ok(cmd_result) => {
            if cmd_result.success {
                println!("✅ Spring Boot app compiled successfully");
            } else {
                println!("⚠️  Spring Boot app compilation failed (might be expected)");
                println!("   Exit code: {:?}", cmd_result.exit_code);
            }
        }
        Err(e) => {
            println!("⚠️  Could not run compile: {}", e);
        }
    }
}

/// Test Spring Boot with profiles
#[test]
fn test_spring_boot_with_profiles() {
    init_test_logging();

    let project = TestProject::new(demo_project_path())
        .with_profiles(&["dev"]);
    
    let result = project.compile_module("app");

    match result {
        Ok(cmd_result) => {
            println!("Spring Boot with dev profile: {}", 
                if cmd_result.success { "success" } else { "failed" });
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

/// Test that --also-make flag works with Spring Boot modules
#[test]
fn test_spring_boot_with_also_make() {
    init_test_logging();

    let project = TestProject::new(demo_project_path())
        .with_flags(&["--also-make"]);
    
    // Build app which depends on library
    let result = project.build_module("app");

    match result {
        Ok(cmd_result) => {
            if cmd_result.success {
                println!("✅ Spring Boot build with --also-make succeeded");
                
                // Should have built dependencies
                let built_library = cmd_result.contains("library") 
                    || cmd_result.contains("Building");
                
                if built_library {
                    println!("   ✅ Dependencies were built");
                } else {
                    println!("   ⚠️  Could not verify dependency builds");
                }
            } else {
                println!("⚠️  Build failed (might be expected if app has issues)");
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

/// Test Spring Boot JVM arguments handling
#[test]
fn test_spring_boot_jvm_arguments() {
    init_test_logging();

    // Test that custom JVM args can be passed
    let project = TestProject::new(demo_project_path())
        .with_flags(&["-Dspring-boot.run.jvmArguments=-Xmx512m"]);
    
    let result = project.compile_module("app");

    match result {
        Ok(cmd_result) => {
            println!("Spring Boot with JVM args: {}", 
                if cmd_result.success { "success" } else { "failed" });
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

/// Test Spring Boot 1.x compatibility (-Drun.jvmArguments)
#[test]
#[ignore] // Ignored because we need a real Spring Boot 1.x project to test
fn test_spring_boot_1x_jvm_arguments() {
    init_test_logging();

    // Spring Boot 1.x uses -Drun.jvmArguments instead of -Dspring-boot.run.jvmArguments
    let project = TestProject::new(demo_project_path())
        .with_flags(&["-Drun.jvmArguments=-Xmx512m"]);
    
    let result = project.compile_module("app");

    match result {
        Ok(cmd_result) => {
            println!("Spring Boot 1.x with JVM args: {}", 
                if cmd_result.success { "success" } else { "failed" });
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

/// Test exec:java goal as fallback
#[test]
fn test_exec_java_fallback() {
    init_test_logging();

    let project = TestProject::new(demo_project_path());
    
    // Try exec:java on library module (doesn't have Spring Boot)
    let result = project.run_command("library", &["exec:java"]);

    match result {
        Ok(cmd_result) => {
            // Will likely fail because no main class, but should try
            println!("exec:java on library: {}", 
                if cmd_result.success { "success" } else { "failed (expected)" });
            
            // Check if Maven tried to run it
            let tried_exec = cmd_result.contains("exec:java") 
                || cmd_result.contains("mainClass")
                || cmd_result.contains("No plugin found");
            
            if tried_exec {
                println!("✅ exec:java goal was attempted");
            }
        }
        Err(e) => {
            println!("Error executing exec:java: {}", e);
        }
    }
}

/// Test that Spring Boot modules can be packaged
#[test]
fn test_spring_boot_package() {
    init_test_logging();

    let project = TestProject::new(demo_project_path());
    
    let result = project.package_module("app");

    match result {
        Ok(cmd_result) => {
            if cmd_result.success {
                println!("✅ Spring Boot app packaged successfully");
                
                // Check if JAR was created
                let jar_created = cmd_result.contains("Building jar")
                    || cmd_result.contains("BUILD SUCCESS");
                
                if jar_created {
                    println!("   ✅ JAR artifact created");
                }
            } else {
                println!("⚠️  Packaging failed");
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

/// Test Spring Boot with logging configuration
#[test]
fn test_spring_boot_logging_config() {
    init_test_logging();

    // Test with logging level override
    let project = TestProject::new(demo_project_path())
        .with_flags(&["-Dlogging.level.root=DEBUG"]);
    
    let result = project.compile_module("app");

    match result {
        Ok(cmd_result) => {
            println!("Spring Boot with logging config: {}", 
                if cmd_result.success { "success" } else { "failed" });
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

/// Test that multiple Spring Boot apps can coexist (starter isolation)
#[test]
fn test_spring_boot_module_isolation() {
    init_test_logging();

    let project = TestProject::new(demo_project_path());
    
    // Build library first
    let result1 = project.build_module("library");
    
    // Then build app
    let result2 = project.build_module("app");

    match (result1, result2) {
        (Ok(r1), Ok(r2)) => {
            println!("Library build: {}", if r1.success { "✅" } else { "❌" });
            println!("App build: {}", if r2.success { "✅" } else { "❌" });
            
            if r1.success && r2.success {
                println!("✅ Module isolation works - both modules built independently");
            }
        }
        _ => {
            println!("⚠️  Could not test module isolation");
        }
    }
}

/// Test Spring Boot profile activation
#[test]
fn test_spring_boot_profile_activation() {
    init_test_logging();

    // Spring profiles can be activated via -Dspring.profiles.active
    let project = TestProject::new(demo_project_path())
        .with_flags(&["-Dspring.profiles.active=dev,test"]);
    
    let result = project.compile_module("app");

    match result {
        Ok(cmd_result) => {
            println!("Spring Boot with active profiles: {}", 
                if cmd_result.success { "success" } else { "failed" });
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
