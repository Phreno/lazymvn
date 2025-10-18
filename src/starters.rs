use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Represents a Spring Boot starter (main class)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Starter {
    pub fully_qualified_class_name: String,
    pub label: String,
    pub is_default: bool,
}

impl Starter {
    pub fn new(fqcn: String, label: String, is_default: bool) -> Self {
        Self {
            fully_qualified_class_name: fqcn,
            label,
            is_default,
        }
    }

    /// Get display name for UI
    #[allow(dead_code)]
    pub fn display_name(&self) -> String {
        if self.is_default {
            format!("★ {} ({})", self.label, self.fully_qualified_class_name)
        } else {
            format!("{} ({})", self.label, self.fully_qualified_class_name)
        }
    }
}

/// Manages starters for a project
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StartersCache {
    pub starters: Vec<Starter>,
    pub last_used: Option<String>, // FQCN of last used starter
}

impl StartersCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load starters for a specific project
    pub fn load(project_root: &Path) -> Self {
        let cache_file = get_starters_cache_file(project_root);

        if let Ok(content) = fs::read_to_string(&cache_file)
            && let Ok(cache) = serde_json::from_str(&content) {
                log::debug!("Loaded starters cache from {:?}", cache_file);
                return cache;
            }

        log::debug!("No starters cache found for project");
        Self::new()
    }

    /// Save starters cache for a project
    pub fn save(&self, project_root: &Path) -> Result<(), String> {
        let cache_file = get_starters_cache_file(project_root);

        // Ensure parent directory exists
        if let Some(parent) = cache_file.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create starters directory: {}", e))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize starters: {}", e))?;

        fs::write(&cache_file, content)
            .map_err(|e| format!("Failed to write starters cache: {}", e))?;

        log::info!("Saved {} starters to {:?}", self.starters.len(), cache_file);
        Ok(())
    }

    /// Add a new starter
    pub fn add_starter(&mut self, starter: Starter) {
        // If this is marked as default, unmark others
        if starter.is_default {
            for s in &mut self.starters {
                s.is_default = false;
            }
        }

        self.starters.push(starter);
    }

    /// Remove a starter by FQCN
    pub fn remove_starter(&mut self, fqcn: &str) -> bool {
        let initial_len = self.starters.len();
        self.starters
            .retain(|s| s.fully_qualified_class_name != fqcn);
        initial_len != self.starters.len()
    }

    /// Set a starter as default
    pub fn set_default(&mut self, fqcn: &str) -> bool {
        let mut found = false;
        for starter in &mut self.starters {
            if starter.fully_qualified_class_name == fqcn {
                starter.is_default = true;
                found = true;
            } else {
                starter.is_default = false;
            }
        }
        found
    }

    /// Get the default starter
    pub fn get_default(&self) -> Option<&Starter> {
        self.starters.iter().find(|s| s.is_default)
    }

    /// Get the last used or default starter
    pub fn get_preferred_starter(&self) -> Option<&Starter> {
        // First try last used
        if let Some(ref last_fqcn) = self.last_used
            && let Some(starter) = self
                .starters
                .iter()
                .find(|s| &s.fully_qualified_class_name == last_fqcn)
            {
                return Some(starter);
            }

        // Fall back to default
        self.get_default()
    }

    /// Update last used starter
    pub fn set_last_used(&mut self, fqcn: String) {
        self.last_used = Some(fqcn);
    }
}

/// Get the starters cache file path for a project
fn get_starters_cache_file(project_root: &Path) -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
        .join("lazymvn")
        .join("starters");

    // Create a hash of the project root path for the filename
    let hash = format!(
        "{:x}",
        md5::compute(project_root.to_string_lossy().as_bytes())
    );
    config_dir.join(format!("{}.json", hash))
}

/// Scan project for potential Spring Boot main classes
pub fn find_potential_starters(project_root: &Path) -> Vec<String> {
    let mut candidates = Vec::new();

    log::debug!("Scanning for Spring Boot starters in {:?}", project_root);

    // Look for Java files matching common patterns
    let patterns = vec!["*Application.java", "*Main.java"];

    for pattern in patterns {
        if let Ok(entries) = glob::glob(&format!("{}/**/{}", project_root.display(), pattern)) {
            for entry in entries.flatten() {
                if let Some(fqcn) = extract_fqcn_from_file(&entry) {
                    log::debug!("Found potential starter: {}", fqcn);
                    candidates.push(fqcn);
                }
            }
        }
    }

    // Also scan for files containing SpringApplication.run
    if let Ok(entries) = glob::glob(&format!("{}/**/*.java", project_root.display())) {
        for entry in entries.flatten() {
            if let Ok(content) = fs::read_to_string(&entry)
                && (content.contains("SpringApplication.run")
                    || content.contains("@SpringBootApplication"))
                    && let Some(fqcn) = extract_fqcn_from_file(&entry)
                        && !candidates.contains(&fqcn) {
                            log::debug!("Found Spring Boot class: {}", fqcn);
                            candidates.push(fqcn);
                        }
        }
    }

    log::info!("Found {} potential starters", candidates.len());
    candidates
}

