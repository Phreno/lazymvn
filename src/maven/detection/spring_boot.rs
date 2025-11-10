//! Spring Boot detection types and capabilities

#![allow(dead_code)]

use std::path::Path;

/// Information about a module's Spring Boot capabilities
#[derive(Debug, Clone)]
pub struct SpringBootDetection {
    pub has_spring_boot_plugin: bool,
    pub has_exec_plugin: bool,
    pub main_class: Option<String>,
    pub packaging: Option<String>,
    pub spring_boot_version: Option<String>,
}

impl SpringBootDetection {
    /// Check if spring-boot:run should work
    pub fn can_use_spring_boot_run(&self) -> bool {
        self.has_spring_boot_plugin && has_compatible_packaging(&self.packaging)
    }

    /// Check if this looks like a Spring Boot web application that should prefer spring-boot:run
    pub fn should_prefer_spring_boot_run(&self) -> bool {
        self.has_spring_boot_plugin && is_war_packaging(&self.packaging)
    }

    /// Check if exec:java can be used as fallback
    pub fn can_use_exec_java(&self) -> bool {
        self.has_exec_plugin || self.main_class.is_some()
    }
}

/// Check if packaging is compatible with spring-boot:run
fn has_compatible_packaging(packaging: &Option<String>) -> bool {
    packaging
        .as_ref()
        .map(|p| p == "jar" || p == "war")
        .unwrap_or(true)
}

/// Check if packaging is war
fn is_war_packaging(packaging: &Option<String>) -> bool {
    packaging.as_ref().map(|p| p == "war").unwrap_or(false)
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

    let pom_content = get_effective_pom(project_root, module)?;
    let detection = parse_spring_boot_detection(&pom_content);
    
    log_detection_results(&detection);

    Ok(detection)
}

/// Get effective POM content
fn get_effective_pom(project_root: &Path, module: Option<&str>) -> Result<String, std::io::Error> {
    let config = crate::core::config::load_config(project_root);
    let args = vec!["help:effective-pom"];

    let output = crate::maven::command::execute_maven_command(
        project_root,
        module,
        &args,
        &[],
        config.maven_settings.as_deref(),
        &[],
    )?;

    Ok(output.join("\n"))
}

/// Parse Spring Boot detection from POM content
fn parse_spring_boot_detection(pom_content: &str) -> SpringBootDetection {
    let mut detection = SpringBootDetection {
        has_spring_boot_plugin: false,
        has_exec_plugin: false,
        main_class: None,
        packaging: None,
        spring_boot_version: None,
    };

    parse_effective_pom(pom_content, &mut detection);
    detection
}

/// Log detection results
fn log_detection_results(detection: &SpringBootDetection) {
    log::info!(
        "Spring Boot detection results: plugin={}, exec={}, mainClass={:?}, packaging={:?}",
        detection.has_spring_boot_plugin,
        detection.has_exec_plugin,
        detection.main_class,
        detection.packaging
    );
}

fn parse_effective_pom(pom_content: &str, detection: &mut SpringBootDetection) {
    let mut in_plugins = false;
    let mut in_plugin = false;
    let mut current_plugin_artifact_id = String::new();
    let mut in_configuration = false;

    for line in pom_content.lines() {
        let trimmed = line.trim();

        detect_packaging(trimmed, detection);

        if trimmed.starts_with("<plugins>") {
            in_plugins = true;
        } else if trimmed.starts_with("</plugins>") {
            in_plugins = false;
        }

        if in_plugins {
            track_plugin_state(trimmed, &mut in_plugin, &mut in_configuration);

            if in_plugin {
                detect_plugins(
                    trimmed,
                    &mut current_plugin_artifact_id,
                    &mut in_configuration,
                    detection,
                );
            }
        }

        detect_main_class_properties(trimmed, detection);
    }
}

fn detect_packaging(line: &str, detection: &mut SpringBootDetection) {
    if is_packaging_line(line) && let Some(packaging) = extract_packaging_value(line) {
        detection.packaging = Some(packaging.to_string());
        log::debug!("Found packaging: {}", packaging);
    }
}

/// Check if line contains packaging tag
fn is_packaging_line(line: &str) -> bool {
    line.starts_with("<packaging>") && line.contains("</packaging>")
}

/// Extract packaging value from line
fn extract_packaging_value(line: &str) -> Option<&str> {
    let start = line.find("<packaging>")?;
    let end = line.find("</packaging>")?;
    Some(&line[start + 11..end])
}

fn track_plugin_state(line: &str, in_plugin: &mut bool, in_configuration: &mut bool) {
    if is_plugin_start(line) {
        *in_plugin = true;
    } else if is_plugin_end(line) {
        *in_plugin = false;
        *in_configuration = false;
    }
}

/// Check if line is plugin start tag
fn is_plugin_start(line: &str) -> bool {
    line.starts_with("<plugin>")
}

/// Check if line is plugin end tag
fn is_plugin_end(line: &str) -> bool {
    line.starts_with("</plugin>")
}

