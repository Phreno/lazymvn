// Configuration file I/O operations
use std::fs;
use std::path::{Path, PathBuf};

use super::types::Config;

/// Get the LazyMVN configuration directory
/// Returns ~/.config/lazymvn/ (or equivalent on Windows/Mac)
pub(super) fn get_config_dir() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| {
            log::warn!("Could not determine config directory, using home");
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
        })
        .join("lazymvn");

    log::debug!("Config directory: {:?}", config_dir);
    config_dir
}

/// Get the configuration file path for a specific project
/// Returns ~/.config/lazymvn/projects/<hash>/config.toml
pub fn get_project_config_path(project_root: &Path) -> PathBuf {
    let config_dir = get_config_dir();
    let project_hash = format!(
        "{:x}",
        md5::compute(project_root.to_string_lossy().as_bytes())
    )
    .chars()
    .take(8)
    .collect::<String>();

    config_dir
        .join("projects")
        .join(&project_hash)
        .join("config.toml")
}

/// Check if a project has a configuration file
pub fn has_project_config(project_root: &Path) -> bool {
    get_project_config_path(project_root).exists()
}

/// Create a project configuration file from template
/// Returns the path to the created configuration file
pub fn create_project_config(project_root: &Path) -> Result<PathBuf, String> {
    // Get config path
    let config_path = get_project_config_path(project_root);
    let config_dir = config_path.parent().unwrap();

    // Create directory structure
    fs::create_dir_all(config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    // Read template file
    let template_content = include_str!("../../../config_template.toml");

    // Write to config path
    fs::write(&config_path, template_content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(config_path)
}

/// Load configuration from disk
/// First checks centralized location, then falls back to legacy location
pub fn load_config(project_root: &Path) -> Config {
    log::debug!("Loading config from project root: {:?}", project_root);
    let mut config: Config = {
        // First try the new centralized location
        let config_path = get_project_config_path(project_root);
        log::debug!("Checking for config file at: {:?}", config_path);

        if let Ok(content) = fs::read_to_string(&config_path) {
            log::info!(
                "Found config.toml in centralized location, parsing configuration"
            );
            match toml::from_str(&content) {
                Ok(cfg) => {
                    log::debug!("Successfully parsed config.toml");
                    cfg
                }
                Err(e) => {
                    log::error!("Failed to parse config.toml: {}", e);
                    log::error!("Using default configuration instead");
                    Config::default()
                }
            }
        } else {
            // Fallback: try legacy location (project_root/lazymvn.toml) for backward compatibility
            let legacy_config_path = project_root.join("lazymvn.toml");
            log::debug!(
                "Checking legacy config file at: {:?}",
                legacy_config_path
            );

            if let Ok(content) = fs::read_to_string(&legacy_config_path) {
                log::warn!("Found lazymvn.toml in project root (legacy location)");
                log::warn!(
                    "Consider running 'lazymvn --setup' to migrate to centralized config"
                );
                match toml::from_str(&content) {
                    Ok(cfg) => {
                        log::debug!("Successfully parsed legacy lazymvn.toml");
                        cfg
                    }
                    Err(e) => {
                        log::error!("Failed to parse lazymvn.toml: {}", e);
                        log::error!("Using default configuration instead");
                        Config::default()
                    }
                }
            } else {
                log::debug!("No config file found, using defaults");
                Config::default()
            }
        }
    };

    // Auto-detect Maven settings if not configured
    if config.maven_settings.is_none() {
        log::debug!("No maven_settings in config, searching for settings.xml");
        config.maven_settings =
            find_maven_settings(project_root).map(|p| p.to_str().unwrap().to_string());
    }

    if let Some(ref settings) = config.maven_settings {
        log::info!("Using Maven settings file: {}", settings);
    } else {
        log::debug!("No Maven settings file found");
    }

    // Log loaded logging configuration
    if let Some(ref logging) = config.logging {
        log::debug!(
            "Loaded logging config with {} packages:",
            logging.packages.len()
        );
        for pkg in &logging.packages {
            log::debug!("  {} = {}", pkg.name, pkg.level);
        }
    } else {
        log::debug!("No logging configuration found in lazymvn.toml");
    }

    config
}

/// Find Maven settings.xml in project directory
/// Searches for maven_settings.xml or settings.xml
fn find_maven_settings(project_root: &Path) -> Option<PathBuf> {
    let filenames = ["maven_settings.xml", "settings.xml"];

    for filename in &filenames {
        let path = project_root.join(filename);
        if path.exists() {
            log::debug!("Found Maven settings file: {:?}", path);
            return Some(path);
        }
    }

    log::debug!("No Maven settings file found in project root");
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_get_config_dir() {
        let config_dir = get_config_dir();
        assert!(config_dir.ends_with("lazymvn"));
    }

    #[test]
    fn test_get_project_config_path() {
        let project_root = PathBuf::from("/tmp/test-project");
        let config_path = get_project_config_path(&project_root);

        assert!(config_path.to_string_lossy().contains("lazymvn"));
        assert!(config_path.to_string_lossy().contains("projects"));
        assert!(config_path.ends_with("config.toml"));
    }

    #[test]
    fn test_has_project_config_false() {
        let project_root = PathBuf::from("/tmp/nonexistent-project-12345");
        assert!(!has_project_config(&project_root));
    }

    #[test]
    fn test_create_and_load_config() {
        let temp_dir = tempdir().unwrap();
        let project_root = temp_dir.path();

        // Create config
        let config_path = create_project_config(project_root).unwrap();
        assert!(config_path.exists());

        // Verify it can be loaded
        let config = load_config(project_root);
        assert!(config.maven_settings.is_none() || config.maven_settings.is_some());
    }

    #[test]
    fn test_find_maven_settings() {
        let temp_dir = tempdir().unwrap();
        let project_root = temp_dir.path();

        // No settings file
        assert!(find_maven_settings(project_root).is_none());

        // Create maven_settings.xml
        let maven_settings = project_root.join("maven_settings.xml");
        fs::write(&maven_settings, "<settings></settings>").unwrap();

        let found = find_maven_settings(project_root);
        assert!(found.is_some());
        assert_eq!(found.unwrap(), maven_settings);
    }

    #[test]
    fn test_find_maven_settings_prefers_maven_settings_xml() {
        let temp_dir = tempdir().unwrap();
        let project_root = temp_dir.path();

        // Create both files
        fs::write(
            project_root.join("maven_settings.xml"),
            "<settings></settings>",
        )
        .unwrap();
        fs::write(project_root.join("settings.xml"), "<settings></settings>").unwrap();

        let found = find_maven_settings(project_root);
        assert!(found.is_some());
        assert!(found.unwrap().ends_with("maven_settings.xml"));
    }
}
