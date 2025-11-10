use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Cache {
    pub project_root: PathBuf,
    pub modules: Vec<String>,
    #[serde(default)]
    pub pom_hash: Option<u64>,
}

/// Save cache to disk
pub fn save_cache(path: &Path, cache: &Cache) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string(cache)?;
    fs::write(path, json.as_bytes())?;
    Ok(())
}

/// Load cache from disk
pub fn load_cache(path: &Path) -> Result<Cache, Box<dyn std::error::Error>> {
    let json = fs::read_to_string(path)?;
    let cache: Cache = serde_json::from_str(&json)?;
    Ok(cache)
}

/// Get cache directory and cache file path
pub fn get_cache_paths() -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let cache_dir = home_dir.join(".config/lazymvn");
    let cache_path = cache_dir.join("cache.json");
    log::debug!("Cache path: {:?}", cache_path);
    Ok((cache_dir, cache_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn cache_save_and_load() {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("cache.json");

        let cache_to_save = Cache {
            project_root: PathBuf::from("/tmp"),
            modules: vec!["module1".to_string(), "module2".to_string()],
            pom_hash: Some(42),
        };

        save_cache(&cache_path, &cache_to_save).unwrap();

        let loaded_cache = load_cache(&cache_path).unwrap();
        assert_eq!(cache_to_save, loaded_cache);
    }

    #[test]
    fn cache_handles_missing_pom_hash() {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("cache.json");

        let cache_to_save = Cache {
            project_root: PathBuf::from("/tmp"),
            modules: vec!["module1".to_string()],
            pom_hash: None,
        };

        save_cache(&cache_path, &cache_to_save).unwrap();
        let loaded_cache = load_cache(&cache_path).unwrap();
        assert_eq!(loaded_cache.pom_hash, None);
    }
}