fn detect_plugins(
    line: &str,
    current_plugin_artifact_id: &mut String,
    in_configuration: &mut bool,
    detection: &mut SpringBootDetection,
) {
    detect_spring_boot_plugin(line, current_plugin_artifact_id, detection);
    detect_exec_plugin(line, current_plugin_artifact_id, detection);
    track_configuration_state(line, in_configuration);
    detect_main_class_in_config(line, in_configuration, current_plugin_artifact_id, detection);
}

/// Detect Spring Boot Maven plugin
fn detect_spring_boot_plugin(
    line: &str,
    current_plugin_artifact_id: &mut String,
    detection: &mut SpringBootDetection,
) {
    if line.starts_with("<artifactId>spring-boot-maven-plugin</artifactId>") {
        detection.has_spring_boot_plugin = true;
        *current_plugin_artifact_id = "spring-boot-maven-plugin".to_string();
        log::debug!("Found spring-boot-maven-plugin");
    }

    if *current_plugin_artifact_id == "spring-boot-maven-plugin" {
        detect_spring_boot_version(line, detection);
    }
}

/// Detect Spring Boot plugin version
fn detect_spring_boot_version(line: &str, detection: &mut SpringBootDetection) {
    if line.starts_with("<version>") && line.contains("</version>")
        && let Some(version) = super::xml_parser::extract_tag_content(line, "version")
    {
        detection.spring_boot_version = Some(version.clone());
        log::debug!("Found Spring Boot plugin version: {}", version);
    }
}

/// Detect exec Maven plugin
fn detect_exec_plugin(
    line: &str,
    current_plugin_artifact_id: &mut String,
    detection: &mut SpringBootDetection,
) {
    if line.starts_with("<artifactId>exec-maven-plugin</artifactId>") {
        detection.has_exec_plugin = true;
        *current_plugin_artifact_id = "exec-maven-plugin".to_string();
        log::debug!("Found exec-maven-plugin");
    }
}

/// Track configuration section state
fn track_configuration_state(line: &str, in_configuration: &mut bool) {
    if line.starts_with("<configuration>") {
        *in_configuration = true;
    } else if line.starts_with("</configuration>") {
        *in_configuration = false;
    }
}

/// Detect main class in configuration
fn detect_main_class_in_config(
    line: &str,
    in_configuration: &bool,
    current_plugin_artifact_id: &str,
    detection: &mut SpringBootDetection,
) {
    if !*in_configuration {
        return;
    }

    if is_main_class_line(line) && let Some(mc) = extract_main_class(line) {
        detection.main_class = Some(mc.clone());
        log::debug!("Found mainClass '{}' in {}", mc, current_plugin_artifact_id);
    }
}

/// Check if line contains main class tag
fn is_main_class_line(line: &str) -> bool {
    (line.starts_with("<mainClass>") || line.starts_with("<main-class>"))
        && (line.contains("</mainClass>") || line.contains("</main-class>"))
}

/// Extract main class value from line
fn extract_main_class(line: &str) -> Option<String> {
    if line.contains("</mainClass>") {
        super::xml_parser::extract_tag_content(line, "mainClass")
    } else {
        super::xml_parser::extract_tag_content(line, "main-class")
    }
}

