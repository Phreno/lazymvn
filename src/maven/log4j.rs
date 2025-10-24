

//! Log4j 1.x configuration file generation
//!
//! This module handles automatic generation of Log4j 1.x configuration files
//! to override logging levels based on lazymvn.toml configuration.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Generate a temporary Log4j 1.x properties file with the specified logging overrides
///
/// Returns the path to the generated file, or None if generation failed
pub fn generate_log4j_config(
    project_root: &Path,
    logging_overrides: &[(String, String)],
) -> Option<PathBuf> {
    if logging_overrides.is_empty() {
        return None;
    }

    // Create .lazymvn directory in project root for temp files
    let temp_dir = project_root.join(".lazymvn");
    if let Err(e) = fs::create_dir_all(&temp_dir) {
        log::error!("Failed to create .lazymvn directory: {}", e);
        return None;
    }

    let config_path = temp_dir.join("log4j-override.properties");

    // Generate Log4j 1.x properties content
    let mut content = String::new();
    
    // Root logger configuration
    content.push_str("# LazyMVN Generated Log4j 1.x Configuration\n");
    content.push_str("# This file is auto-generated from lazymvn.toml [logging] section\n");
    content.push('\n');
    content.push_str("# Root logger\n");
    content.push_str("log4j.rootLogger=INFO, CONSOLE\n");
    content.push('\n');
    
    // Console appender configuration
    content.push_str("# Console appender\n");
    content.push_str("log4j.appender.CONSOLE=org.apache.log4j.ConsoleAppender\n");
    content.push_str("log4j.appender.CONSOLE.layout=org.apache.log4j.PatternLayout\n");
    content.push_str("log4j.appender.CONSOLE.layout.ConversionPattern=[%d{dd/MM/yyyy HH:mm:ss:SSS}] %5p %c{1} - %m%n\n");
    content.push('\n');
    
    // Add logging overrides
    content.push_str("# Logging level overrides from lazymvn.toml\n");
    for (package, level) in logging_overrides {
        content.push_str(&format!("log4j.logger.{}={}\n", package, level));
    }

    // Write to file
    match fs::File::create(&config_path) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(content.as_bytes()) {
                log::error!("Failed to write Log4j config file: {}", e);
                return None;
            }
            log::info!("Generated Log4j override config at: {:?}", config_path);
            Some(config_path)
        }
        Err(e) => {
            log::error!("Failed to create Log4j config file: {}", e);
            None
        }
    }
}

/// Check if the application output indicates Log4j 1.x usage
///
/// Looks for common indicators like:
/// - "Log4jJbossLoggerFactory"
/// - "log4j.properties"
/// - "log4j:WARN" messages
#[allow(dead_code)]
pub fn detect_log4j1_usage(output_lines: &[String]) -> bool {
    for line in output_lines.iter().take(50) {
        // Check first 50 lines for Log4j indicators
        let lower = line.to_lowercase();
        if lower.contains("log4jjbossloggerfactory")
            || lower.contains("log4j.properties")
            || lower.contains("log4j:warn")
            || lower.contains("log4j:error")
            || (lower.contains("log4j") && lower.contains("configuration"))
        {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_log4j1_usage() {
        let lines = vec![
            "Starting application...".to_string(),
            "Log4jJbossLoggerFactory : utilise le fichier log4j.properties".to_string(),
            "Application started".to_string(),
        ];
        assert!(detect_log4j1_usage(&lines));

        let lines_no_log4j = vec![
            "Starting application...".to_string(),
            "Using Logback for logging".to_string(),
        ];
        assert!(!detect_log4j1_usage(&lines_no_log4j));
    }

    #[test]
    fn test_generate_log4j_config() {
        let temp_dir = std::env::temp_dir();
        let overrides = vec![
            ("org.springframework".to_string(), "WARN".to_string()),
            ("com.example".to_string(), "DEBUG".to_string()),
        ];

        let config_path = generate_log4j_config(&temp_dir, &overrides);
        assert!(config_path.is_some());

        let path = config_path.unwrap();
        assert!(path.exists());

        // Check content
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("log4j.logger.org.springframework=WARN"));
        assert!(content.contains("log4j.logger.com.example=DEBUG"));
        assert!(content.contains("log4j.rootLogger"));

        // Cleanup
        let _ = fs::remove_file(path);
    }
}
