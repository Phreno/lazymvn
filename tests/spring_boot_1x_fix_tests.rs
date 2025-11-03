//! Integration tests for Spring Boot 1.x plugin resolution fix
//!
//! These tests verify that LazyMVN generates the correct Maven command
//! for Spring Boot 1.x projects, avoiding the invalid fully-qualified
//! plugin syntax that caused "plugin JAR not found" errors.
//!
//! Bug: org.springframework.boot:spring-boot-maven-plugin:1.4.13:run (INVALID)
//! Fix: spring-boot:run (VALID)

use lazymvn::maven::{build_launch_command, LaunchStrategy};

#[test]
fn test_spring_boot_1x_does_not_use_fully_qualified_syntax() {
    // Test all common Spring Boot 1.x versions
    let versions = vec![
        "1.0.0.RELEASE",
        "1.1.0.RELEASE",
        "1.2.2.RELEASE",
        "1.3.8.RELEASE",
        "1.4.13",
        "1.5.10.RELEASE",
        "1.5.22.RELEASE",
    ];

    for version in versions {
        let command = build_launch_command(
            LaunchStrategy::SpringBootRun,
            None,
            &[],
            &[],
            Some("jar"),
            Some(version),
        );

        // Should NOT contain the problematic fully-qualified syntax
        assert!(
            !command.iter().any(|arg| arg.contains(&format!(
                "org.springframework.boot:spring-boot-maven-plugin:{}",
                version
            ))),
            "Spring Boot {} should NOT use fully-qualified plugin syntax with version",
            version
        );

        // Should use the simple goal
        assert!(
            command.iter().any(|arg| arg == "spring-boot:run"),
            "Spring Boot {} should use 'spring-boot:run' goal",
            version
        );
    }
}

#[test]
fn test_spring_boot_1x_uses_correct_properties() {
    // Spring Boot 1.x uses different property names than 2.x
    let command = build_launch_command(
        LaunchStrategy::SpringBootRun,
        None,
        &["dev".to_string()],
        &["-Xmx512m".to_string()],
        Some("jar"),
        Some("1.4.13"),
    );

    // Should use 1.x properties
    assert!(
        command.iter().any(|arg| arg.contains("run.profiles=")),
        "Spring Boot 1.x should use -Drun.profiles"
    );

    assert!(
        command.iter().any(|arg| arg.contains("run.jvmArguments=")),
        "Spring Boot 1.x should use -Drun.jvmArguments"
    );

    // Should NOT use 2.x properties
    assert!(
        !command.iter().any(|arg| arg.contains("spring-boot.run.profiles")),
        "Spring Boot 1.x should NOT use -Dspring-boot.run.profiles"
    );

    assert!(
        !command.iter().any(|arg| arg.contains("spring-boot.run.jvmArguments")),
        "Spring Boot 1.x should NOT use -Dspring-boot.run.jvmArguments"
    );
}

#[test]
fn test_spring_boot_2x_uses_correct_properties() {
    // Verify 2.x still works correctly
    let command = build_launch_command(
        LaunchStrategy::SpringBootRun,
        None,
        &["prod".to_string()],
        &["-Xmx1g".to_string()],
        Some("jar"),
        Some("2.7.0"),
    );

    // Should use 2.x properties
    assert!(
        command.iter().any(|arg| arg.contains("spring-boot.run.profiles=")),
        "Spring Boot 2.x should use -Dspring-boot.run.profiles"
    );

    assert!(
        command.iter().any(|arg| arg.contains("spring-boot.run.jvmArguments=")),
        "Spring Boot 2.x should use -Dspring-boot.run.jvmArguments"
    );

    // Should still use simple goal
    assert!(
        command.iter().any(|arg| arg == "spring-boot:run"),
        "Spring Boot 2.x should use 'spring-boot:run' goal"
    );
}

#[test]
fn test_spring_boot_3x_compatibility() {
    // Ensure 3.x also works (uses same properties as 2.x)
    let command = build_launch_command(
        LaunchStrategy::SpringBootRun,
        None,
        &[],
        &[],
        Some("jar"),
        Some("3.1.0"),
    );

    // Should use 2.x/3.x properties
    assert!(
        command.iter().any(|arg| arg == "spring-boot:run"),
        "Spring Boot 3.x should use 'spring-boot:run' goal"
    );
}

#[test]
fn test_spring_boot_version_edge_cases() {
    // Test edge cases in version detection
    let edge_cases = vec![
        ("1", true),           // Just "1"
        ("1.5", true),         // Major.minor
        ("1.5.22", true),      // Full version
        ("1.5.22.RELEASE", true),
        ("2.0.0", false),      // 2.x should not use 1.x properties
        ("2.7.18", false),
        ("3.0.0", false),
    ];

    for (version, is_1x) in edge_cases {
        let command = build_launch_command(
            LaunchStrategy::SpringBootRun,
            None,
            &["test".to_string()],
            &[],
            Some("jar"),
            Some(version),
        );

        if is_1x {
            assert!(
                command.iter().any(|arg| arg.contains("run.profiles=")),
                "Version {} should be detected as 1.x",
                version
            );
        } else {
            assert!(
                command.iter().any(|arg| arg.contains("spring-boot.run.profiles=")),
                "Version {} should be detected as 2.x+",
                version
            );
        }

        // All should use the same goal
        assert!(
            command.iter().any(|arg| arg == "spring-boot:run"),
            "Version {} should use 'spring-boot:run' goal",
            version
        );
    }
}

