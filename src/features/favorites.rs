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
        let module_display = format_module_display(&self.module);
        format!("{} → [{}] {}", self.name, module_display, self.goal)
    }
}

/// Format module name for display
fn format_module_display(module: &str) -> String {
    if module == "." {
        "(root)".to_string()
    } else {
        module.to_string()
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
        let favorites = load_favorites_from_file(&file_path);

        Self {
            favorites,
            file_path,
        }
    }

    /// Add a favorite
    pub fn add(&mut self, favorite: Favorite) {
        add_or_update_favorite(&mut self.favorites, favorite);
        self.save();
    }

    /// Remove a favorite by index
    pub fn remove(&mut self, index: usize) -> Option<Favorite> {
        remove_favorite_at_index(&mut self.favorites, index).inspect(|removed| {
            log::info!("Removed favorite: {}", removed.name);
            self.save();
        })
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

/// Load favorites from file
fn load_favorites_from_file(file_path: &PathBuf) -> Vec<Favorite> {
    if !file_path.exists() {
        return Vec::new();
    }

    match fs::read_to_string(file_path) {
        Ok(contents) => parse_favorites_json(&contents),
        Err(e) => {
            log::warn!("Failed to read favorites: {}", e);
            Vec::new()
        }
    }
}

/// Parse favorites from JSON string
fn parse_favorites_json(json: &str) -> Vec<Favorite> {
    serde_json::from_str(json).unwrap_or_else(|e| {
        log::warn!("Failed to parse favorites: {}", e);
        Vec::new()
    })
}

/// Add or update a favorite in the list
fn add_or_update_favorite(favorites: &mut Vec<Favorite>, favorite: Favorite) {
    if let Some(existing) = find_favorite_by_name(favorites, &favorite.name) {
        *existing = favorite;
        log::info!("Updated existing favorite: {}", existing.name);
    } else {
        favorites.push(favorite);
        log::info!("Added new favorite");
    }
}

/// Find a favorite by name (mutable)
fn find_favorite_by_name<'a>(favorites: &'a mut [Favorite], name: &str) -> Option<&'a mut Favorite> {
    favorites.iter_mut().find(|f| f.name == name)
}

/// Remove a favorite at the given index
fn remove_favorite_at_index(favorites: &mut Vec<Favorite>, index: usize) -> Option<Favorite> {
    if is_valid_index(favorites.len(), index) {
        Some(favorites.remove(index))
    } else {
        None
    }
}

/// Check if index is valid for the given length
fn is_valid_index(len: usize, index: usize) -> bool {
    index < len
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

    #[test]
    fn favorites_remove_invalid_index() {
        let mut favorites = Favorites::default();
        favorites.add(Favorite::new(
            "Test".to_string(),
            "module".to_string(),
            "test".to_string(),
            vec![],
            vec![],
        ));

        let removed = favorites.remove(10);
        assert!(removed.is_none());
        assert_eq!(favorites.list().len(), 1);
    }

    #[test]
    fn favorites_is_empty() {
        let favorites = Favorites::default();
        assert!(favorites.is_empty());

        let mut favorites = Favorites::default();
        favorites.add(Favorite::new(
            "Test".to_string(),
            "module".to_string(),
            "test".to_string(),
            vec![],
            vec![],
        ));
        assert!(!favorites.is_empty());
    }

    #[test]
    fn favorites_clear() {
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
        favorites.clear();
        assert!(favorites.is_empty());
    }

    #[test]
    fn favorite_with_profiles_and_flags() {
        let fav = Favorite::new(
            "Complex Build".to_string(),
            "api".to_string(),
            "clean install".to_string(),
            vec!["prod".to_string(), "secure".to_string()],
            vec!["-DskipTests".to_string(), "-U".to_string()],
        );

        assert_eq!(fav.profiles.len(), 2);
        assert_eq!(fav.flags.len(), 2);
        assert!(fav.profiles.contains(&"prod".to_string()));
        assert!(fav.flags.contains(&"-DskipTests".to_string()));
    }

    #[test]
    fn test_format_module_display_root() {
        assert_eq!(format_module_display("."), "(root)");
    }

    #[test]
    fn test_format_module_display_named() {
        assert_eq!(format_module_display("my-module"), "my-module");
    }

    #[test]
    fn test_parse_favorites_json_valid() {
        let json = r#"[{"name":"Test","module":".","goal":"test","profiles":[],"flags":[]}]"#;
        let favorites = parse_favorites_json(json);
        assert_eq!(favorites.len(), 1);
        assert_eq!(favorites[0].name, "Test");
    }

    #[test]
    fn test_parse_favorites_json_invalid() {
        let json = "invalid json";
        let favorites = parse_favorites_json(json);
        assert_eq!(favorites.len(), 0);
    }

    #[test]
    fn test_is_valid_index() {
        assert!(is_valid_index(5, 0));
        assert!(is_valid_index(5, 4));
        assert!(!is_valid_index(5, 5));
        assert!(!is_valid_index(5, 10));
    }

    #[test]
    fn test_remove_favorite_at_index_valid() {
        let mut favorites = vec![
            Favorite::new("A".to_string(), ".".to_string(), "test".to_string(), vec![], vec![]),
            Favorite::new("B".to_string(), ".".to_string(), "test".to_string(), vec![], vec![]),
        ];
        
        let removed = remove_favorite_at_index(&mut favorites, 0);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().name, "A");
        assert_eq!(favorites.len(), 1);
    }

    #[test]
    fn test_remove_favorite_at_index_invalid() {
        let mut favorites = vec![
            Favorite::new("A".to_string(), ".".to_string(), "test".to_string(), vec![], vec![]),
        ];
        
        let removed = remove_favorite_at_index(&mut favorites, 10);
        assert!(removed.is_none());
        assert_eq!(favorites.len(), 1);
    }
}
