//! Spring Boot detection and launch strategy

use crate::core::config::LaunchMode;
use std::path::Path;

/// Information about a module's Spring Boot capabilities
#[derive(Debug, Clone)]
pub struct SpringBootDetection {
    pub has_spring_boot_plugin: bool,
    pub has_exec_plugin: bool,
    pub main_class: Option<String>,
    pub packaging: Option<String>,
}

impl SpringBootDetection {
    /// Check if spring-boot:run should work
    pub fn can_use_spring_boot_run(&self) -> bool {
        self.has_spring_boot_plugin
            && self
                .packaging
                .as_ref()
                .map(|p| p == "jar" || p == "war")
                .unwrap_or(true)
    }

    /// Check if this looks like a Spring Boot web application that should prefer spring-boot:run
    pub fn should_prefer_spring_boot_run(&self) -> bool {
        // For war packaging with Spring Boot plugin, prefer spring-boot:run
        // to avoid servlet classpath issues with exec:java
        self.has_spring_boot_plugin && self.packaging.as_ref().map(|p| p == "war").unwrap_or(false)
    }

    /// Check if exec:java can be used as fallback
    pub fn can_use_exec_java(&self) -> bool {
        self.has_exec_plugin || self.main_class.is_some()
    }
}

/// Launch strategy for running applications
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchStrategy {
    SpringBootRun,
    ExecJava,
    #[allow(dead_code)]
    VSCodeJava, // Use VS Code Java extension to launch
}

/// Decide which launch strategy to use
pub fn decide_launch_strategy(
    detection: &SpringBootDetection,
    launch_mode: LaunchMode,
) -> LaunchStrategy {
    match launch_mode {
        LaunchMode::ForceRun => LaunchStrategy::SpringBootRun,
        LaunchMode::ForceExec => LaunchStrategy::ExecJava,
        LaunchMode::Auto => {
            if detection.should_prefer_spring_boot_run() {
                log::info!(
                    "Auto mode: Spring Boot web app detected (war packaging), strongly preferring spring-boot:run"
                );
                LaunchStrategy::SpringBootRun
            } else if detection.can_use_spring_boot_run() {
                log::info!("Auto mode: Spring Boot plugin detected, using spring-boot:run");
                LaunchStrategy::SpringBootRun
            } else if detection.can_use_exec_java() {
                log::info!(
                    "Auto mode: No Spring Boot plugin or incompatible packaging, using exec:java"
                );
                LaunchStrategy::ExecJava
            } else {
                log::warn!(
                    "Auto mode: No viable launch strategy detected, defaulting to spring-boot:run"
                );
                LaunchStrategy::SpringBootRun
            }
        }
    }
}

/// Build launch command based on detection and strategy
pub fn build_launch_command(
    strategy: LaunchStrategy,
    main_class: Option<&str>,
    profiles: &[String],
    jvm_args: &[String],
    packaging: Option<&str>,
) -> Vec<String> {
    let mut command_parts = Vec::new();

    match strategy {
        LaunchStrategy::SpringBootRun => {
            // Build spring-boot:run command with parameters
            if !profiles.is_empty() {
                // Pass profiles as spring-boot.run.profiles
                let profiles_arg = format!("-Dspring-boot.run.profiles={}", profiles.join(","));
                command_parts.push(quote_arg_for_platform(&profiles_arg));
            }

            if !jvm_args.is_empty() {
                // Pass JVM args as spring-boot.run.jvmArguments
                let jvm_args_str = jvm_args.join(" ");
                let jvm_arg = format!("-Dspring-boot.run.jvmArguments={}", jvm_args_str);
                command_parts.push(quote_arg_for_platform(&jvm_arg));
            }

            command_parts.push("spring-boot:run".to_string());

            log::info!(
                "Built spring-boot:run command with {} profile(s) and {} JVM arg(s)",
                profiles.len(),
                jvm_args.len()
            );
        }
        LaunchStrategy::ExecJava => {
            // Build exec:java command with mainClass
            if let Some(mc) = main_class {
                let main_class_arg = format!("-Dexec.mainClass={}", mc);
                command_parts.push(quote_arg_for_platform(&main_class_arg));
            }

            // For WAR packaging, use compile scope to include provided dependencies (servlet-api, etc.)
            // This fixes javax.servlet.Filter NoClassDefFoundError issues
            if packaging == Some("war") {
                command_parts.push(quote_arg_for_platform("-Dexec.classpathScope=compile"));
                log::info!(
                    "WAR packaging detected: adding -Dexec.classpathScope=compile to include provided dependencies"
                );
            }

            // Add cleanup daemon threads flag for better shutdown behavior
            command_parts.push(quote_arg_for_platform("-Dexec.cleanupDaemonThreads=false"));

            // Add JVM args as system properties
            for arg in jvm_args {
                command_parts.push(quote_arg_for_platform(arg));
            }

            command_parts.push("exec:java".to_string());

            log::info!(
                "Built exec:java command with mainClass={:?}, packaging={:?}, and {} JVM arg(s)",
                main_class,
                packaging,
                jvm_args.len()
            );
        }
        LaunchStrategy::VSCodeJava => {
            // This is a placeholder - actual VS Code integration would be different
            command_parts.push("# VS Code Java launch not implemented yet".to_string());
            log::info!("VS Code Java launch strategy selected (not implemented)");
        }
    }

    command_parts
}

