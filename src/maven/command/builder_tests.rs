//! Tests for Maven command builder

#[cfg(test)]
mod tests {
    use super::super::builder::*;
    use crate::core::config::LoggingConfig;
    use std::path::{Path, PathBuf};

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
    fn test_wrapper_exists() {
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join("test_wrapper_exists");
        std::fs::create_dir_all(&test_dir).ok();
        
        let mvnw_path = test_dir.join("mvnw");
        std::fs::write(&mvnw_path, "test").ok();
        
        assert!(wrapper_exists(&test_dir, "mvnw"));
        assert!(!wrapper_exists(&test_dir, "nonexistent"));
        
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
