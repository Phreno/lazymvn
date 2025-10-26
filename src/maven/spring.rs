//! Spring Boot properties file generation
//!
//! This module handles automatic generation of Spring Boot application.properties files
//! to override configuration based on lazymvn.toml [spring] section.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Generate a Spring Boot application.properties file with property overrides
///
/// This function creates an application.properties file in LazyMVN's config directory
/// that overrides Spring Boot configuration defined in lazymvn.toml.
///
/// # Arguments
/// * `project_root` - The root directory of the Maven project (used for hashing)
/// * `properties` - List of (name, value) tuples to override
/// * `active_profiles` - Optional list of active Spring profiles
///
/// # Returns
/// * `Some(PathBuf)` - Path to the generated config file
/// * `None` - If file creation failed or no properties to override
pub fn generate_spring_properties(
    project_root: &Path,
    properties: &[(String, String)],
    active_profiles: &[String],
) -> Option<PathBuf> {
    if properties.is_empty() && active_profiles.is_empty() {
        return None;
    }

    // Use LazyMVN's config directory (~/.config/lazymvn/)
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
        .join("lazymvn")
        .join("spring");

    if let Err(e) = fs::create_dir_all(&config_dir) {
        log::error!("Failed to create spring config directory: {}", e);
        return None;
    }

    // Create a hash of the project root path for unique filename
    let hash = format!(
        "{:x}",
        md5::compute(project_root.to_string_lossy().as_bytes())
    )
    .chars()
    .take(8)
    .collect::<String>();

    let config_path = config_dir.join(format!("application-override-{}.properties", hash));

    // Generate Spring Boot properties content
    let mut content = String::new();
    content.push_str("# LazyMVN Generated Spring Boot Configuration\n");
    content.push_str("# This file is auto-generated from lazymvn.toml [spring] section\n");
    content.push_str("# These properties OVERRIDE project defaults (LazyMVN has the last word)\n");
    content.push('\n');

    // Add active profiles if specified
    if !active_profiles.is_empty() {
        content.push_str("# Active profiles\n");
        content.push_str(&format!(
            "spring.profiles.active={}\n",
            active_profiles.join(",")
        ));
        content.push('\n');
    }

    // Add property overrides
    if !properties.is_empty() {
        content.push_str("# Property overrides from lazymvn.toml\n");
        for (name, value) in properties {
            content.push_str(&format!("{}={}\n", name, value));
        }
    }

    // Write to file
    match fs::File::create(&config_path) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(content.as_bytes()) {
                log::error!("Failed to write Spring properties file: {}", e);
                return None;
            }
            log::info!(
                "Generated Spring Boot override config at: {:?}",
                config_path
            );
            log::debug!("Spring properties override content:\n{}", content);
            Some(config_path)
        }
        Err(e) => {
            log::error!("Failed to create Spring properties file: {}", e);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_spring_properties_with_properties() {
        // Use unique path per test to avoid race conditions
        let test_dir = std::env::temp_dir().join("lazymvn_test_properties");
        let properties = vec![
            ("server.port".to_string(), "8081".to_string()),
            (
                "spring.datasource.url".to_string(),
                "jdbc:h2:mem:testdb".to_string(),
            ),
        ];
        let profiles = vec![];

        let config_path = generate_spring_properties(&test_dir, &properties, &profiles);
        assert!(config_path.is_some());

        let path = config_path.unwrap();
        assert!(path.exists());

        // Check content
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("server.port=8081"));
        assert!(content.contains("spring.datasource.url=jdbc:h2:mem:testdb"));
        assert!(content.contains("LazyMVN Generated Spring Boot Configuration"));

        // Cleanup
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_generate_spring_properties_with_profiles() {
        // Use unique path per test to avoid race conditions
        let test_dir = std::env::temp_dir().join("lazymvn_test_profiles");
        let properties = vec![];
        let profiles = vec!["dev".to_string(), "local".to_string()];

        let config_path = generate_spring_properties(&test_dir, &properties, &profiles);
        assert!(config_path.is_some());

        let path = config_path.unwrap();
        assert!(path.exists());

        // Check content
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("spring.profiles.active=dev,local"));

        // Cleanup
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_generate_spring_properties_empty() {
        // Use unique path per test to avoid race conditions
        let test_dir = std::env::temp_dir().join("lazymvn_test_empty");
        let properties = vec![];
        let profiles = vec![];

        let config_path = generate_spring_properties(&test_dir, &properties, &profiles);
        assert!(config_path.is_none());
    }
}