#[test]
fn test_spring_boot_no_version_defaults_to_2x() {
    // When version is unknown, should default to 2.x properties
    let command = build_launch_command(
        LaunchStrategy::SpringBootRun,
        None,
        &["default".to_string()],
        &[],
        Some("jar"),
        None, // No version
    );

    // Should use 2.x properties as default
    assert!(
        command.iter().any(|arg| arg.contains("spring-boot.run.profiles=")),
        "Unknown version should default to 2.x properties"
    );

    assert!(
        command.iter().any(|arg| arg == "spring-boot:run"),
        "Should use 'spring-boot:run' goal"
    );
}

#[test]
fn test_spring_boot_1x_multiple_profiles() {
    let command = build_launch_command(
        LaunchStrategy::SpringBootRun,
        None,
        &["dev".to_string(), "local".to_string(), "debug".to_string()],
        &[],
        Some("jar"),
        Some("1.5.10.RELEASE"),
    );

    // Should contain all profiles comma-separated
    let profiles_arg = command
        .iter()
        .find(|arg| arg.starts_with("-Drun.profiles="))
        .expect("Should have profiles argument");

    assert!(
        profiles_arg.contains("dev") && profiles_arg.contains("local") && profiles_arg.contains("debug"),
        "Should contain all profiles: {}",
        profiles_arg
    );

    // Should be comma-separated
    assert!(
        profiles_arg.contains(","),
        "Profiles should be comma-separated: {}",
        profiles_arg
    );
}

#[test]
fn test_spring_boot_1x_multiple_jvm_args() {
    let jvm_args = vec![
        "-Xmx512m".to_string(),
        "-Xms256m".to_string(),
        "-Ddebug=true".to_string(),
        "-Dspring.profiles.active=dev".to_string(),
    ];

    let command = build_launch_command(
        LaunchStrategy::SpringBootRun,
        None,
        &[],
        &jvm_args,
        Some("jar"),
        Some("1.4.13"),
    );

    // Should contain all JVM args space-separated in run.jvmArguments
    let jvm_args_arg = command
        .iter()
        .find(|arg| arg.starts_with("-Drun.jvmArguments="))
        .expect("Should have JVM arguments");

    for arg in &jvm_args {
        assert!(
            jvm_args_arg.contains(arg),
            "Should contain JVM arg {}: {}",
            arg,
            jvm_args_arg
        );
    }

    // Should be space-separated
    assert!(
        jvm_args_arg.contains(" "),
        "JVM args should be space-separated: {}",
        jvm_args_arg
    );
}

#[test]
fn test_spring_boot_command_order() {
    let command = build_launch_command(
        LaunchStrategy::SpringBootRun,
        None,
        &["dev".to_string()],
        &["-Xmx512m".to_string()],
        Some("jar"),
        Some("1.4.13"),
    );

    // Goal should be the last argument
    assert_eq!(
        command.last().unwrap(),
        "spring-boot:run",
        "spring-boot:run should be the last argument"
    );

    // Properties should come before the goal
    let goal_index = command.iter().position(|arg| arg == "spring-boot:run").unwrap();
    let profiles_index = command
        .iter()
        .position(|arg| arg.starts_with("-Drun.profiles="))
        .unwrap();
    let jvm_args_index = command
        .iter()
        .position(|arg| arg.starts_with("-Drun.jvmArguments="))
        .unwrap();

    assert!(
        profiles_index < goal_index,
        "Profiles should come before goal"
    );
    assert!(
        jvm_args_index < goal_index,
        "JVM args should come before goal"
    );
}

#[test]
fn test_spring_boot_1x_empty_profiles_and_jvm_args() {
    let command = build_launch_command(
        LaunchStrategy::SpringBootRun,
        None,
        &[], // No profiles
        &[], // No JVM args
        Some("jar"),
        Some("1.4.13"),
    );

    // Should not add empty properties
    assert!(
        !command.iter().any(|arg| arg.starts_with("-Drun.profiles=")),
        "Should not add empty profiles property"
    );

    assert!(
        !command.iter().any(|arg| arg.starts_with("-Drun.jvmArguments=")),
        "Should not add empty JVM arguments property"
    );

    // Should only have the goal
    assert_eq!(
        command.len(),
        1,
        "Should only contain the goal when no profiles/args"
    );
    assert_eq!(command[0], "spring-boot:run");
}

#[test]
fn test_spring_boot_1x_war_packaging() {
    // Test that WAR packaging works with Spring Boot 1.x
    let command = build_launch_command(
        LaunchStrategy::SpringBootRun,
        None,
        &[],
        &[],
        Some("war"), // WAR instead of JAR
        Some("1.4.13"),
    );

    // Should still use spring-boot:run for WAR
    assert!(
        command.iter().any(|arg| arg == "spring-boot:run"),
        "Spring Boot 1.x should support WAR packaging"
    );
}

#[test]
fn test_spring_boot_fixes_reported_user_issue() {
    // Direct regression test for the reported issue:
    // User had Spring Boot 1.4.13 and got "plugin JAR not found" error
    
    let command = build_launch_command(
        LaunchStrategy::SpringBootRun,
        None,
        &[],
        &[],
        Some("war"),
        Some("1.4.13"),
    );

    // The bug was generating this INVALID command:
    // org.springframework.boot:spring-boot-maven-plugin:1.4.13:run
    
    // Verify it's NOT generated
    assert!(
        !command.iter().any(|arg| 
            arg == "org.springframework.boot:spring-boot-maven-plugin:1.4.13:run"
        ),
        "Should NOT generate the buggy fully-qualified syntax"
    );

    // Verify correct command IS generated
    assert!(
        command.iter().any(|arg| arg == "spring-boot:run"),
        "Should generate correct 'spring-boot:run' command"
    );

    println!("✅ User-reported issue is fixed");
    println!("   Generated command: {:?}", command);
}