fn detect_main_class_properties(line: &str, detection: &mut SpringBootDetection) {
    if line.starts_with("<spring-boot.run.mainClass>")
        || line.starts_with("<spring-boot.main-class>")
        || line.starts_with("<start-class>")
    {
        let property_name = if line.contains("spring-boot.run.mainClass") {
            "spring-boot.run.mainClass"
        } else if line.contains("spring-boot.main-class") {
            "spring-boot.main-class"
        } else {
            "start-class"
        };

        if let Some(mc) = super::xml_parser::extract_tag_content(line, property_name)
            && detection.main_class.is_none()
        {
            detection.main_class = Some(mc.clone());
            log::debug!("Found mainClass '{}' from property {}", mc, property_name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_use_spring_boot_run_with_plugin_and_jar() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: true,
            has_exec_plugin: false,
            main_class: None,
            packaging: Some("jar".to_string()),
            spring_boot_version: Some("2.5.0".to_string()),
        };
        assert!(detection.can_use_spring_boot_run());
    }

    #[test]
    fn test_can_use_spring_boot_run_with_plugin_and_war() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: true,
            has_exec_plugin: false,
            main_class: None,
            packaging: Some("war".to_string()),
            spring_boot_version: Some("2.5.0".to_string()),
        };
        assert!(detection.can_use_spring_boot_run());
    }

    #[test]
    fn test_can_use_spring_boot_run_without_plugin() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: false,
            has_exec_plugin: true,
            main_class: Some("com.example.Main".to_string()),
            packaging: Some("jar".to_string()),
            spring_boot_version: None,
        };
        assert!(!detection.can_use_spring_boot_run());
    }

    #[test]
    fn test_can_use_spring_boot_run_with_pom_packaging() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: true,
            has_exec_plugin: false,
            main_class: None,
            packaging: Some("pom".to_string()),
            spring_boot_version: Some("2.5.0".to_string()),
        };
        assert!(!detection.can_use_spring_boot_run());
    }

    #[test]
    fn test_should_prefer_spring_boot_run_war() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: true,
            has_exec_plugin: false,
            main_class: None,
            packaging: Some("war".to_string()),
            spring_boot_version: Some("2.5.0".to_string()),
        };
        assert!(detection.should_prefer_spring_boot_run());
    }

    #[test]
    fn test_should_prefer_spring_boot_run_jar() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: true,
            has_exec_plugin: false,
            main_class: None,
            packaging: Some("jar".to_string()),
            spring_boot_version: Some("2.5.0".to_string()),
        };
        assert!(!detection.should_prefer_spring_boot_run());
    }

    #[test]
    fn test_can_use_exec_java_with_plugin() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: false,
            has_exec_plugin: true,
            main_class: None,
            packaging: Some("jar".to_string()),
            spring_boot_version: None,
        };
        assert!(detection.can_use_exec_java());
    }

    #[test]
    fn test_can_use_exec_java_with_main_class() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: false,
            has_exec_plugin: false,
            main_class: Some("com.example.App".to_string()),
            packaging: Some("jar".to_string()),
            spring_boot_version: None,
        };
        assert!(detection.can_use_exec_java());
    }

    #[test]
    fn test_can_use_exec_java_neither() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: false,
            has_exec_plugin: false,
            main_class: None,
            packaging: Some("jar".to_string()),
            spring_boot_version: None,
        };
        assert!(!detection.can_use_exec_java());
    }

    #[test]
    fn test_detect_packaging() {
        let mut detection = SpringBootDetection {
            has_spring_boot_plugin: false,
            has_exec_plugin: false,
            main_class: None,
            packaging: None,
            spring_boot_version: None,
        };
        
        detect_packaging("<packaging>war</packaging>", &mut detection);
        assert_eq!(detection.packaging, Some("war".to_string()));
    }

    #[test]
    fn test_detect_packaging_jar() {
        let mut detection = SpringBootDetection {
            has_spring_boot_plugin: false,
            has_exec_plugin: false,
            main_class: None,
            packaging: None,
            spring_boot_version: None,
        };
        
        detect_packaging("<packaging>jar</packaging>", &mut detection);
        assert_eq!(detection.packaging, Some("jar".to_string()));
    }

    #[test]
    fn test_track_plugin_state() {
        let mut in_plugin = false;
        let mut in_configuration = false;
        
        track_plugin_state("<plugin>", &mut in_plugin, &mut in_configuration);
        assert!(in_plugin);
        
        track_plugin_state("</plugin>", &mut in_plugin, &mut in_configuration);
        assert!(!in_plugin);
        assert!(!in_configuration);
    }

    #[test]
    fn test_detect_plugins_spring_boot() {
        let mut detection = SpringBootDetection {
            has_spring_boot_plugin: false,
            has_exec_plugin: false,
            main_class: None,
            packaging: None,
            spring_boot_version: None,
        };
        let mut artifact_id = String::new();
        let mut in_configuration = false;
        
        detect_plugins(
            "<artifactId>spring-boot-maven-plugin</artifactId>",
            &mut artifact_id,
            &mut in_configuration,
            &mut detection,
        );
        
        assert!(detection.has_spring_boot_plugin);
        assert_eq!(artifact_id, "spring-boot-maven-plugin");
    }

    #[test]
    fn test_detect_plugins_exec() {
        let mut detection = SpringBootDetection {
            has_spring_boot_plugin: false,
            has_exec_plugin: false,
            main_class: None,
            packaging: None,
            spring_boot_version: None,
        };
        let mut artifact_id = String::new();
        let mut in_configuration = false;
        
        detect_plugins(
            "<artifactId>exec-maven-plugin</artifactId>",
            &mut artifact_id,
            &mut in_configuration,
            &mut detection,
        );
        
        assert!(detection.has_exec_plugin);
        assert_eq!(artifact_id, "exec-maven-plugin");
    }

    #[test]
    fn test_parse_effective_pom_complete() {
        let pom = r#"
            <project>
                <packaging>jar</packaging>
                <plugins>
                    <plugin>
                        <artifactId>spring-boot-maven-plugin</artifactId>
                        <version>2.7.0</version>
                        <configuration>
                            <mainClass>com.example.Application</mainClass>
                        </configuration>
                    </plugin>
                </plugins>
            </project>
        "#;
        
        let mut detection = SpringBootDetection {
            has_spring_boot_plugin: false,
            has_exec_plugin: false,
            main_class: None,
            packaging: None,
            spring_boot_version: None,
        };
        
        parse_effective_pom(pom, &mut detection);
        
        assert!(detection.has_spring_boot_plugin);
        assert_eq!(detection.packaging, Some("jar".to_string()));
        assert_eq!(detection.spring_boot_version, Some("2.7.0".to_string()));
    }
}
