use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Default)]
pub struct Config {
    pub maven_settings: Option<String>,
}

pub fn load_config(project_root: &Path) -> Config {
    let mut config: Config = {
        let config_path = project_root.join("lazymvn.toml");
        if let Ok(content) = fs::read_to_string(config_path) {
            toml::from_str(&content).unwrap_or_default()
        } else {
            Config::default()
        }
    };

    if config.maven_settings.is_none() {
        config.maven_settings = find_maven_settings(project_root).map(|p| p.to_str().unwrap().to_string());
    }

    config
}

fn find_maven_settings(project_root: &Path) -> Option<PathBuf> {
    let filenames = ["maven_settings.xml", "settings.xml"];
    let dirs = [
        project_root.to_path_buf(),
        dirs::home_dir().unwrap().join(".m2"),
    ];

    for dir in &dirs {
        for filename in &filenames {
            let path = dir.join(filename);
            if path.exists() {
                return Some(path);
            }
        }
    }

    None
}
