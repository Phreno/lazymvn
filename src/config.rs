use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Default)]
pub struct Config {
    pub maven_settings: Option<String>,
}

pub fn load_config(project_root: &Path) -> Config {
    log::debug!("Loading config from project root: {:?}", project_root);
    let mut config: Config = {
        let config_path = project_root.join("lazymvn.toml");
        log::debug!("Checking for config file at: {:?}", config_path);
        if let Ok(content) = fs::read_to_string(&config_path) {
            log::info!("Found lazymvn.toml, parsing configuration");
            toml::from_str(&content).unwrap_or_default()
        } else {
            log::debug!("No lazymvn.toml found, using defaults");
            Config::default()
        }
    };

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

    config
}

fn find_maven_settings(project_root: &Path) -> Option<PathBuf> {
    let filenames = ["maven_settings.xml", "settings.xml"];
    let dirs = [
        project_root.to_path_buf(),
        dirs::home_dir().unwrap().join(".m2"),
    ];

    log::debug!("Searching for Maven settings in:");
    for dir in &dirs {
        log::debug!("  - {:?}", dir);
        for filename in &filenames {
            let path = dir.join(filename);
            if path.exists() {
                log::info!("Found Maven settings at: {:?}", path);
                return Some(path);
            }
        }
    }

    log::debug!("No Maven settings file found");
    None
}
