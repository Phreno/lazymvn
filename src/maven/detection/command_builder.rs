//! Maven command building for launch strategies

#![allow(dead_code)]

use super::strategy::LaunchStrategy;

/// Build launch command based on detection and strategy
pub fn build_launch_command(
    strategy: LaunchStrategy,
    main_class: Option<&str>,
    profiles: &[String],
    jvm_args: &[String],
    packaging: Option<&str>,
    spring_boot_version: Option<&str>,
) -> Vec<String> {
    let mut command_parts = Vec::new();

    match strategy {
        LaunchStrategy::SpringBootRun => {
            build_spring_boot_command(&mut command_parts, profiles, jvm_args, spring_boot_version);
        }
        LaunchStrategy::ExecJava => {
            build_exec_java_command(&mut command_parts, main_class, jvm_args, packaging);
        }
        LaunchStrategy::VSCodeJava => {
            command_parts.push("# VS Code Java launch not implemented yet".to_string());
            log::info!("VS Code Java launch strategy selected (not implemented)");
        }
    }

    command_parts
}

fn build_spring_boot_command(
    command_parts: &mut Vec<String>,
    profiles: &[String],
    jvm_args: &[String],
    spring_boot_version: Option<&str>,
) {
    let is_spring_boot_1x = spring_boot_version
        .map(|v| v.starts_with("1."))
        .unwrap_or(false);

    let (profiles_property, jvm_args_property) = if is_spring_boot_1x {
        ("run.profiles", "run.jvmArguments")
    } else {
        ("spring-boot.run.profiles", "spring-boot.run.jvmArguments")
    };

    if !profiles.is_empty() {
        let profiles_arg = format!("-D{}={}", profiles_property, profiles.join(","));
        command_parts.push(profiles_arg);
    }

    if !jvm_args.is_empty() {
        let jvm_args_str = jvm_args.join(" ");
        let jvm_arg = format!("-D{}={}", jvm_args_property, jvm_args_str);
        command_parts.push(jvm_arg);
    }

    let goal = if is_spring_boot_1x && spring_boot_version.is_some() {
        format!(
            "org.springframework.boot:spring-boot-maven-plugin:{}:run",
            spring_boot_version.unwrap()
        )
    } else {
        "spring-boot:run".to_string()
    };

    command_parts.push(goal.clone());

    log::info!(
        "Built {} command (v{}) with {} profile(s) and {} JVM arg(s)",
        goal,
        spring_boot_version.unwrap_or("unknown"),
        profiles.len(),
        jvm_args.len()
    );
}

