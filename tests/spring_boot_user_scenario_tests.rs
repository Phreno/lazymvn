//! End-to-end regression test for the Spring Boot 1.4.13 bug
//!
//! This test simulates the exact user scenario from the bug report:
//! - Spring Boot 1.4.13 project
//! - WAR packaging
//! - Maven settings file
//! - Plugin resolution issue
//!
//! Bug Report: 2025-11-03 21:18:21
//! Error: "Could not find artifact org.springframework.boot:spring-boot-maven-plugin:jar:1.4.13"

use lazymvn::maven::{build_launch_command, LaunchStrategy};

#[test]
fn test_exact_user_scenario_spring_boot_1_4_13_war() {
    // Simulate user's exact configuration from bug report
    let version = "1.4.13";
    let packaging = Some("war");
    let profiles = vec![];
    let jvm_args = vec![];

    let command = build_launch_command(
        LaunchStrategy::SpringBootRun,
        None,
        &profiles,
        &jvm_args,
        packaging,
        Some(version),
    );

    // CRITICAL: Must NOT generate the buggy syntax
    let buggy_syntax = format!(
        "org.springframework.boot:spring-boot-maven-plugin:{}:run",
        version
    );

    assert!(
        !command.contains(&buggy_syntax),
        "REGRESSION: Buggy syntax detected! This would cause:\n\
         [ERROR] Could not find artifact org.springframework.boot:spring-boot-maven-plugin:jar:{}\n\
         Command generated: {:?}",
        version,
        command
    );

    // MUST generate the correct syntax
    assert!(
        command.contains(&"spring-boot:run".to_string()),
        "Correct syntax not found. Command: {:?}",
        command
    );

    println!("✅ User scenario test passed");
    println!("   Version: {}", version);
    println!("   Packaging: {:?}", packaging);
    println!("   Command: {:?}", command);
}

#[test]
fn test_user_scenario_with_jvm_args() {
    // User scenario with JVM arguments (from debug report)
    let version = "1.4.13";
    let jvm_args = vec![
        "-Dlog4j.ignoreTCL=true".to_string(),
        "-Dlog4j.defaultInitOverride=true".to_string(),
        "-Xmx512m".to_string(),
    ];

    let command = build_launch_command(
        LaunchStrategy::SpringBootRun,
        None,
        &[],
        &jvm_args,
        Some("war"),
        Some(version),
    );

    // Verify JVM args are passed correctly with 1.x property name
    assert!(
        command.iter().any(|arg| arg.starts_with("-Drun.jvmArguments=")),
        "Spring Boot 1.x should use -Drun.jvmArguments for JVM args"
    );

    // Verify all JVM args are included
    let jvm_args_param = command
        .iter()
        .find(|arg| arg.starts_with("-Drun.jvmArguments="))
        .expect("JVM arguments property not found");

    for arg in &jvm_args {
        assert!(
            jvm_args_param.contains(arg),
            "JVM arg {} not found in: {}",
            arg,
            jvm_args_param
        );
    }

    // Verify correct goal
    assert!(
        command.contains(&"spring-boot:run".to_string()),
        "Command: {:?}",
        command
    );

    println!("✅ User scenario with JVM args test passed");
}

#[test]
fn test_user_scenario_with_profiles() {
    // User scenario with profiles
    let version = "1.4.13";
    let profiles = vec!["dev".to_string()];

    let command = build_launch_command(
        LaunchStrategy::SpringBootRun,
        None,
        &profiles,
        &[],
        Some("war"),
        Some(version),
    );

    // Verify profiles are passed correctly with 1.x property name
    assert!(
        command.iter().any(|arg| arg.starts_with("-Drun.profiles=")),
        "Spring Boot 1.x should use -Drun.profiles for profiles"
    );

    // Verify profile is included
    let profiles_param = command
        .iter()
        .find(|arg| arg.starts_with("-Drun.profiles="))
        .expect("Profiles property not found");

    assert!(
        profiles_param.contains("dev"),
        "Profile 'dev' not found in: {}",
        profiles_param
    );

    println!("✅ User scenario with profiles test passed");
}

