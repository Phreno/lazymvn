use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// A favorite command bookmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Favorite {
    pub name: String,
    pub module: String,
    pub goal: String,
    pub profiles: Vec<String>,
    pub flags: Vec<String>,
}

impl Favorite {
    /// Create a new favorite
    pub fn new(
        name: String,
        module: String,
        goal: String,
        profiles: Vec<String>,
        flags: Vec<String>,
    ) -> Self {
        Self {
            name,
            module,
            goal,
            profiles,
            flags,
        }
    }

    /// Format the favorite for display in list
    pub fn format_summary(&self) -> String {
        let module_display = if self.module == "." {
            "(root)".to_string()
        } else {
            self.module.clone()
        };
        
        format!("{} → [{}] {}", self.name, module_display, self.goal)
    }
}

/// Favorites manager
#[derive(Debug, Default)]
pub struct Favorites {
    favorites: Vec<Favorite>,
    file_path: PathBuf,
}

impl Favorites {
    /// Load favorites from disk
    pub fn load() -> Self {
        let file_path = Self::get_favorites_file_path();
        
        let favorites = if file_path.exists() {
            match fs::read_to_string(&file_path) {
                Ok(contents) => {
                    serde_json::from_str(&contents).unwrap_or_else(|e| {
                        log::warn!("Failed to parse favorites: {}", e);
                        Vec::new()
                    })
                }
                Err(e) => {
                    log::warn!("Failed to read favorites: {}", e);
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };

        Self { favorites, file_path }
    }

    /// Add a favorite
    pub fn add(&mut self, favorite: Favorite) {
        // Check if name already exists
        if let Some(existing) = self.favorites.iter_mut().find(|f| f.name == favorite.name) {
            // Replace existing favorite with same name
            *existing = favorite;
            log::info!("Updated existing favorite: {}", existing.name);
        } else {
            // Add new favorite
            self.favorites.push(favorite);
            log::info!("Added new favorite");
        }
        
        self.save();
    }

    /// Remove a favorite by index
    pub fn remove(&mut self, index: usize) -> Option<Favorite> {
        if index < self.favorites.len() {
            let removed = self.favorites.remove(index);
            log::info!("Removed favorite: {}", removed.name);
            self.save();
            Some(removed)
        } else {
            None
        }
    }

    /// Get all favorites
    pub fn list(&self) -> &[Favorite] {
        &self.favorites
    }

    /// Check if favorites list is empty
    pub fn is_empty(&self) -> bool {
        self.favorites.is_empty()
    }

    /// Save favorites to disk
    fn save(&self) {
        // Ensure directory exists
        if let Some(parent) = self.file_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        match serde_json::to_string_pretty(&self.favorites) {
            Ok(json) => {
                if let Err(e) = fs::write(&self.file_path, json) {
                    log::error!("Failed to save favorites: {}", e);
                }
            }
            Err(e) => {
                log::error!("Failed to serialize favorites: {}", e);
            }
        }
    }

    /// Get the path to the favorites file
    fn get_favorites_file_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("lazymvn");
        config_dir.join("favorites.json")
    }

    /// Clear all favorites
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.favorites.clear();
        self.save();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn favorite_format_summary_with_root_module() {
        let fav = Favorite::new(
            "Quick Build".to_string(),
            ".".to_string(),
            "clean install".to_string(),
            vec![],
            vec![],
        );

        let summary = fav.format_summary();
        assert!(summary.contains("Quick Build"));
        assert!(summary.contains("(root)"));
        assert!(summary.contains("clean install"));
    }

    #[test]
    fn favorite_format_summary_with_named_module() {
        let fav = Favorite::new(
            "Build API".to_string(),
            "api-module".to_string(),
            "package".to_string(),
            vec!["dev".to_string()],
            vec!["-DskipTests".to_string()],
        );

        let summary = fav.format_summary();
        assert!(summary.contains("Build API"));
        assert!(summary.contains("api-module"));
        assert!(summary.contains("package"));
    }

    #[test]
    fn favorites_add_and_list() {
        let mut favorites = Favorites::default();
        
        favorites.add(Favorite::new(
            "Test 1".to_string(),
            "module1".to_string(),
            "test".to_string(),
            vec![],
            vec![],
        ));
        
        favorites.add(Favorite::new(
            "Test 2".to_string(),
            "module2".to_string(),
            "package".to_string(),
            vec![],
            vec![],
        ));

        assert_eq!(favorites.list().len(), 2);
        assert_eq!(favorites.list()[0].name, "Test 1");
        assert_eq!(favorites.list()[1].name, "Test 2");
    }

    #[test]
    fn favorites_replace_existing_name() {
        let mut favorites = Favorites::default();
        
        favorites.add(Favorite::new(
            "Build".to_string(),
            "module1".to_string(),
            "test".to_string(),
            vec![],
            vec![],
        ));
        
        favorites.add(Favorite::new(
            "Build".to_string(),
            "module2".to_string(),
            "package".to_string(),
            vec![],
            vec![],
        ));

        assert_eq!(favorites.list().len(), 1);
        assert_eq!(favorites.list()[0].module, "module2");
        assert_eq!(favorites.list()[0].goal, "package");
    }

    #[test]
    fn favorites_remove_by_index() {
        let mut favorites = Favorites::default();
        
        favorites.add(Favorite::new(
            "Test 1".to_string(),
            "module1".to_string(),
            "test".to_string(),
            vec![],
            vec![],
        ));
        
        favorites.add(Favorite::new(
            "Test 2".to_string(),
            "module2".to_string(),
            "package".to_string(),
            vec![],
            vec![],
        ));

        let removed = favorites.remove(0);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().name, "Test 1");
        assert_eq!(favorites.list().len(), 1);
        assert_eq!(favorites.list()[0].name, "Test 2");
    }
}
