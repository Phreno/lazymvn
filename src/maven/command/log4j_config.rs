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
