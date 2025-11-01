//! Maven command string building utilities

use crate::core::config::LoggingConfig;
use std::path::Path;
use super::log4j_config::get_logging_overrides;

/// Determine the Maven command to use (wrapper or system Maven)
///
/// Prefers Maven wrapper (mvnw) over system Maven when available.
///
/// # Examples
///
/// ```ignore
/// use std::path::Path;
/// use lazymvn::maven::command::get_maven_command;
///
/// let project_root = Path::new("/path/to/project");
/// let maven_cmd = get_maven_command(project_root);
/// // Returns "./mvnw" if mvnw exists, otherwise "mvn" (or "mvn.cmd" on Windows)
/// ```
pub fn get_maven_command(project_root: &Path) -> String {
    // On Unix, check for mvnw
    #[cfg(unix)]
    {
        if project_root.join("mvnw").exists() {
            return "./mvnw".to_string();
        }
    }

    // On Windows, check for mvnw.bat, mvnw.cmd, or mvnw
    #[cfg(windows)]
    {
        if project_root.join("mvnw.bat").exists() {
            return "mvnw.bat".to_string();
        }
        if project_root.join("mvnw.cmd").exists() {
            return "mvnw.cmd".to_string();
        }
        if project_root.join("mvnw").exists() {
            return "mvnw".to_string();
        }
    }

    // On Windows, use mvn.cmd; on Unix, use mvn
    #[cfg(windows)]
    {
        "mvn.cmd".to_string()
    }
    #[cfg(not(windows))]
    {
        "mvn".to_string()
    }
}

/// Build Maven command string for display purposes
#[allow(dead_code)]
pub fn build_command_string(
    maven_command: &str,
    module: Option<&str>,
    args: &[&str],
    profiles: &[String],
    settings_path: Option<&str>,
    flags: &[String],
) -> String {
    build_command_string_with_options(
        maven_command,
        module,
        args,
        profiles,
        settings_path,
        flags,
        false,
        Path::new("."),
        None, // No logging overrides for backward compatibility
    )
}

/// Build the full command string for display with option to use -f
#[allow(clippy::too_many_arguments)]
pub fn build_command_string_with_options(
    maven_command: &str,
    module: Option<&str>,
    args: &[&str],
    profiles: &[String],
    settings_path: Option<&str>,
    flags: &[String],
    use_file_flag: bool,
    project_root: &Path,
    logging_config: Option<&LoggingConfig>,
) -> String {
    let mut parts = vec![maven_command.to_string()];

    if let Some(settings_path) = settings_path {
        parts.push("--settings".to_string());
        parts.push(settings_path.to_string());
    }

    if !profiles.is_empty() {
        parts.push("-P".to_string());
        parts.push(profiles.join(","));
    }

    if let Some(module) = module
        && module != "."
    {
        if use_file_flag {
            let module_pom = project_root.join(module).join("pom.xml");
            parts.push("-f".to_string());
            parts.push(module_pom.to_string_lossy().to_string());

            // Auto-add --also-make for exec:java to ensure dependencies are built
            if args.contains(&"exec:java") && !flags.iter().any(|f| f.contains("also-make")) {
                parts.push("--also-make".to_string());
            }
        } else {
            parts.push("-pl".to_string());
            parts.push(module.to_string());

            // Note: We don't auto-add --also-make for spring-boot:run because it would
            // try to execute the goal on all modules in the reactor (including parent POM).
        }
    }

    // Filter incompatible flags for spring-boot:run
    let is_spring_boot_run = args.iter().any(|arg| {
        arg.contains("spring-boot:run") || arg.contains("spring-boot-maven-plugin") && arg.contains(":run")
    });
    
    let filtered_flags: Vec<&str> = if is_spring_boot_run {
        flags.iter()
            .filter(|flag| {
                let flag_lower = flag.to_lowercase();
                !flag_lower.contains("also-make")
            })
            .map(|s| s.as_str())
            .collect()
    } else {
        flags.iter().map(|s| s.as_str()).collect()
    };

    for flag in filtered_flags {
        // Split flags like "-U, --update-snapshots" into individual flags
        // Take only the first part before comma to skip aliases
        let flag_parts: Vec<&str> = flag
            .split(',')
            .next()
            .unwrap_or(flag)
            .split_whitespace()
            .collect();
        
        for part in flag_parts {
            if !part.is_empty() {
                parts.push(part.to_string());
            }
        }
    }

    let has_spring_boot_jvm_args = args
        .iter()
        .any(|arg| arg.starts_with("-Dspring-boot.run.jvmArguments=") || arg.starts_with("-Drun.jvmArguments="));

    if let Some(logging_config) = logging_config
        && !has_spring_boot_jvm_args
    {
        if let Some(log_format) = &logging_config.log_format {
            parts.push(format!("-Dlog4j.conversionPattern={}", log_format));
            parts.push(format!("-Dlogging.pattern.console={}", log_format));
        }
        for (package, level) in &get_logging_overrides(Some(logging_config)) {
            parts.push(format!("-Dlog4j.logger.{}={}", package, level));
            parts.push(format!("-Dlogging.level.{}={}", package, level));
        }
    }

    for arg in args {
        parts.push(arg.to_string());
    }

    parts.join(" ")
}

