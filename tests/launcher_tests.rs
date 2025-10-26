// Launch command building tests
use lazymvn::maven::{build_launch_command, quote_arg_for_platform, LaunchStrategy};

mod common;

#[test]
fn test_build_launch_command_spring_boot_run() {
    let profiles = vec!["dev".to_string(), "debug".to_string()];
    let jvm_args = vec!["-Dfoo=bar".to_string(), "-Xmx512m".to_string()];

    let command = build_launch_command(
        LaunchStrategy::SpringBootRun,
        None,
        &profiles,
        &jvm_args,
        None,
    );

    // Should contain profiles argument
    assert!(
        command
            .iter()
            .any(|arg| arg.contains("spring-boot.run.profiles=dev,debug")),
        "Should set profiles: {:?}",
        command
    );

    // Should contain JVM arguments
    assert!(
        command
            .iter()
            .any(|arg| arg.contains("spring-boot.run.jvmArguments")),
        "Should set jvmArguments: {:?}",
        command
    );

    // Should end with the goal
    assert_eq!(command.last(), Some(&"spring-boot:run".to_string()));
}

#[test]
fn test_build_launch_command_exec_java() {
    let jvm_args = vec!["-Dfoo=bar".to_string()];

    let command = build_launch_command(
        LaunchStrategy::ExecJava,
        Some("com.example.Application"),
        &[],
        &jvm_args,
        None,
    );

    // Should contain mainClass argument
    assert!(
        command
            .iter()
            .any(|arg| arg.contains("exec.mainClass=com.example.Application")),
        "Should set mainClass: {:?}",
        command
    );

    // Should contain JVM args
    assert!(
        command.contains(&quote_arg_for_platform("-Dfoo=bar")),
        "Should include JVM args: {:?}",
        command
    );

    // Should contain cleanup daemon threads flag
    assert!(
        command
            .iter()
            .any(|arg| arg.contains("exec.cleanupDaemonThreads=false")),
        "Should include cleanupDaemonThreads flag: {:?}",
        command
    );

    // Should end with the goal
    assert_eq!(command.last(), Some(&"exec:java".to_string()));
}

#[test]
fn test_build_launch_command_exec_java_without_main_class() {
    let command = build_launch_command(LaunchStrategy::ExecJava, None, &[], &[], None);

    // Should not contain mainClass if not provided
    assert!(
        !command.iter().any(|arg| arg.contains("exec.mainClass")),
        "Should not set mainClass if none provided: {:?}",
        command
    );

    // Should still have the goal
    assert_eq!(command.last(), Some(&"exec:java".to_string()));
}

#[test]
fn test_build_launch_command_exec_java_war_packaging() {
    // Test that WAR packaging adds classpathScope=compile
    let command = build_launch_command(
        LaunchStrategy::ExecJava,
        Some("com.example.WarApplication"),
        &[],
        &[],
        Some("war"),
    );

    // Should contain mainClass argument
    assert!(
        command
            .iter()
            .any(|arg| arg.contains("exec.mainClass=com.example.WarApplication")),
        "Should set mainClass: {:?}",
        command
    );

    // Should contain classpathScope=compile for WAR packaging
    assert!(
        command
            .iter()
            .any(|arg| arg.contains("exec.classpathScope=compile")),
        "Should include classpathScope=compile for WAR packaging: {:?}",
        command
    );

    // Should contain cleanup daemon threads flag
    assert!(
        command
            .iter()
            .any(|arg| arg.contains("exec.cleanupDaemonThreads=false")),
        "Should include cleanupDaemonThreads flag: {:?}",
        command
    );

    // Should end with the goal
    assert_eq!(command.last(), Some(&"exec:java".to_string()));
}

#[test]
fn test_build_launch_command_exec_java_jar_packaging() {
    // Test that JAR packaging does NOT add classpathScope=compile
    let command = build_launch_command(
        LaunchStrategy::ExecJava,
        Some("com.example.JarApplication"),
        &[],
        &[],
        Some("jar"),
    );

    // Should contain mainClass argument
    assert!(
        command
            .iter()
            .any(|arg| arg.contains("exec.mainClass=com.example.JarApplication")),
        "Should set mainClass: {:?}",
        command
    );

    // Should NOT contain classpathScope for JAR packaging
    assert!(
        !command
            .iter()
            .any(|arg| arg.contains("exec.classpathScope")),
        "Should NOT include classpathScope for JAR packaging: {:?}",
        command
    );

    // Should end with the goal
    assert_eq!(command.last(), Some(&"exec:java".to_string()));
}

#[test]
#[cfg(unix)] // Shell script execution not supported on Windows
fn test_command_display_in_output() {
    let _guard = common::test_lock().lock().unwrap();
    use lazymvn::maven::execute_maven_command;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let project_root = dir.path();

    let mvnw_path = project_root.join("mvnw");
    common::write_script(&mvnw_path, "#!/bin/sh\necho 'test output'\n");

    let profiles = vec!["dev".to_string()];
    let flags = vec!["--offline".to_string()];
    let output = execute_maven_command(
        project_root,
        Some("my-module"),
        &["clean", "install"],
        &profiles,
        None,
        &flags,
    )
    .unwrap();

    // First line should be the command
    assert!(
        output[0].starts_with("$ ./mvnw"),
        "First line should be the command: {}",
        output[0]
    );
    assert!(
        output[0].contains("-P dev"),
        "Command should include profiles: {}",
        output[0]
    );
    assert!(
        output[0].contains("-pl my-module"),
        "Command should include module: {}",
        output[0]
    );
    assert!(
        output[0].contains("--offline"),
        "Command should include flags: {}",
        output[0]
    );
    assert!(
        output[0].contains("clean install"),
        "Command should include goals: {}",
        output[0]
    );
}