#[test]
fn test_all_spring_boot_1x_versions_from_bug_reports() {
    // Test all Spring Boot 1.x versions that could have this bug
    let affected_versions = vec![
        "1.0.0.RELEASE",
        "1.0.1.RELEASE",
        "1.0.2.RELEASE",
        "1.1.0.RELEASE",
        "1.1.12.RELEASE",
        "1.2.0.RELEASE",
        "1.2.8.RELEASE",
        "1.3.0.RELEASE",
        "1.3.8.RELEASE",
        "1.4.0.RELEASE",
        "1.4.7.RELEASE",
        "1.4.13", // User's version
        "1.5.0.RELEASE",
        "1.5.10.RELEASE",
        "1.5.22.RELEASE", // Last 1.x version
    ];

    let version_count = affected_versions.len();

    for version in affected_versions {
        let command = build_launch_command(
            LaunchStrategy::SpringBootRun,
            None,
            &[],
            &[],
            Some("jar"),
            Some(version),
        );

        // None should use the buggy syntax
        let buggy_syntax = format!(
            "org.springframework.boot:spring-boot-maven-plugin:{}:run",
            version
        );

        assert!(
            !command.contains(&buggy_syntax),
            "Version {} FAILED: Uses buggy syntax",
            version
        );

        // All should use the correct syntax
        assert!(
            command.contains(&"spring-boot:run".to_string()),
            "Version {} FAILED: Missing correct syntax",
            version
        );
    }

    println!("✅ All {} Spring Boot 1.x versions tested successfully", 
        version_count);
}

#[test]
fn test_maven_error_message_not_generated() {
    // Verify we don't generate a command that would produce the user's error
    let version = "1.4.13";

    let command = build_launch_command(
        LaunchStrategy::SpringBootRun,
        None,
        &[],
        &[],
        Some("war"),
        Some(version),
    );

    // The error message Maven would generate with buggy syntax:
    // "[ERROR] Could not find artifact org.springframework.boot:spring-boot-maven-plugin:jar:1.4.13"
    //
    // This happens because Maven tries to download the plugin as a JAR dependency
    // when the syntax "groupId:artifactId:version:goal" is used.
    //
    // Our command should NOT cause this error.

    // Check that we're not creating a command that would trigger this
    let would_cause_error = command.iter().any(|arg| {
        arg.contains("spring-boot-maven-plugin") && arg.contains(version) && arg.contains(":run")
    });

    assert!(
        !would_cause_error,
        "Command would cause plugin JAR resolution error: {:?}",
        command
    );

    println!("✅ Command will not trigger plugin JAR error");
}

#[test]
fn test_comparison_1x_vs_2x_both_work() {
    // Demonstrate that both 1.x and 2.x now use the same goal
    // (they differ only in property names)

    let command_1x = build_launch_command(
        LaunchStrategy::SpringBootRun,
        None,
        &[],
        &[],
        Some("jar"),
        Some("1.4.13"),
    );

    let command_2x = build_launch_command(
        LaunchStrategy::SpringBootRun,
        None,
        &[],
        &[],
        Some("jar"),
        Some("2.7.0"),
    );

    // Both should use the same goal
    assert!(
        command_1x.contains(&"spring-boot:run".to_string()),
        "1.x: {:?}",
        command_1x
    );
    assert!(
        command_2x.contains(&"spring-boot:run".to_string()),
        "2.x: {:?}",
        command_2x
    );

    // The goal should be identical
    let goal_1x = command_1x.last().unwrap();
    let goal_2x = command_2x.last().unwrap();

    assert_eq!(
        goal_1x, goal_2x,
        "Goals should be identical between 1.x and 2.x"
    );

    println!("✅ Both 1.x and 2.x use the same goal: {}", goal_1x);
}

#[test]
fn test_fix_backward_compatibility() {
    // Verify the fix doesn't break any existing functionality

    // Test various scenarios that should still work
    let scenarios = vec![
        ("2.7.0", vec!["prod"], vec!["-Xmx1g"]),
        ("3.0.0", vec!["test"], vec!["-Xmx512m"]),
        ("1.5.22.RELEASE", vec!["dev"], vec!["-Xmx256m"]),
    ];

    let scenario_count = scenarios.len();

    for (version, profiles, jvm_args) in scenarios {
        let profiles_str: Vec<String> = profiles.iter().map(|s| s.to_string()).collect();
        let jvm_args_str: Vec<String> = jvm_args.iter().map(|s| s.to_string()).collect();

        let command = build_launch_command(
            LaunchStrategy::SpringBootRun,
            None,
            &profiles_str,
            &jvm_args_str,
            Some("jar"),
            Some(version),
        );

        // Should have the goal
        assert!(
            command.contains(&"spring-boot:run".to_string()),
            "Version {}: {:?}",
            version,
            command
        );

        // Should have profile property (1.x or 2.x)
        let has_profiles = command.iter().any(|arg| {
            arg.contains("profiles=") && arg.contains(profiles[0])
        });
        assert!(
            has_profiles,
            "Version {} missing profiles: {:?}",
            version,
            command
        );

        // Should have JVM args property (1.x or 2.x)
        let has_jvm_args = command.iter().any(|arg| {
            arg.contains("jvmArguments=") && arg.contains(jvm_args[0])
        });
        assert!(
            has_jvm_args,
            "Version {} missing JVM args: {:?}",
            version,
            command
        );
    }

    println!("✅ Backward compatibility verified for {} scenarios", scenario_count);
}