/// Check Maven availability and return version info
pub fn check_maven_availability(project_root: &Path) -> Result<String, std::io::Error> {
    use std::process::Command;
    
    let maven_command = get_maven_command(project_root);

    let output = Command::new(&maven_command)
        .arg("--version")
        .current_dir(project_root)
        .output()?;

    if !output.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Maven command '{}' failed", maven_command),
        ));
    }

    let version_output = String::from_utf8_lossy(&output.stdout);
    let first_line = version_output
        .lines()
        .next()
        .unwrap_or("Unknown version")
        .to_string();

    Ok(first_line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_get_maven_command_with_wrapper() {
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join("test_maven_wrapper");
        std::fs::create_dir_all(&test_dir).ok();
        
        #[cfg(unix)]
        {
            let mvnw_path = test_dir.join("mvnw");
            std::fs::write(&mvnw_path, "#!/bin/sh\necho test").ok();
            
            let result = get_maven_command(&test_dir);
            assert_eq!(result, "./mvnw");
        }
        
        #[cfg(windows)]
        {
            let mvnw_path = test_dir.join("mvnw.bat");
            std::fs::write(&mvnw_path, "@echo test").ok();
            
            let result = get_maven_command(&test_dir);
            assert_eq!(result, "mvnw.bat");
        }
        
        std::fs::remove_dir_all(&test_dir).ok();
    }

    #[test]
    fn test_get_maven_command_without_wrapper() {
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join("test_no_wrapper");
        std::fs::create_dir_all(&test_dir).ok();
        
        let result = get_maven_command(&test_dir);
        
        #[cfg(windows)]
        assert_eq!(result, "mvn.cmd");
        
        #[cfg(not(windows))]
        assert_eq!(result, "mvn");
        
        std::fs::remove_dir_all(&test_dir).ok();
    }

    #[test]
    fn test_build_command_string_basic() {
        let result = build_command_string(
            "mvn",
            None,
            &["clean", "install"],
            &[],
            None,
            &[],
        );
        assert_eq!(result, "mvn clean install");
    }

    #[test]
    fn test_build_command_string_with_module() {
        let result = build_command_string(
            "mvn",
            Some("mymodule"),
            &["test"],
            &[],
            None,
            &[],
        );
        assert!(result.contains("-pl"));
        assert!(result.contains("mymodule"));
        assert!(result.contains("test"));
    }

    #[test]
    fn test_build_command_string_with_profiles() {
        let result = build_command_string(
            "mvn",
            None,
            &["package"],
            &["dev".to_string(), "debug".to_string()],
            None,
            &[],
        );
        assert!(result.contains("-P"));
        assert!(result.contains("dev,debug"));
    }

    #[test]
    fn test_build_command_string_with_settings() {
        let result = build_command_string(
            "mvn",
            None,
            &["install"],
            &[],
            Some("/path/to/settings.xml"),
            &[],
        );
        assert!(result.contains("--settings"));
        assert!(result.contains("/path/to/settings.xml"));
    }

    #[test]
    fn test_build_command_string_with_flags() {
        let result = build_command_string(
            "mvn",
            None,
            &["test"],
            &[],
            None,
            &["-U".to_string(), "--offline".to_string()],
        );
        assert!(result.contains("-U"));
        assert!(result.contains("--offline"));
    }

    #[test]
    fn test_build_command_string_with_options_file_flag() {
        let project_root = PathBuf::from("/tmp/project");
        let result = build_command_string_with_options(
            "mvn",
            Some("module1"),
            &["exec:java"],
            &[],
            None,
            &[],
            true,
            &project_root,
            None,
        );
        
        assert!(result.contains("-f"));
        assert!(result.contains("module1/pom.xml"));
        assert!(result.contains("--also-make"));
    }

    #[test]
    fn test_build_command_string_filters_also_make_for_spring_boot() {
        let result = build_command_string(
            "mvn",
            None,
            &["spring-boot:run"],
            &[],
            None,
            &["--also-make".to_string()],
        );
        
        assert!(!result.contains("--also-make"));
        assert!(result.contains("spring-boot:run"));
    }

    #[test]
    fn test_build_command_string_with_logging_config() {
        let logging_config = LoggingConfig {
            log_format: Some("%d{HH:mm:ss.SSS} [%t] %-5level %logger{36} - %msg%n".to_string()),
            ..Default::default()
        };
        
        let result = build_command_string_with_options(
            "mvn",
            None,
            &["test"],
            &[],
            None,
            &[],
            false,
            Path::new("."),
            Some(&logging_config),
        );
        
        assert!(result.contains("-Dlog4j.conversionPattern="));
        assert!(result.contains("-Dlogging.pattern.console="));
    }

    #[test]
    fn test_build_command_string_skips_logging_with_jvm_args() {
        let logging_config = LoggingConfig::default();
        
        let result = build_command_string_with_options(
            "mvn",
            None,
            &["-Dspring-boot.run.jvmArguments=-Xmx512m"],
            &[],
            None,
            &[],
            false,
            Path::new("."),
            Some(&logging_config),
        );
        
        // Should not add logging config when JVM args are present
        assert!(!result.contains("-Dlog4j.conversionPattern="));
    }

    #[test]
    fn test_build_command_string_with_comma_separated_flag() {
        let result = build_command_string(
            "mvn",
            None,
            &["clean"],
            &[],
            None,
            &["-U, --update-snapshots".to_string()],
        );
        
        // Should only use the first part before comma
        assert!(result.contains("-U"));
        assert!(!result.contains(","));
    }
}