fn build_exec_java_command(
    command_parts: &mut Vec<String>,
    main_class: Option<&str>,
    jvm_args: &[String],
    packaging: Option<&str>,
) {
    if let Some(mc) = main_class {
        let main_class_arg = format!("-Dexec.mainClass={}", mc);
        command_parts.push(main_class_arg);
    }

    if packaging == Some("war") {
        command_parts.push("-Dexec.classpathScope=compile".to_string());
        log::info!(
            "WAR packaging detected: adding -Dexec.classpathScope=compile to include provided dependencies"
        );
    }

    command_parts.push("-Dexec.cleanupDaemonThreads=false".to_string());

    if !jvm_args.is_empty() {
        let exec_args = jvm_args.join(" ");
        command_parts.push(format!("-Dexec.args={}", exec_args));
        log::debug!("Adding JVM args via -Dexec.args: {}", exec_args);
    }

    command_parts.push("exec:java".to_string());

    log::info!(
        "Built exec:java command with mainClass={:?}, packaging={:?}, and {} JVM arg(s)",
        main_class,
        packaging,
        jvm_args.len()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_launch_command_spring_boot_basic() {
        let cmd = build_launch_command(
            LaunchStrategy::SpringBootRun,
            None,
            &[],
            &[],
            Some("jar"),
            Some("2.5.0"),
        );
        assert_eq!(cmd, vec!["spring-boot:run"]);
    }

    #[test]
    fn test_build_launch_command_spring_boot_with_profiles() {
        let profiles = vec!["dev".to_string(), "test".to_string()];
        let cmd = build_launch_command(
            LaunchStrategy::SpringBootRun,
            None,
            &profiles,
            &[],
            Some("jar"),
            Some("2.5.0"),
        );
        assert!(
            cmd.iter()
                .any(|arg| arg.contains("spring-boot.run.profiles=dev,test"))
        );
        assert_eq!(cmd.last().unwrap(), "spring-boot:run");
    }

    #[test]
    fn test_build_launch_command_spring_boot_with_jvm_args() {
        let jvm_args = vec!["-Xmx512m".to_string(), "-Ddebug=true".to_string()];
        let cmd = build_launch_command(
            LaunchStrategy::SpringBootRun,
            None,
            &[],
            &jvm_args,
            Some("jar"),
            Some("2.5.0"),
        );
        assert!(
            cmd.iter()
                .any(|arg| arg.contains("spring-boot.run.jvmArguments"))
        );
        assert!(cmd.iter().any(|arg| arg.contains("-Xmx512m")));
        assert!(cmd.iter().any(|arg| arg.contains("-Ddebug=true")));
    }

    #[test]
    fn test_build_launch_command_spring_boot_complete() {
        let profiles = vec!["dev".to_string()];
        let jvm_args = vec!["-Xmx512m".to_string(), "-Ddebug=true".to_string()];
        let cmd = build_launch_command(
            LaunchStrategy::SpringBootRun,
            None,
            &profiles,
            &jvm_args,
            Some("jar"),
            Some("2.5.0"),
        );
        assert!(
            cmd.iter()
                .any(|arg| arg.contains("spring-boot.run.profiles=dev"))
        );
        assert!(cmd.iter().any(|arg| arg.contains("-Xmx512m")));
        assert!(cmd.iter().any(|arg| arg.contains("-Ddebug=true")));
        assert_eq!(cmd.last().unwrap(), "spring-boot:run");
    }

    #[test]
    fn test_build_launch_command_spring_boot_1x_uses_run_properties() {
        let profiles = vec!["dev".to_string()];
        let jvm_args = vec!["-Xmx512m".to_string(), "-Ddebug=true".to_string()];
        let cmd = build_launch_command(
            LaunchStrategy::SpringBootRun,
            None,
            &profiles,
            &jvm_args,
            Some("jar"),
            Some("1.2.2.RELEASE"),
        );
        assert!(
            cmd.iter().any(|arg| arg.contains("run.profiles=dev")),
            "Expected -Drun.profiles for Spring Boot 1.x"
        );
        assert!(
            cmd.iter().any(|arg| arg.contains("run.jvmArguments")),
            "Expected -Drun.jvmArguments for Spring Boot 1.x"
        );
        assert!(
            !cmd.iter().any(|arg| arg.contains("spring-boot.run.")),
            "Should not use spring-boot.run.* properties for Spring Boot 1.x"
        );
        assert!(cmd.iter().any(|arg| arg.contains("-Xmx512m")));
        assert!(cmd.iter().any(|arg| arg.contains("-Ddebug=true")));
        assert_eq!(
            cmd.last().unwrap(),
            "org.springframework.boot:spring-boot-maven-plugin:1.2.2.RELEASE:run"
        );
    }

    #[test]
    fn test_build_launch_command_exec_java_basic() {
        let cmd = build_launch_command(
            LaunchStrategy::ExecJava,
            Some("com.example.Main"),
            &[],
            &[],
            Some("jar"),
            None,
        );
        assert_eq!(cmd.len(), 3);
        assert!(
            cmd.iter()
                .any(|arg| arg.contains("exec.mainClass=com.example.Main"))
        );
        assert_eq!(cmd.last().unwrap(), "exec:java");
    }

    #[test]
    fn test_build_launch_command_exec_java_with_jvm_args() {
        let jvm_args = vec![
            "-Xmx1g".to_string(),
            "-javaagent:/path/to/agent.jar".to_string(),
            "-Dlog4j.configuration=file:///config.properties".to_string(),
        ];
        let cmd = build_launch_command(
            LaunchStrategy::ExecJava,
            Some("com.example.Main"),
            &[],
            &jvm_args,
            Some("jar"),
            None,
        );

        let exec_args_param = cmd.iter().find(|arg| arg.starts_with("-Dexec.args="));
        assert!(
            exec_args_param.is_some(),
            "Should have -Dexec.args parameter containing JVM arguments"
        );

        let exec_args_value = exec_args_param.unwrap();
        assert!(exec_args_value.contains("-Xmx1g"));
        assert!(exec_args_value.contains("-javaagent:/path/to/agent.jar"));
        assert!(exec_args_value.contains("-Dlog4j.configuration="));

        assert!(
            !cmd.iter().any(|arg| arg == "-Xmx1g"),
            "-Xmx1g should not be a separate Maven argument"
        );
        assert!(
            !cmd.iter().any(|arg| arg == "-javaagent:/path/to/agent.jar"),
            "javaagent should not be a separate Maven argument"
        );

        assert!(
            cmd.iter()
                .any(|arg| arg.contains("exec.mainClass=com.example.Main"))
        );
    }

    #[test]
    fn test_build_launch_command_exec_java_no_main_class() {
        let cmd = build_launch_command(LaunchStrategy::ExecJava, None, &[], &[], Some("jar"), None);
        assert!(cmd.contains(&"exec:java".to_string()));
    }

    #[test]
    fn test_build_launch_command_spring_boot_run_with_jvm_args() {
        let jvm_args = vec![
            "-javaagent:/path/to/agent.jar".to_string(),
            "-Xmx512m".to_string(),
        ];

        let command = build_launch_command(
            LaunchStrategy::SpringBootRun,
            Some("com.example.Main"),
            &[],
            &jvm_args,
            None,
            Some("2.7.0"),
        );

        let jvm_args_param = command
            .iter()
            .find(|arg| arg.starts_with("-Dspring-boot.run.jvmArguments="));
        assert!(
            jvm_args_param.is_some(),
            "Should have -Dspring-boot.run.jvmArguments parameter"
        );

        let jvm_args_value = jvm_args_param.unwrap();
        assert!(jvm_args_value.contains("-javaagent:/path/to/agent.jar"));
        assert!(jvm_args_value.contains("-Xmx512m"));

        assert!(command.contains(&"spring-boot:run".to_string()));
    }

    #[test]
    fn test_build_launch_command_spring_boot_1x_with_jvm_args() {
        let jvm_args = vec!["-Xmx512m".to_string()];

        let command = build_launch_command(
            LaunchStrategy::SpringBootRun,
            Some("com.example.Main"),
            &[],
            &jvm_args,
            None,
            Some("1.5.10.RELEASE"),
        );

        let jvm_args_param = command
            .iter()
            .find(|arg| arg.starts_with("-Drun.jvmArguments="));
        assert!(
            jvm_args_param.is_some(),
            "Should have -Drun.jvmArguments parameter for Spring Boot 1.x"
        );

        assert!(command.iter().any(|arg| arg.contains("org.springframework.boot:spring-boot-maven-plugin:1.5.10.RELEASE:run")));
    }
}