/// Extract fully qualified class name from a Java file
fn extract_fqcn_from_file(file_path: &Path) -> Option<String> {
    let content = fs::read_to_string(file_path).ok()?;

    // Extract package name
    let package = content
        .lines()
        .find(|line| line.trim().starts_with("package "))
        .and_then(|line| {
            line.trim()
                .strip_prefix("package ")?
                .strip_suffix(';')
                .map(|s| s.trim().to_string())
        })?;

    // Extract class name from filename
    let class_name = file_path.file_stem()?.to_str()?.to_string();

    Some(format!("{}.{}", package, class_name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_starter_display_name() {
        let starter = Starter::new(
            "com.example.Application".to_string(),
            "Main API".to_string(),
            false,
        );
        assert_eq!(starter.display_name(), "Main API (com.example.Application)");

        let default_starter = Starter::new(
            "com.example.Application".to_string(),
            "Main API".to_string(),
            true,
        );
        assert!(default_starter.display_name().starts_with("★"));
    }

    #[test]
    fn test_starters_cache_add_and_get() {
        let mut cache = StartersCache::new();

        let starter = Starter::new("com.example.App".to_string(), "Main".to_string(), true);

        cache.add_starter(starter.clone());

        assert_eq!(cache.starters.len(), 1);
        assert_eq!(cache.get_default(), Some(&starter));
    }

    #[test]
    fn test_starters_cache_set_default_unmarks_others() {
        let mut cache = StartersCache::new();

        cache.add_starter(Starter::new(
            "com.example.App1".to_string(),
            "App1".to_string(),
            true,
        ));
        cache.add_starter(Starter::new(
            "com.example.App2".to_string(),
            "App2".to_string(),
            false,
        ));

        cache.set_default("com.example.App2");

        assert!(!cache.starters[0].is_default);
        assert!(cache.starters[1].is_default);
    }

    #[test]
    fn test_starters_cache_remove() {
        let mut cache = StartersCache::new();

        cache.add_starter(Starter::new(
            "com.example.App1".to_string(),
            "App1".to_string(),
            false,
        ));
        cache.add_starter(Starter::new(
            "com.example.App2".to_string(),
            "App2".to_string(),
            false,
        ));

        assert!(cache.remove_starter("com.example.App1"));
        assert_eq!(cache.starters.len(), 1);
        assert!(!cache.remove_starter("com.example.NonExistent"));
    }

    #[test]
    fn test_starters_cache_save_and_load() {
        let temp_dir = tempdir().unwrap();
        let project_root = temp_dir.path();

        let mut cache = StartersCache::new();
        cache.add_starter(Starter::new(
            "com.example.App".to_string(),
            "Main".to_string(),
            true,
        ));
        cache.set_last_used("com.example.App".to_string());

        cache.save(project_root).unwrap();

        let loaded = StartersCache::load(project_root);
        assert_eq!(loaded.starters.len(), 1);
        assert_eq!(loaded.last_used, Some("com.example.App".to_string()));
    }

    #[test]
    fn test_get_preferred_starter_uses_last_used() {
        let mut cache = StartersCache::new();

        cache.add_starter(Starter::new(
            "com.example.App1".to_string(),
            "App1".to_string(),
            true,
        ));
        cache.add_starter(Starter::new(
            "com.example.App2".to_string(),
            "App2".to_string(),
            false,
        ));
        cache.set_last_used("com.example.App2".to_string());

        let preferred = cache.get_preferred_starter().unwrap();
        assert_eq!(preferred.fully_qualified_class_name, "com.example.App2");
    }

    #[test]
    fn test_get_preferred_starter_falls_back_to_default() {
        let mut cache = StartersCache::new();

        cache.add_starter(Starter::new(
            "com.example.App1".to_string(),
            "App1".to_string(),
            true,
        ));
        cache.add_starter(Starter::new(
            "com.example.App2".to_string(),
            "App2".to_string(),
            false,
        ));

        let preferred = cache.get_preferred_starter().unwrap();
        assert_eq!(preferred.fully_qualified_class_name, "com.example.App1");
    }
}
