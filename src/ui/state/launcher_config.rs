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
        
        log::debug!("Found logging config with {} packages", logging_config.packages.len());

        let logging_overrides: Vec<(String, String)> = logging_config
            .packages
            .iter()
            .map(|pkg| (pkg.name.clone(), pkg.level.clone()))
            .collect();

        if logging_overrides.is_empty() {
            return None;
        }

        let log4j_config_path = crate::maven::generate_log4j_config(
            &tab.project_root,
            &logging_overrides,
        )?;

        let config_url = Self::path_to_file_url(&log4j_config_path);
        log::info!("Injecting Log4j 1.x configuration: {}", config_url);

        Some(format!("-Dlog4j.configuration={}", config_url))
    }

    /// Add Logback/Spring Boot logging level arguments
    pub(super) fn add_logback_logging_args(&self, jvm_args: &mut Vec<String>) {
        let tab = self.get_active_tab();
        if let Some(ref logging_config) = tab.config.logging {
            for pkg in &logging_config.packages {
                jvm_args.push(format!("-Dlogging.level.{}={}", pkg.name, pkg.level));
            }
        }
    }

    /// Generate Spring Boot properties JVM argument
    pub(super) fn generate_spring_properties_jvm_arg(&self) -> Option<String> {
        let tab = self.get_active_tab();
        let spring_config = tab.config.spring.as_ref()?;

        log::debug!(
            "Found spring config with {} properties and {} profiles",
            spring_config.properties.len(),
            spring_config.active_profiles.len()
        );

        let spring_properties: Vec<(String, String)> = spring_config
            .properties
            .iter()
            .map(|prop| (prop.name.clone(), prop.value.clone()))
            .collect();

        let spring_config_path = crate::maven::generate_spring_properties(
            &tab.project_root,
            &spring_properties,
            &spring_config.active_profiles,
        )?;

        let config_url = Self::path_to_file_url(&spring_config_path);
        log::info!("Injecting Spring Boot properties override: {}", config_url);
        log::debug!("Spring properties will OVERRIDE project defaults (LazyMVN has the last word)");

        // Log each property for debugging
        for (name, value) in &spring_properties {
            log::debug!("  Spring property override: {}={}", name, value);
        }
        if !spring_config.active_profiles.is_empty() {
            log::debug!("  Spring active profiles: {}", spring_config.active_profiles.join(","));
        }

        Some(format!("-Dspring.config.additional-location={}", config_url))
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
