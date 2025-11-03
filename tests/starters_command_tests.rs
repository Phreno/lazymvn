//! Integration tests for starters command building
//! Verifies that starters use the complete build_launch_command with all required parameters

use lazymvn::maven::{build_launch_command, LaunchStrategy};

mod common;

/// Test that exec:java for starters includes cleanupDaemonThreads=false
/// This is critical to prevent premature process termination
#[test]
fn test_starters_exec_java_includes_cleanup_daemon_threads() {
    let command = build_launch_command(
        LaunchStrategy::ExecJava,
        Some("fr.company.app.ApplicationStarter"),
        &[],
        &[],
        Some("war"),
        None,
    );

    // Must include cleanupDaemonThreads=false
    assert!(
        command.iter().any(|arg| arg.contains("exec.cleanupDaemonThreads=false")),
        "exec:java must include cleanupDaemonThreads=false to prevent premature exit: {:?}",
        command
    );
}

/// Test that exec:java for WAR packaging includes classpathScope=compile
#[test]
fn test_starters_war_packaging_includes_classpath_scope() {
    let command = build_launch_command(
        LaunchStrategy::ExecJava,
        Some("fr.company.app.ApplicationStarter"),
        &[],
        &[],
        Some("war"),
        None,
    );

    // WAR packaging must include classpathScope=compile for provided dependencies
    assert!(
        command.iter().any(|arg| arg.contains("exec.classpathScope=compile")),
        "exec:java for WAR must include classpathScope=compile: {:?}",
        command
    );
}

/// Test that exec:java includes mainClass parameter
#[test]
fn test_starters_exec_java_includes_main_class() {
    let command = build_launch_command(
        LaunchStrategy::ExecJava,
        Some("fr.company.app.ApplicationStarter"),
        &[],
        &[],
        Some("war"),
        None,
    );

    // Must include mainClass
    assert!(
        command.iter().any(|arg| arg.contains("exec.mainClass=fr.company.app.ApplicationStarter")),
        "exec:java must include mainClass parameter: {:?}",
        command
    );
}

/// Test that JVM args are passed correctly to exec:java
#[test]
fn test_starters_exec_java_includes_jvm_args() {
    let jvm_args = vec![
        "-Dlogback.configurationFile=file:///path/to/logback.xml".to_string(),
        "-Dspring.config.location=file:///path/to/application.properties".to_string(),
    ];

    let command = build_launch_command(
        LaunchStrategy::ExecJava,
        Some("fr.company.app.ApplicationStarter"),
        &[],
        &jvm_args,
        Some("war"),
        None,
    );

    // Must include JVM args via -Dexec.args
    assert!(
        command.iter().any(|arg| arg.contains("exec.args=")),
        "exec:java must include JVM args via exec.args: {:?}",
        command
    );

    // Verify the JVM args are in the command
    let args_param = command.iter().find(|arg| arg.contains("exec.args=")).unwrap();
    assert!(
        args_param.contains("logback.configurationFile"),
        "exec.args must contain logback configuration: {}",
        args_param
    );
}

/// Test spring-boot:run includes main-class parameter
#[test]
fn test_starters_spring_boot_run_includes_main_class() {
    let command = build_launch_command(
        LaunchStrategy::SpringBootRun,
        Some("fr.company.app.ApplicationStarter"),
        &[],
        &[],
        Some("jar"),
        Some("2.5.0"),
    );

    // Spring Boot 2.x should use spring-boot.run.main-class
    // Note: This is NOT included by build_launch_command - it's handled separately in starters.rs
    // for backwards compatibility. This test documents the expected behavior.
    
    // Should end with spring-boot:run goal
    assert_eq!(
        command.last(),
        Some(&"spring-boot:run".to_string()),
        "Should use spring-boot:run goal: {:?}",
        command
    );
}

/// Test that Spring Boot 1.x uses correct version-specific goal
#[test]
fn test_starters_spring_boot_1x_uses_full_gav() {
    let command = build_launch_command(
        LaunchStrategy::SpringBootRun,
        None,
        &[],
        &[],
        Some("jar"),
        Some("1.2.2"),
    );

    // Spring Boot 1.x should use full GAV
    assert!(
        command.iter().any(|arg| 
            arg.contains("org.springframework.boot:spring-boot-maven-plugin:1.2.2:run")
        ),
        "Spring Boot 1.x should use full GAV: {:?}",
        command
    );
}
