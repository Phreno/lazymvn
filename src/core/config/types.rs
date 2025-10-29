// Configuration types and structures
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use super::logging::LoggingConfig;

/// Main configuration structure
#[derive(Deserialize, Default, Clone)]
pub struct Config {
    pub maven_settings: Option<String>,
    pub launch_mode: Option<LaunchMode>,
    pub notifications_enabled: Option<bool>,
    pub watch: Option<WatchConfig>,
    pub output: Option<OutputConfig>,
    pub logging: Option<LoggingConfig>,
    pub spring: Option<SpringConfig>,
    pub maven: Option<MavenConfig>,
}

/// Maven configuration for custom arguments
#[derive(Deserialize, Clone, Debug, Default, PartialEq)]
pub struct MavenConfig {
    /// List of custom Maven flags/arguments
    #[serde(default)]
    pub custom_flags: Vec<CustomFlag>,
}

/// Custom Maven flag that can be toggled
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct CustomFlag {
    /// Display name for the flag (e.g., "Custom property")
    pub name: String,
    /// The actual Maven flag (e.g., "-Dmy.property=value")
    pub flag: String,
    /// Whether the flag is enabled by default (optional, default: false)
    #[serde(default)]
    pub enabled: bool,
}

/// Spring Boot configuration overrides
#[derive(Deserialize, Clone, Debug, Default, PartialEq)]
pub struct SpringConfig {
    /// List of Spring Boot properties to override
    #[serde(default)]
    pub properties: Vec<SpringProperty>,

    /// Active profiles (alternative to -Dspring.profiles.active)
    #[serde(default)]
    pub active_profiles: Vec<String>,
}

/// Spring Boot property override
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct SpringProperty {
    /// Property name (e.g., "server.port")
    pub name: String,
    /// Property value (e.g., "8081")
    pub value: String,
}

/// Output buffer configuration
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct OutputConfig {
    /// Maximum number of lines to keep in output buffer (default: 10000)
    #[serde(default = "default_max_lines")]
    pub max_lines: usize,

    /// Maximum number of updates to process per poll cycle (default: 100)
    #[serde(default = "default_max_updates_per_poll")]
    pub max_updates_per_poll: usize,
}

fn default_max_lines() -> usize {
    10_000
}

fn default_max_updates_per_poll() -> usize {
    100
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            max_lines: default_max_lines(),
            max_updates_per_poll: default_max_updates_per_poll(),
        }
    }
}

/// File watching configuration
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct WatchConfig {
    /// Enable file watching (default: false)
    #[serde(default)]
    pub enabled: bool,

    /// Commands that should trigger auto-reload on file changes
    /// Default: ["test", "start"]
    #[serde(default = "default_watch_commands")]
    pub commands: Vec<String>,

    /// File patterns to watch (glob patterns)
    /// Default: ["src/**/*.java", "src/**/*.properties", "src/**/*.xml"]
    #[serde(default = "default_watch_patterns")]
    pub patterns: Vec<String>,

    /// Debounce delay in milliseconds (default: 500ms)
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,
}

fn default_watch_commands() -> Vec<String> {
    vec!["test".to_string(), "start".to_string()]
}

fn default_watch_patterns() -> Vec<String> {
    vec![
        "src/**/*.java".to_string(),
        "src/**/*.properties".to_string(),
        "src/**/*.xml".to_string(),
    ]
}

fn default_debounce_ms() -> u64 {
    500
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            commands: default_watch_commands(),
            patterns: default_watch_patterns(),
            debounce_ms: default_debounce_ms(),
        }
    }
}

/// Launch mode for running Maven applications
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LaunchMode {
    /// Auto-detect: use spring-boot:run if available, fallback to exec:java
    #[default]
    Auto,
    /// Always use spring-boot:run
    ForceRun,
    /// Always use exec:java
    ForceExec,
}

/// Recent projects tracking
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
        let config_dir = super::io::get_config_dir();
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
        let config_dir = super::io::get_config_dir();
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

    #[allow(dead_code)]
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

/// Module preferences for saving selected profiles and flags per module
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModulePreferences {
    pub active_profiles: Vec<String>,
    pub enabled_flags: Vec<String>,
}

/// Maven profiles cache for avoiding slow Maven calls
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfilesCache {
    pub profiles: Vec<String>,
    #[serde(default)]
    pub auto_activated: Vec<String>,
}

impl ProfilesCache {
    /// Load profiles cache for a specific project
    pub fn load(project_root: &Path) -> Option<Self> {
        let cache_file = Self::get_cache_file(project_root);

        if let Ok(content) = fs::read_to_string(&cache_file)
            && let Ok(cache) = serde_json::from_str(&content)
        {
            log::debug!("Loaded profiles cache from {:?}", cache_file);
            return Some(cache);
        }

        log::debug!("No profiles cache found");
        None
    }

    /// Save profiles cache to disk
    pub fn save(&self, project_root: &Path) -> Result<(), String> {
        let cache_file = Self::get_cache_file(project_root);

        if let Some(parent) = cache_file.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create cache directory: {}", e))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize profiles cache: {}", e))?;

        fs::write(&cache_file, content)
            .map_err(|e| format!("Failed to write profiles cache: {}", e))?;

        log::info!(
            "Saved {} profiles to cache at {:?}",
            self.profiles.len(),
            cache_file
        );
        Ok(())
    }

    /// Delete profiles cache for a project
    pub fn invalidate(project_root: &Path) -> Result<(), String> {
        let cache_file = Self::get_cache_file(project_root);
        if cache_file.exists() {
            fs::remove_file(&cache_file)
                .map_err(|e| format!("Failed to delete profiles cache: {}", e))?;
            log::info!("Invalidated profiles cache at {:?}", cache_file);
        }
        Ok(())
    }

    /// Get the cache file path for a project
    fn get_cache_file(project_root: &Path) -> PathBuf {
        let config_dir = super::io::get_config_dir();
        
        // Canonicalize the path to ensure consistent hashing regardless of
        // symlinks, relative paths, etc.
        let canonical_root = project_root
            .canonicalize()
            .unwrap_or_else(|_| project_root.to_path_buf());
        
        let project_hash = format!(
            "{:x}",
            md5::compute(canonical_root.to_string_lossy().as_bytes())
        );
        config_dir
            .join("profiles")
            .join(format!("{}.json", project_hash))
    }
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
        let config_dir = super::io::get_config_dir();
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
mod tests {
    use super::*;
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

        assert_eq!(recent.projects.len(), 1);
        assert_eq!(recent.projects[0], valid_path.to_string_lossy().to_string());
    }

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
