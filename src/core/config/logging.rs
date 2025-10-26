// Logging configuration for JVM log level control
use serde::Deserialize;

/// Logging configuration for controlling log verbosity via JVM arguments
#[derive(Deserialize, Clone, Debug, Default, PartialEq)]
pub struct LoggingConfig {
    /// Custom log format override
    pub log_format: Option<String>,

    /// List of packages with custom log levels
    #[serde(default)]
    pub packages: Vec<PackageLogLevel>,
}

/// Package-specific log level configuration
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct PackageLogLevel {
    /// Package name (e.g., "com.mycompany.api.service")
    pub name: String,
    /// Log level (e.g., "ERROR", "WARN", "INFO", "DEBUG", "TRACE")
    pub level: String,
}

#[allow(dead_code)] // Public API methods not yet used
impl LoggingConfig {
    /// Validate that all log levels are valid
    pub fn validate(&self) -> Result<(), String> {
        for pkg in &self.packages {
            validate_log_level(&pkg.level)?;
        }
        Ok(())
    }

    /// Check if logging configuration is empty
    pub fn is_empty(&self) -> bool {
        self.packages.is_empty()
    }

    /// Get log level for a specific package
    pub fn get_level(&self, package_name: &str) -> Option<&str> {
        self.packages
            .iter()
            .find(|p| p.name == package_name)
            .map(|p| p.level.as_str())
    }
}

/// Validate a log level string
#[allow(dead_code)] // Used by public API methods
fn validate_log_level(level: &str) -> Result<(), String> {
    let valid_levels = ["OFF", "ERROR", "WARN", "INFO", "DEBUG", "TRACE", "ALL"];
    let level_upper = level.to_uppercase();

    if valid_levels.contains(&level_upper.as_str()) {
        Ok(())
    } else {
        Err(format!(
            "Invalid log level '{}'. Valid levels are: {}",
            level,
            valid_levels.join(", ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_log_level_valid() {
        assert!(validate_log_level("ERROR").is_ok());
        assert!(validate_log_level("WARN").is_ok());
        assert!(validate_log_level("INFO").is_ok());
        assert!(validate_log_level("DEBUG").is_ok());
        assert!(validate_log_level("TRACE").is_ok());
        assert!(validate_log_level("OFF").is_ok());
        assert!(validate_log_level("ALL").is_ok());
    }

    #[test]
    fn test_validate_log_level_case_insensitive() {
        assert!(validate_log_level("error").is_ok());
        assert!(validate_log_level("Error").is_ok());
        assert!(validate_log_level("eRRoR").is_ok());
    }

    #[test]
    fn test_validate_log_level_invalid() {
        assert!(validate_log_level("INVALID").is_err());
        assert!(validate_log_level("FINE").is_err());
        assert!(validate_log_level("").is_err());
    }

    #[test]
    fn test_logging_config_validate() {
        let config = LoggingConfig {
            log_format: None,
            packages: vec![
                PackageLogLevel {
                    name: "com.example".to_string(),
                    level: "DEBUG".to_string(),
                },
                PackageLogLevel {
                    name: "org.springframework".to_string(),
                    level: "WARN".to_string(),
                },
            ],
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_logging_config_validate_invalid() {
        let config = LoggingConfig {
            log_format: None,
            packages: vec![PackageLogLevel {
                name: "com.example".to_string(),
                level: "INVALID".to_string(),
            }],
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_logging_config_get_level() {
        let config = LoggingConfig {
            log_format: None,
            packages: vec![
                PackageLogLevel {
                    name: "com.example".to_string(),
                    level: "DEBUG".to_string(),
                },
                PackageLogLevel {
                    name: "org.springframework".to_string(),
                    level: "WARN".to_string(),
                },
            ],
        };

        assert_eq!(config.get_level("com.example"), Some("DEBUG"));
        assert_eq!(config.get_level("org.springframework"), Some("WARN"));
        assert_eq!(config.get_level("nonexistent"), None);
    }
}
