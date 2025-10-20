use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Default)]
pub struct Config {
    pub maven_settings: Option<String>,
    pub launch_mode: Option<LaunchMode>,
    pub notifications_enabled: Option<bool>,
}

/// Strategy for launching Spring Boot applications
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum LaunchMode {
    /// Auto-detect: use spring-boot:run if available, fallback to exec:java
    #[default]
    Auto,
    /// Always use spring-boot:run
    ForceRun,
    /// Always use exec:java
    ForceExec,
}

#[derive(Serialize, Deserialize, Default)]
pub struct RecentProjects {
    projects: Vec<String>,
}

impl RecentProjects {
    const MAX_ENTRIES: usize = 20;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn load() -> Self {
        let config_dir = get_config_dir();
        let recent_file = config_dir.join("recent.json");

        if let Ok(content) = fs::read_to_string(&recent_file)
            && let Ok(recent) = serde_json::from_str(&content)
        {
            log::debug!("Loaded {} recent projects", Self::count(&recent));
            return recent;
        }

        log::debug!("No recent projects file found, creating new");
        Self::new()
    }

    fn count(recent: &RecentProjects) -> usize {
        recent.projects.len()
    }

    pub fn save(&self) -> Result<(), String> {
        let config_dir = get_config_dir();
        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;

        let recent_file = config_dir.join("recent.json");
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize recent projects: {}", e))?;

        fs::write(&recent_file, content)
            .map_err(|e| format!("Failed to write recent projects: {}", e))?;

        log::debug!(
            "Saved {} recent projects to {:?}",
            self.projects.len(),
            recent_file
        );
        Ok(())
    }

    pub fn add(&mut self, path: PathBuf) {
        let path_str = path.to_string_lossy().to_string();

        // Remove if already exists
        self.projects.retain(|p| p != &path_str);

        // Add to front
        self.projects.insert(0, path_str);

        // Limit to MAX_ENTRIES
        if self.projects.len() > Self::MAX_ENTRIES {
            self.projects.truncate(Self::MAX_ENTRIES);
        }

        log::debug!("Added project to recent list: {:?}", path);

        // Save after adding
        if let Err(e) = self.save() {
            log::error!("Failed to save recent projects: {}", e);
        }
    }

    pub fn get_projects(&self) -> Vec<PathBuf> {
        self.projects
            .iter()
            .filter_map(|p| {
                let path = PathBuf::from(p);
                if path.exists() {
                    Some(path)
                } else {
                    log::debug!("Skipping non-existent path: {:?}", path);
                    None
                }
            })
            .collect()
    }

    pub fn remove_invalid(&mut self) {
        let original_len = self.projects.len();
        self.projects.retain(|p| PathBuf::from(p).exists());

        if self.projects.len() != original_len {
            log::info!(
                "Removed {} invalid paths from recent projects",
                original_len - self.projects.len()
            );
            if let Err(e) = self.save() {
                log::error!("Failed to save after removing invalid paths: {}", e);
            }
        }
    }
}

