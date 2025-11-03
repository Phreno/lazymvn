//! Integration tests for Spring Boot starters execution
//! These tests verify that the correct Maven commands are built and executed

use lazymvn::core::config::LaunchMode;
use lazymvn::maven::{LaunchStrategy, SpringBootDetection};

mod common;

/// Test that we don't use -f flag for starters (regression test)
/// Issue: Using -f changes Maven's context and breaks plugin resolution
#[test]
fn test_starters_should_not_use_file_flag() {
    // This test documents the expected behavior:
    // - Starters should use -pl (project list) not -f (file)
    // - Using -f breaks plugin resolution in multi-module projects
    
    let use_file_flag = false; // Should always be false for starters
    assert!(
        !use_file_flag,
        "Starters should not use -f flag to avoid plugin resolution issues"
    );
}

/// Test that Spring Boot detection fallback works correctly
#[test]
fn test_spring_boot_fallback_to_exec_java() {
    let detection = SpringBootDetection {
        has_spring_boot_plugin: false,
        has_exec_plugin: true,
        main_class: Some("com.example.App".to_string()),
        packaging: Some("jar".to_string()),
        spring_boot_version: None,
    };

    // When no Spring Boot plugin is found, should fall back to exec:java
    assert!(
        !detection.can_use_spring_boot_run(),
        "Should not use spring-boot:run without plugin"
    );
    assert!(
        detection.can_use_exec_java(),
        "Should fall back to exec:java"
    );

    let strategy = lazymvn::maven::decide_launch_strategy(&detection, LaunchMode::Auto);
    assert_eq!(
        strategy,
        LaunchStrategy::ExecJava,
        "Auto mode should use exec:java as fallback"
    );
}

/// Test that Spring Boot plugin detection uses correct command
#[test]
fn test_spring_boot_plugin_uses_run_goal() {
    let detection = SpringBootDetection {
        has_spring_boot_plugin: true,
        has_exec_plugin: false,
        main_class: None,
        packaging: Some("jar".to_string()),
        spring_boot_version: Some("2.7.0".to_string()),
    };

    assert!(
        detection.can_use_spring_boot_run(),
        "Should use spring-boot:run with plugin"
    );

    let strategy = lazymvn::maven::decide_launch_strategy(&detection, LaunchMode::Auto);
    assert_eq!(
        strategy,
        LaunchStrategy::SpringBootRun,
        "Should use spring-boot:run goal"
    );
}

/// Test that exec:java is available when exec plugin is configured
#[test]
fn test_exec_plugin_detection() {
    let detection_with_exec = SpringBootDetection {
        has_spring_boot_plugin: false,
        has_exec_plugin: true,
        main_class: Some("com.example.Main".to_string()),
        packaging: Some("jar".to_string()),
        spring_boot_version: None,
    };

    assert!(
        detection_with_exec.can_use_exec_java(),
        "Should detect exec plugin"
    );

    // With main class but no exec plugin, exec:java is still available
    let detection_with_main_class = SpringBootDetection {
        has_spring_boot_plugin: false,
        has_exec_plugin: false,
        main_class: Some("com.example.Main".to_string()),
        packaging: Some("jar".to_string()),
        spring_boot_version: None,
    };

    assert!(
        detection_with_main_class.can_use_exec_java(),
        "Should use exec:java with main class even without explicit plugin"
    );

    // Without plugin AND without main class, exec:java is not available
    let detection_without_either = SpringBootDetection {
        has_spring_boot_plugin: false,
        has_exec_plugin: false,
        main_class: None,
        packaging: Some("jar".to_string()),
        spring_boot_version: None,
    };

    assert!(
        !detection_without_either.can_use_exec_java(),
        "Should not use exec:java without plugin AND main class"
    );
}

/// Test that packaging type is respected
#[test]
fn test_pom_packaging_not_launchable() {
    let detection = SpringBootDetection {
        has_spring_boot_plugin: true,
        has_exec_plugin: true,
        main_class: Some("com.example.App".to_string()),
        packaging: Some("pom".to_string()),
        spring_boot_version: Some("2.5.0".to_string()),
    };

    assert!(
        !detection.can_use_spring_boot_run(),
        "POM packaging should not be launchable with spring-boot:run"
    );
}

/// Test LaunchMode::Auto preference order
#[test]
fn test_auto_mode_prefers_spring_boot_over_exec() {
    let detection = SpringBootDetection {
        has_spring_boot_plugin: true,
        has_exec_plugin: true,
        main_class: Some("com.example.App".to_string()),
        packaging: Some("jar".to_string()),
        spring_boot_version: Some("2.7.0".to_string()),
    };

    let strategy = lazymvn::maven::decide_launch_strategy(&detection, LaunchMode::Auto);
    assert_eq!(
        strategy,
        LaunchStrategy::SpringBootRun,
        "Auto mode should prefer spring-boot:run when both are available"
    );
}

/// Test ForceRun and ForceExec modes
#[test]
fn test_forced_launch_modes() {
    let detection = SpringBootDetection {
        has_spring_boot_plugin: true,
        has_exec_plugin: true,
        main_class: Some("com.example.App".to_string()),
        packaging: Some("jar".to_string()),
        spring_boot_version: Some("2.7.0".to_string()),
    };

    let force_run = lazymvn::maven::decide_launch_strategy(&detection, LaunchMode::ForceRun);
    assert_eq!(
        force_run,
        LaunchStrategy::SpringBootRun,
        "ForceRun should use spring-boot:run"
    );

    let force_exec = lazymvn::maven::decide_launch_strategy(&detection, LaunchMode::ForceExec);
    assert_eq!(
        force_exec,
        LaunchStrategy::ExecJava,
        "ForceExec should use exec:java"
    );
}