/// Detect Spring Boot capabilities for a module
pub fn detect_spring_boot_capabilities(
    project_root: &Path,
    module: Option<&str>,
) -> Result<SpringBootDetection, std::io::Error> {
    log::debug!(
        "Detecting Spring Boot capabilities for module: {:?}",
        module
    );

    let config = crate::core::config::load_config(project_root);

    // Get effective POM for the module
    let args = vec!["help:effective-pom"];

    let output = super::command::execute_maven_command(
        project_root,
        module,
        &args,
        &[],
        config.maven_settings.as_deref(),
        &[],
    )?;

    let pom_content = output.join("\n");

    let mut detection = SpringBootDetection {
        has_spring_boot_plugin: false,
        has_exec_plugin: false,
        main_class: None,
        packaging: None,
    };

    // Parse the effective POM
    let mut in_plugins = false;
    let mut in_plugin = false;
    let mut current_plugin_artifact_id = String::new();
    let mut in_configuration = false;

    for line in pom_content.lines() {
        let trimmed = line.trim();

        // Detect packaging
        if trimmed.starts_with("<packaging>")
            && trimmed.contains("</packaging>")
            && let Some(start) = trimmed.find("<packaging>")
            && let Some(end) = trimmed.find("</packaging>")
        {
            let packaging = &trimmed[start + 11..end];
            detection.packaging = Some(packaging.to_string());
            log::debug!("Found packaging: {}", packaging);
        }

        // Track plugin sections
        if trimmed.starts_with("<plugins>") {
            in_plugins = true;
        } else if trimmed.starts_with("</plugins>") {
            in_plugins = false;
        }

        if in_plugins {
            if trimmed.starts_with("<plugin>") {
                in_plugin = true;
                current_plugin_artifact_id.clear();
            } else if trimmed.starts_with("</plugin>") {
                in_plugin = false;
                in_configuration = false;
            }

            if in_plugin {
                // Check for Spring Boot plugin
                if trimmed.starts_with("<artifactId>spring-boot-maven-plugin</artifactId>") {
                    detection.has_spring_boot_plugin = true;
                    current_plugin_artifact_id = "spring-boot-maven-plugin".to_string();
                    log::debug!("Found spring-boot-maven-plugin");
                }

                // Check for exec plugin
                if trimmed.starts_with("<artifactId>exec-maven-plugin</artifactId>") {
                    detection.has_exec_plugin = true;
                    current_plugin_artifact_id = "exec-maven-plugin".to_string();
                    log::debug!("Found exec-maven-plugin");
                }

                // Track configuration section
                if trimmed.starts_with("<configuration>") {
                    in_configuration = true;
                } else if trimmed.starts_with("</configuration>") {
                    in_configuration = false;
                }

                // Extract mainClass from configuration
                if in_configuration
                    && (trimmed.starts_with("<mainClass>") || trimmed.starts_with("<main-class>"))
                    && (trimmed.contains("</mainClass>") || trimmed.contains("</main-class>"))
                {
                    let main_class = if trimmed.contains("</mainClass>") {
                        extract_tag_content(trimmed, "mainClass")
                    } else {
                        extract_tag_content(trimmed, "main-class")
                    };

                    if let Some(mc) = main_class {
                        detection.main_class = Some(mc.clone());
                        log::debug!("Found mainClass '{}' in {}", mc, current_plugin_artifact_id);
                    }
                }
            }
        }

        // Also check for properties (spring-boot.run.mainClass, start-class, etc.)
        if trimmed.starts_with("<spring-boot.run.mainClass>")
            || trimmed.starts_with("<spring-boot.main-class>")
            || trimmed.starts_with("<start-class>")
        {
            let property_name = if trimmed.contains("spring-boot.run.mainClass") {
                "spring-boot.run.mainClass"
            } else if trimmed.contains("spring-boot.main-class") {
                "spring-boot.main-class"
            } else {
                "start-class"
            };

            if let Some(mc) = extract_tag_content(trimmed, property_name)
                && detection.main_class.is_none()
            {
                detection.main_class = Some(mc.clone());
                log::debug!("Found mainClass '{}' from property {}", mc, property_name);
            }
        }
    }

    log::info!(
        "Spring Boot detection results: plugin={}, exec={}, mainClass={:?}, packaging={:?}",
        detection.has_spring_boot_plugin,
        detection.has_exec_plugin,
        detection.main_class,
        detection.packaging
    );

    Ok(detection)
}

/// Quote arguments appropriately for the platform (especially PowerShell on Windows)
pub fn quote_arg_for_platform(arg: &str) -> String {
    #[cfg(windows)]
    {
        // On Windows (PowerShell), quote -D arguments
        if arg.starts_with("-D") {
            format!("\"{}\"", arg)
        } else {
            arg.to_string()
        }
    }
    #[cfg(not(windows))]
    {
        arg.to_string()
    }
}

/// Extract content from an XML tag
pub fn extract_tag_content(line: &str, tag_name: &str) -> Option<String> {
    let open_tag = format!("<{}>", tag_name);
    let close_tag = format!("</{}>", tag_name);

    if let Some(start) = line.find(&open_tag)
        && let Some(end) = line.find(&close_tag)
    {
        let content = &line[start + open_tag.len()..end];
        return Some(content.trim().to_string());
    }
    None
}