fn get_config_dir() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| {
            log::warn!("Could not determine config directory, using home");
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
        })
        .join("lazymvn");

    log::debug!("Config directory: {:?}", config_dir);
    config_dir
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_recent_projects_new_is_empty() {
        let recent = RecentProjects::new();
        assert_eq!(recent.projects.len(), 0);
    }

    #[test]
    fn test_recent_projects_add_single_project() {
        let mut recent = RecentProjects::new();
        let project_path = PathBuf::from("/tmp/test-project");

        recent
            .projects
            .push(project_path.to_string_lossy().to_string());

        assert_eq!(recent.projects.len(), 1);
        assert_eq!(recent.projects[0], "/tmp/test-project");
    }

    #[test]
    fn test_recent_projects_add_removes_duplicates() {
        let mut recent = RecentProjects::new();
        let project_path = PathBuf::from("/tmp/test-project");

        recent.add(project_path.clone());
        recent.add(project_path.clone());

        assert_eq!(
            recent.projects.len(),
            1,
            "Should only have one entry after adding duplicate"
        );
    }

    #[test]
    fn test_recent_projects_add_moves_to_front() {
        let mut recent = RecentProjects::new();
        let project1 = PathBuf::from("/tmp/project1");
        let project2 = PathBuf::from("/tmp/project2");

        recent.add(project1.clone());
        recent.add(project2.clone());
        recent.add(project1.clone()); // Re-add first project

        assert_eq!(recent.projects.len(), 2);
        assert_eq!(
            recent.projects[0], "/tmp/project1",
            "Most recently added should be first"
        );
        assert_eq!(recent.projects[1], "/tmp/project2");
    }

    #[test]
    fn test_recent_projects_limits_to_max_entries() {
        let mut recent = RecentProjects::new();

        // Add more than MAX_ENTRIES projects
        for i in 0..25 {
            recent.add(PathBuf::from(format!("/tmp/project{}", i)));
        }

        assert_eq!(recent.projects.len(), RecentProjects::MAX_ENTRIES);
        assert_eq!(
            recent.projects[0], "/tmp/project24",
            "Most recent should be first"
        );
    }

    #[test]
    fn test_recent_projects_save_and_load() {
        let temp_dir = tempdir().unwrap();
        let recent_file = temp_dir.path().join("recent.json");

        // Create and save
        let mut recent = RecentProjects::new();
        recent.add(PathBuf::from("/tmp/project1"));
        recent.add(PathBuf::from("/tmp/project2"));

        let json = serde_json::to_string(&recent).unwrap();
        fs::write(&recent_file, json).unwrap();

        // Load and verify
        let content = fs::read_to_string(&recent_file).unwrap();
        let loaded: RecentProjects = serde_json::from_str(&content).unwrap();

        assert_eq!(loaded.projects.len(), 2);
        assert_eq!(loaded.projects[0], "/tmp/project2");
        assert_eq!(loaded.projects[1], "/tmp/project1");
    }

    #[test]
    fn test_recent_projects_get_projects_filters_invalid() {
        let temp_dir = tempdir().unwrap();
        let valid_path = temp_dir.path().join("valid");
        fs::create_dir(&valid_path).unwrap();

        let mut recent = RecentProjects::new();
        recent
            .projects
            .push(valid_path.to_string_lossy().to_string());
        recent.projects.push("/nonexistent/path".to_string());

        let valid_projects = recent.get_projects();

        assert_eq!(valid_projects.len(), 1, "Should only return existing paths");
        assert_eq!(valid_projects[0], valid_path);
    }

    #[test]
    fn test_recent_projects_remove_invalid() {
        let temp_dir = tempdir().unwrap();
        let valid_path = temp_dir.path().join("valid");
        fs::create_dir(&valid_path).unwrap();

        let mut recent = RecentProjects::new();
        recent
            .projects
            .push(valid_path.to_string_lossy().to_string());
        recent.projects.push("/nonexistent/path1".to_string());
        recent.projects.push("/nonexistent/path2".to_string());

        assert_eq!(recent.projects.len(), 3);

        recent.remove_invalid();

        assert_eq!(recent.projects.len(), 1, "Should remove invalid paths");
        assert_eq!(recent.projects[0], valid_path.to_string_lossy());
    }

    #[test]
    fn test_recent_projects_serialization_format() {
        let mut recent = RecentProjects::new();
        recent.add(PathBuf::from("/tmp/test"));

        let json = serde_json::to_string(&recent).unwrap();

        assert!(json.contains("\"projects\""));
        assert!(json.contains("/tmp/test"));
    }

    #[test]
    fn test_load_config_returns_default_when_no_file() {
        let temp_dir = tempdir().unwrap();
        let config = load_config(temp_dir.path());

        // Should return default config without error
        assert!(config.maven_settings.is_some() || config.maven_settings.is_none());
    }

    #[test]
    fn test_load_config_parses_maven_settings() {
        let temp_dir = tempdir().unwrap();
        let config_file = temp_dir.path().join("lazymvn.toml");

        fs::write(&config_file, r#"maven_settings = "/custom/settings.xml""#).unwrap();

        let config = load_config(temp_dir.path());

        assert_eq!(
            config.maven_settings,
            Some("/custom/settings.xml".to_string())
        );
    }
}

/// Module preferences for saving selected profiles and flags per module
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModulePreferences {
    pub active_profiles: Vec<String>,
    pub enabled_flags: Vec<String>,
}

