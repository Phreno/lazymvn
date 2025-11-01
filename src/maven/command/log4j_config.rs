//! Log4j configuration extraction utilities

use crate::core::config::LoggingConfig;

/// Extract Log4j configuration URL from Maven JVM arguments
///
/// Searches for `-Dlog4j.configuration=file:///...` in the JVM arguments string
/// (either in -Drun.jvmArguments or -Dspring-boot.run.jvmArguments).
///
/// Returns the full file:/// URL if found.
pub(super) fn extract_log4j_config_url(args: &[&str]) -> Option<String> {
    for arg in args {
        // Check if this is a JVM arguments string
        if arg.starts_with("-Drun.jvmArguments=") || arg.starts_with("-Dspring-boot.run.jvmArguments=") {
            // Extract the value part after the FIRST '=' using split_once
            // This is critical because the value contains multiple '=' signs
            // Example: "-Drun.jvmArguments=-Dlog4j.configuration=file:///..."
            if let Some((_, jvm_args_str)) = arg.split_once('=') {
                // Look for -Dlog4j.configuration=file:///...
                for part in jvm_args_str.split_whitespace() {
                    if let Some(config_url) = part.strip_prefix("-Dlog4j.configuration=") {
                        return Some(config_url.to_string());
                    }
                }
            }
        }
    }
    None
}

/// Extract logging overrides from config
///
/// # Examples
///
/// ```
/// use lazymvn::core::config::{LoggingConfig, PackageLogLevel};
///
/// let mut config = LoggingConfig::default();
/// config.packages.push(PackageLogLevel {
///     name: "com.example".to_string(),
///     level: "DEBUG".to_string(),
/// });
/// // Function is internal, tested indirectly through public APIs
/// ```
pub(super) fn get_logging_overrides(logging_config: Option<&LoggingConfig>) -> Vec<(String, String)> {
    logging_config
        .map(|config| {
            config
                .packages
                .iter()
                .map(|pkg| (pkg.name.clone(), pkg.level.clone()))
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::PackageLogLevel;

    #[test]
    fn test_extract_log4j_config_url_run_jvm_arguments() {
        let args = vec![
            "-Drun.jvmArguments=-Dlog4j.configuration=file:///path/to/log4j.xml",
        ];
        let result = extract_log4j_config_url(&args);
        assert_eq!(result, Some("file:///path/to/log4j.xml".to_string()));
    }

    #[test]
    fn test_extract_log4j_config_url_spring_boot() {
        let args = vec![
            "-Dspring-boot.run.jvmArguments=-Dlog4j.configuration=file:///config/log4j.xml",
        ];
        let result = extract_log4j_config_url(&args);
        assert_eq!(result, Some("file:///config/log4j.xml".to_string()));
    }

    #[test]
    fn test_extract_log4j_config_url_multiple_jvm_args() {
        let args = vec![
            "-Dspring-boot.run.jvmArguments=-Xmx512m -Dlog4j.configuration=file:///tmp/log4j.xml -Dfoo=bar",
        ];
        let result = extract_log4j_config_url(&args);
        assert_eq!(result, Some("file:///tmp/log4j.xml".to_string()));
    }

    #[test]
    fn test_extract_log4j_config_url_not_found() {
        let args = vec!["-Dfoo=bar", "-Dother.property=value"];
        let result = extract_log4j_config_url(&args);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_log4j_config_url_empty() {
        let args: Vec<&str> = vec![];
        let result = extract_log4j_config_url(&args);
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_logging_overrides_with_packages() {
        let mut config = LoggingConfig::default();
        config.packages.push(PackageLogLevel {
            name: "com.example".to_string(),
            level: "DEBUG".to_string(),
        });
        config.packages.push(PackageLogLevel {
            name: "org.test".to_string(),
            level: "TRACE".to_string(),
        });

        let result = get_logging_overrides(Some(&config));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("com.example".to_string(), "DEBUG".to_string()));
        assert_eq!(result[1], ("org.test".to_string(), "TRACE".to_string()));
    }

    #[test]
    fn test_get_logging_overrides_empty() {
        let config = LoggingConfig::default();
        let result = get_logging_overrides(Some(&config));
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_get_logging_overrides_none() {
        let result = get_logging_overrides(None);
        assert_eq!(result.len(), 0);
    }
}
