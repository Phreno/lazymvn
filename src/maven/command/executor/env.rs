use crate::core::config::LoggingConfig;
use crate::maven::command::log4j_config::extract_log4j_config_url;
use maven_java_agent::AgentBuilder;
use std::process::Command;

/// Configure environment variables for Maven command, including Log4j settings
pub fn configure_environment(
    command: &mut Command,
    args: &[&str],
    _logging_config: Option<&LoggingConfig>,
) {
    // CRITICAL: Set JAVA_TOOL_OPTIONS environment variable to inject Log4j configuration
    // This ensures Log4j properties are set BEFORE any application code runs
    // (including custom factories like Log4jJbossLoggerFactory that initialize in constructors)
    if let Some(log4j_config_url) = extract_log4j_config_url(args) {
        // Use the new maven-java-agent library to configure environment
        match AgentBuilder::new()
            .with_log4j_config(&log4j_config_url)
            .build()
        {
            Ok(deployment) => {
                // Set environment variables from the deployment
                for (key, value) in deployment.env_vars {
                    log::info!("Setting {}: {}", key, value);
                    command.env(key, value);
                }
            }
            Err(e) => {
                // Fallback to manual configuration if agent setup fails
                log::warn!("Failed to use maven-java-agent library: {}, using fallback", e);
                set_java_tool_options_fallback(command, &log4j_config_url);
            }
        }
    } else {
        log::debug!("No Log4j configuration URL found in args");
    }
}

/// Fallback method to set JAVA_TOOL_OPTIONS manually
fn set_java_tool_options_fallback(command: &mut Command, log4j_config_url: &str) {
    let opts_str = format!(
        "-Dlog4j.ignoreTCL=true -Dlog4j.defaultInitOverride=true -Dlog4j.configuration={}",
        log4j_config_url
    );
    log::info!("Setting JAVA_TOOL_OPTIONS with Log4j configuration: {}", log4j_config_url);
    log::info!("JAVA_TOOL_OPTIONS={}", opts_str);
    command.env("JAVA_TOOL_OPTIONS", &opts_str);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configure_environment_without_log4j_config() {
        let mut command = Command::new("echo");
        let args = vec!["clean", "install"];
        configure_environment(&mut command, &args, None);
        // Should not panic and should not set JAVA_TOOL_OPTIONS
    }

    #[test]
    fn configure_environment_with_log4j_config() {
        let mut command = Command::new("echo");
        let args = vec!["-Dlog4j.configuration=file:///tmp/log4j.xml"];
        configure_environment(&mut command, &args, None);
        // Should set JAVA_TOOL_OPTIONS or use agent
    }
}