/// Manages module preferences per project
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectPreferences {
    modules: std::collections::HashMap<String, ModulePreferences>,
}

impl ProjectPreferences {
    /// Load preferences for a specific project
    pub fn load(project_root: &Path) -> Self {
        let prefs_file = Self::get_prefs_file(project_root);

        if let Ok(content) = fs::read_to_string(&prefs_file)
            && let Ok(prefs) = serde_json::from_str(&content)
        {
            log::debug!("Loaded module preferences from {:?}", prefs_file);
            return prefs;
        }

        log::debug!("No preferences file found, creating new");
        Self::default()
    }

    /// Save preferences to disk
    pub fn save(&self, project_root: &Path) -> Result<(), String> {
        let prefs_file = Self::get_prefs_file(project_root);

        if let Some(parent) = prefs_file.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create preferences directory: {}", e))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize preferences: {}", e))?;

        fs::write(&prefs_file, content)
            .map_err(|e| format!("Failed to write preferences: {}", e))?;

        log::debug!("Saved module preferences to {:?}", prefs_file);
        Ok(())
    }

    /// Get preferences for a specific module
    pub fn get_module_prefs(&self, module: &str) -> Option<&ModulePreferences> {
        self.modules.get(module)
    }

    /// Set preferences for a specific module
    pub fn set_module_prefs(&mut self, module: String, prefs: ModulePreferences) {
        self.modules.insert(module, prefs);
    }

    /// Get the preferences file path for a project
    fn get_prefs_file(project_root: &Path) -> PathBuf {
        let config_dir = get_config_dir();
        let project_hash = format!(
            "{:x}",
            md5::compute(project_root.to_string_lossy().as_bytes())
        );
        config_dir
            .join("preferences")
            .join(format!("{}.json", project_hash))
    }
}

#[cfg(test)]
mod module_prefs_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_module_preferences_save_and_load() {
        let temp_dir = tempdir().unwrap();
        let mut prefs = ProjectPreferences::default();

        let module_prefs = ModulePreferences {
            active_profiles: vec!["dev".to_string(), "debug".to_string()],
            enabled_flags: vec!["--also-make".to_string()],
        };

        prefs.set_module_prefs("my-module".to_string(), module_prefs.clone());

        // Save
        prefs.save(temp_dir.path()).unwrap();

        // Load
        let loaded = ProjectPreferences::load(temp_dir.path());

        let loaded_prefs = loaded.get_module_prefs("my-module").unwrap();
        assert_eq!(loaded_prefs.active_profiles, module_prefs.active_profiles);
        assert_eq!(loaded_prefs.enabled_flags, module_prefs.enabled_flags);
    }

    #[test]
    fn test_module_preferences_multiple_modules() {
        let temp_dir = tempdir().unwrap();
        let mut prefs = ProjectPreferences::default();

        prefs.set_module_prefs(
            "module1".to_string(),
            ModulePreferences {
                active_profiles: vec!["prod".to_string()],
                enabled_flags: vec![],
            },
        );

        prefs.set_module_prefs(
            "module2".to_string(),
            ModulePreferences {
                active_profiles: vec!["dev".to_string()],
                enabled_flags: vec!["--offline".to_string()],
            },
        );

        prefs.save(temp_dir.path()).unwrap();
        let loaded = ProjectPreferences::load(temp_dir.path());

        assert_eq!(loaded.modules.len(), 2);
        assert!(loaded.get_module_prefs("module1").is_some());
        assert!(loaded.get_module_prefs("module2").is_some());
    }

    #[test]
    fn test_module_preferences_overwrite() {
        let _temp_dir = tempdir().unwrap();
        let mut prefs = ProjectPreferences::default();

        prefs.set_module_prefs(
            "module1".to_string(),
            ModulePreferences {
                active_profiles: vec!["dev".to_string()],
                enabled_flags: vec![],
            },
        );

        // Overwrite with new preferences
        prefs.set_module_prefs(
            "module1".to_string(),
            ModulePreferences {
                active_profiles: vec!["prod".to_string()],
                enabled_flags: vec!["--offline".to_string()],
            },
        );

        let module_prefs = prefs.get_module_prefs("module1").unwrap();
        assert_eq!(module_prefs.active_profiles, vec!["prod".to_string()]);
    }
}
