//! Launcher configuration helpers
//!
//! Helper functions for building JVM arguments and Spring Boot configuration
//! for launching applications with custom logging and properties.

use std::path::Path;

use super::TuiState;

impl TuiState {
    /// Build JVM arguments from logging and Spring configurations
    pub(super) fn build_jvm_args_for_launcher(&self) -> Vec<String> {
        let mut jvm_args = Vec::new();

        // Add Log4j configuration arguments
        if let Some(log4j_arg) = self.generate_log4j_jvm_arg() {
            // Prevent Log4j from auto-loading log4j.properties from classpath
            // This ensures our configuration file takes precedence
            jvm_args.push("-Dlog4j.defaultInitOverride=true".to_string());
            jvm_args.push(log4j_arg);
        }

        // Add Logback/Spring Boot logging level arguments
        self.add_logback_logging_args(&mut jvm_args);

        // Add Spring Boot properties configuration
        if let Some(spring_arg) = self.generate_spring_properties_jvm_arg() {
            jvm_args.push(spring_arg);
        }

        log::debug!("Generated {} JVM args total", jvm_args.len());
        for arg in &jvm_args {
            log::debug!("  JVM arg: {}", arg);
        }

        jvm_args
    }

    /// Generate Log4j configuration JVM argument
    pub(super) fn generate_log4j_jvm_arg(&self) -> Option<String> {
        let tab = self.get_active_tab();
        let logging_config = tab.config.logging.as_ref()?;

        log::debug!(
            "Found logging config with {} packages",
            logging_config.packages.len()
        );

        let logging_overrides: Vec<(String, String)> = logging_config
            .packages
            .iter()
            .map(|pkg| (pkg.name.clone(), pkg.level.clone()))
            .collect();

        if logging_overrides.is_empty() && logging_config.log_format.is_none() {
            return None;
        }

        let log4j_config_path = crate::maven::generate_log4j_config(
            &tab.project_root,
            &logging_overrides,
            logging_config.log_format.as_deref(),
        )?;

        let config_url = Self::path_to_file_url(&log4j_config_path);
        log::info!("Injecting Log4j 1.x configuration: {}", config_url);

        // Return both the configuration file URL and the defaultInitOverride flag
        // The defaultInitOverride prevents Log4j from auto-loading log4j.properties from classpath
        Some(format!("-Dlog4j.configuration={}", config_url))
    }

    /// Add Logback/Spring Boot logging level arguments
    /// Also adds Log4j 1.x logger arguments for compatibility
    pub(super) fn add_logback_logging_args(&self, jvm_args: &mut Vec<String>) {
        let tab = self.get_active_tab();
        if let Some(ref logging_config) = tab.config.logging {
            for pkg in &logging_config.packages {
                // Add both Logback (Spring Boot) and Log4j 1.x arguments
                // This ensures logging levels work regardless of the framework
                jvm_args.push(format!("-Dlogging.level.{}={}", pkg.name, pkg.level));
                jvm_args.push(format!("-Dlog4j.logger.{}={}", pkg.name, pkg.level));
            }
        }
    }

    /// Generate Spring Boot properties JVM argument
    pub(super) fn generate_spring_properties_jvm_arg(&self) -> Option<String> {
        let tab = self.get_active_tab();

        let mut spring_properties: Vec<(String, String)> = Vec::new();
        let mut active_profiles: Vec<String> = Vec::new();
        let mut inserted_log_format = false;

        if let Some(spring_config) = tab.config.spring.as_ref() {
            log::debug!(
                "Found spring config with {} properties and {} profiles",
                spring_config.properties.len(),
                spring_config.active_profiles.len()
            );

            spring_properties.extend(
                spring_config
                    .properties
                    .iter()
                    .map(|prop| (prop.name.clone(), prop.value.clone())),
            );
            active_profiles.extend(spring_config.active_profiles.clone());
        }

        if let Some(logging_config) = tab.config.logging.as_ref()
            && let Some(log_format) = logging_config.log_format.as_ref()
        {
            // Remove any existing pattern overrides defined in the [spring] section
            spring_properties.retain(|(name, _)| {
                name != "logging.pattern.console" && name != "logging.pattern.file"
            });

            spring_properties.push(("logging.pattern.console".to_string(), log_format.clone()));
            spring_properties.push(("logging.pattern.file".to_string(), log_format.clone()));
            inserted_log_format = true;

            log::debug!(
                "Added logging format override to Spring properties: {}",
                log_format
            );
        }

        let spring_config_path = crate::maven::generate_spring_properties(
            &tab.project_root,
            &spring_properties,
            &active_profiles,
        )?;

        let config_url = Self::path_to_file_url(&spring_config_path);
        log::info!("Injecting Spring Boot properties override: {}", config_url);
        log::debug!("Spring properties will OVERRIDE project defaults (LazyMVN has the last word)");

        for (name, value) in &spring_properties {
            log::debug!("  Spring property override: {}={}", name, value);
        }
        if !active_profiles.is_empty() {
            log::debug!("  Spring active profiles: {}", active_profiles.join(","));
        }
        if inserted_log_format {
            log::debug!("  Logging pattern overrides applied via properties file");
        }

        Some(format!(
            "-Dspring.config.additional-location={}",
            config_url
        ))
    }

    /// Convert file path to file:// URL (cross-platform)
    pub(super) fn path_to_file_url(path: &Path) -> String {
        if cfg!(windows) {
            format!("file:///{}", path.display().to_string().replace('\\', "/"))
        } else {
            format!("file://{}", path.display())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TuiState;
    use std::path::Path;

    #[cfg(not(windows))]
    #[test]
    fn test_path_to_file_url_unix() {
        let path = Path::new("/tmp/project/config.properties");
        let url = TuiState::path_to_file_url(path);
        assert_eq!(url, "file:///tmp/project/config.properties");
    }

    #[cfg(windows)]
    #[test]
    fn test_path_to_file_url_windows() {
        let path = Path::new(r"C:\Projects\App\config.properties");
        let url = TuiState::path_to_file_url(path);
        assert_eq!(url, "file:///C:/Projects/App/config.properties");
    }
}
