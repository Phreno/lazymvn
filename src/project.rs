use quick_xml::Reader;
use quick_xml::events::Event;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

pub fn find_pom() -> Option<PathBuf> {
    let mut current_dir = std::env::current_dir().ok()?;
    log::debug!("Searching for pom.xml starting from: {:?}", current_dir);
    loop {
        let pom_path = current_dir.join("pom.xml");
        log::debug!("Checking path: {:?}", pom_path);
        if pom_path.exists() {
            log::info!("Found pom.xml at: {:?}", pom_path);
            return Some(pom_path);
        }
        if !current_dir.pop() {
            log::warn!("pom.xml not found in any parent directory");
            return None;
        }
    }
}

fn parse_modules_from_str(content: &str) -> Vec<String> {
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut modules = Vec::new();
    let mut in_module = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                if e.name().as_ref() == b"module" {
                    in_module = true;
                }
            }
            Ok(Event::Text(e)) => {
                if in_module {
                    if let Ok(text) = e.decode() {
                        modules.push(text.to_string());
                    }
                    in_module = false; // Reset after getting the text
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => (),
        }
        buf.clear();
    }

    modules
}

fn compute_pom_hash(content: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

pub fn get_project_modules() -> Result<(Vec<String>, PathBuf), Box<dyn std::error::Error>> {
    log::debug!("get_project_modules: Starting module discovery");
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let cache_dir = home_dir.join(".config/lazymvn");
    let cache_path = cache_dir.join("cache.json");
    log::debug!("Cache path: {:?}", cache_path);

    let cached_entry = if cache_path.exists() {
        log::debug!("Cache file exists, attempting to load");
        cache::load_cache(&cache_path).ok()
    } else {
        log::debug!("No cache file found");
        None
    };

    let current_dir = std::env::current_dir()?;
    log::debug!("Current directory: {:?}", current_dir);

    if let Some(cache) = cached_entry.as_ref() {
        log::debug!("Checking cached project root: {:?}", cache.project_root);
        if current_dir.starts_with(&cache.project_root) {
            let pom_path = cache.project_root.join("pom.xml");
            if let Ok(pom_content) = fs::read_to_string(&pom_path) {
                let pom_hash = compute_pom_hash(&pom_content);
                log::debug!(
                    "Current pom hash: {}, cached hash: {:?}",
                    pom_hash,
                    cache.pom_hash
                );
                if cache.pom_hash == Some(pom_hash) {
                    log::info!(
                        "Using cached modules (hash match): {} modules",
                        cache.modules.len()
                    );
                    let modules = normalize_modules(cache.modules.clone());
                    return Ok((modules, cache.project_root.clone()));
                }

                log::info!("POM changed, reparsing modules");
                let modules = normalize_modules(parse_modules_from_str(&pom_content));
                log::debug!("Parsed {} modules from updated POM", modules.len());
                fs::create_dir_all(&cache_dir)?;
                let updated_cache = cache::Cache {
                    project_root: cache.project_root.clone(),
                    modules: modules.clone(),
                    pom_hash: Some(pom_hash),
                };
                cache::save_cache(&cache_path, &updated_cache)?;
                log::debug!("Updated cache saved");
                return Ok((modules, cache.project_root.clone()));
            } else {
                log::warn!("Could not read POM, using cached modules");
                let modules = normalize_modules(cache.modules.clone());
                return Ok((modules, cache.project_root.clone()));
            }
        } else {
            log::debug!("Current dir not under cached project root, discovering new project");
        }
    }

    log::debug!("No valid cache, searching for pom.xml");
    let pom_path = find_pom().ok_or("pom.xml not found")?;
    let project_root = pom_path.parent().unwrap().to_path_buf();
    log::info!("Discovered project root: {:?}", project_root);

    let pom_content = fs::read_to_string(&pom_path).unwrap_or_default();
    let modules = normalize_modules(parse_modules_from_str(&pom_content));
    log::debug!("Parsed {} modules from POM", modules.len());
    for (i, module) in modules.iter().enumerate() {
        log::debug!("  Module {}: {}", i + 1, module);
    }

    let pom_hash = compute_pom_hash(&pom_content);
    log::debug!("Computed POM hash: {}", pom_hash);

    fs::create_dir_all(&cache_dir)?;
    let new_cache = cache::Cache {
        project_root: project_root.clone(),
        modules: modules.clone(),
        pom_hash: Some(pom_hash),
    };
    cache::save_cache(&cache_path, &new_cache)?;
    log::debug!("New cache saved");

    Ok((modules, project_root))
}

/// Get project modules for a specific project path
/// This is used when opening projects in tabs
pub fn get_project_modules_for_path(
    project_path: &PathBuf,
) -> Result<(Vec<String>, PathBuf), Box<dyn std::error::Error>> {
    log::debug!(
        "get_project_modules_for_path: Loading project from {:?}",
        project_path
    );

    // Find pom.xml in the given path
    let pom_path = project_path.join("pom.xml");
    if !pom_path.exists() {
        // Try to find it by walking up
        let mut current = project_path.clone();
        loop {
            let test_pom = current.join("pom.xml");
            if test_pom.exists() {
                log::debug!("Found pom.xml at: {:?}", test_pom);
                let pom_content = fs::read_to_string(&test_pom)?;
                let modules = normalize_modules(parse_modules_from_str(&pom_content));
                let project_root = test_pom.parent().unwrap().to_path_buf();
                return Ok((modules, project_root));
            }

            if !current.pop() {
                break;
            }
        }
        return Err("pom.xml not found in project path or parent directories".into());
    }

    // Read and parse pom.xml
    let pom_content = fs::read_to_string(&pom_path)?;
    let modules = normalize_modules(parse_modules_from_str(&pom_content));
    let project_root = pom_path.parent().unwrap().to_path_buf();

    log::info!(
        "Loaded {} modules from project: {:?}",
        modules.len(),
        project_root
    );

    Ok((modules, project_root))
}

fn normalize_modules(modules: Vec<String>) -> Vec<String> {
    if modules.is_empty() {
        log::info!("No modules found, treating as single-module project");
        vec![".".to_string()]
    } else {
        modules
    }
}

pub mod cache {
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

    pub fn save_cache(path: &Path, cache: &Cache) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string(cache)?;
        fs::write(path, json.as_bytes())?;
        Ok(())
    }

    pub fn load_cache(path: &Path) -> Result<Cache, Box<dyn std::error::Error>> {
        let json = fs::read_to_string(path)?;
        let cache: Cache = serde_json::from_str(&json)?;
        Ok(cache)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs::File;
    use std::path::Path;
    use std::sync::{Mutex, OnceLock};
    use tempfile::tempdir;

    fn home_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn set_home(path: &Path) {
        unsafe { env::set_var("HOME", path) };
    }

    fn restore_home(original: Option<String>) {
        unsafe {
            if let Some(value) = original {
                env::set_var("HOME", value);
            } else {
                env::remove_var("HOME");
            }
        }
    }

    #[test]
    fn find_pom_in_current_dir() {
        let dir = tempdir().unwrap();
        let pom_path = dir.path().join("pom.xml");
        File::create(&pom_path).unwrap();

        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(dir.path()).unwrap();

        assert_eq!(find_pom(), Some(pom_path));

        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn find_pom_in_parent_dir() {
        let dir = tempdir().unwrap();
        let pom_path = dir.path().join("pom.xml");
        File::create(&pom_path).unwrap();

        let subdir = dir.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();

        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(&subdir).unwrap();

        assert_eq!(find_pom(), Some(pom_path));

        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn get_modules_from_pom() {
        let dir = tempdir().unwrap();
        let pom_path = dir.path().join("pom.xml");
        let mut pom_file = File::create(&pom_path).unwrap();
        use std::io::Write;
        pom_file.write_all(b"<project><modules><module>module1</module><module>module2</module></modules></project>").unwrap();

        let content = std::fs::read_to_string(&pom_path).unwrap();
        let modules = parse_modules_from_str(&content);
        assert_eq!(modules, vec!["module1", "module2"]);
    }

    #[test]
    fn get_modules_from_pom_without_modules() {
        let dir = tempdir().unwrap();
        let pom_path = dir.path().join("pom.xml");
        let mut pom_file = File::create(&pom_path).unwrap();
        use std::io::Write;
        pom_file
            .write_all(
                b"<project><groupId>com.example</groupId><artifactId>simple</artifactId></project>",
            )
            .unwrap();

        let content = std::fs::read_to_string(&pom_path).unwrap();
        let modules = parse_modules_from_str(&content);
        assert_eq!(modules, Vec::<String>::new());
    }

    #[test]
    fn normalize_modules_returns_dot_for_empty() {
        let empty_modules = vec![];
        let normalized = super::normalize_modules(empty_modules);
        assert_eq!(normalized, vec!["."]);
    }

    #[test]
    fn normalize_modules_preserves_non_empty() {
        let modules = vec!["module1".to_string(), "module2".to_string()];
        let normalized = super::normalize_modules(modules.clone());
        assert_eq!(normalized, modules);
    }

    #[test]
    fn cache_save_and_load() {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("cache.json");

        let cache_to_save = cache::Cache {
            project_root: PathBuf::from("/tmp"),
            modules: vec!["module1".to_string(), "module2".to_string()],
            pom_hash: Some(42),
        };

        cache::save_cache(&cache_path, &cache_to_save).unwrap();

        let loaded_cache = cache::load_cache(&cache_path).unwrap();
        assert_eq!(cache_to_save, loaded_cache);
    }

    #[test]
    #[cfg(unix)] // HOME environment variable handling differs on Windows
    fn get_project_modules_integration_test() {
        let _guard = home_lock().lock().unwrap();
        // 1. Setup temp project and home directory
        let project_dir = tempdir().unwrap();
        let home_dir = tempdir().unwrap();
        let original_home = env::var("HOME").ok();
        set_home(home_dir.path());

        // 2. Create pom.xml in project
        let pom_path = project_dir.path().join("pom.xml");
        let mut pom_file = File::create(&pom_path).unwrap();
        use std::io::Write;
        pom_file.write_all(b"<project><modules><module>module1</module><module>module2</module></modules></project>").unwrap();
        let pom_content = std::fs::read_to_string(&pom_path).unwrap();
        let expected_hash = super::compute_pom_hash(&pom_content);

        // 3. Set current dir to project
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(project_dir.path()).unwrap();

        // 4. Call get_project_modules for the first time
        let (modules, project_root) = get_project_modules().unwrap();
        assert_eq!(modules, vec!["module1", "module2"]);
        assert_eq!(project_root, project_dir.path());

        // 5. Check that cache is created
        let cache_dir = home_dir.path().join(".config/lazymvn");
        let cache_path = cache_dir.join("cache.json");
        assert!(cache_path.exists());
        let cache = cache::load_cache(&cache_path).unwrap();
        assert_eq!(
            cache.modules,
            vec!["module1".to_string(), "module2".to_string()]
        );
        assert_eq!(cache.project_root, project_dir.path());
        assert_eq!(cache.pom_hash, Some(expected_hash));

        // 6. Delete pom.xml
        std::fs::remove_file(&pom_path).unwrap();

        // 7. Call get_project_modules for the second time
        let (modules_from_cache, project_root_from_cache) = get_project_modules().unwrap();
        assert_eq!(modules_from_cache, vec!["module1", "module2"]);
        assert_eq!(project_root_from_cache, project_dir.path());

        // 8. Cleanup
        env::set_current_dir(original_dir).unwrap();
        restore_home(original_home);
    }

    #[test]
    #[cfg(unix)] // HOME environment variable handling differs on Windows
    fn get_project_modules_refreshes_cache_when_pom_changes() {
        let _guard = home_lock().lock().unwrap();
        let project_dir = tempdir().unwrap();
        let home_dir = tempdir().unwrap();
        let original_home = env::var("HOME").ok();
        set_home(home_dir.path());

        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(project_dir.path()).unwrap();

        let pom_path = project_dir.path().join("pom.xml");
        {
            let mut pom_file = File::create(&pom_path).unwrap();
            use std::io::Write;
            pom_file
                .write_all(b"<project><modules><module>module1</module></modules></project>")
                .unwrap();
        }

        let (modules_initial, _) = get_project_modules().unwrap();
        assert_eq!(modules_initial, vec!["module1"]);

        {
            let mut pom_file = File::create(&pom_path).unwrap();
            use std::io::Write;
            pom_file
                .write_all(b"<project><modules><module>module3</module></modules></project>")
                .unwrap();
        }
        let updated_pom = std::fs::read_to_string(&pom_path).unwrap();
        let expected_hash = super::compute_pom_hash(&updated_pom);

        let (modules_updated, _) = get_project_modules().unwrap();
        assert_eq!(modules_updated, vec!["module3"]);

        let cache_path = home_dir.path().join(".config/lazymvn/cache.json");
        let cache = cache::load_cache(&cache_path).unwrap();
        assert_eq!(cache.modules, vec!["module3".to_string()]);
        assert_eq!(cache.pom_hash, Some(expected_hash));

        env::set_current_dir(original_dir).unwrap();
        restore_home(original_home);
    }

    #[test]
    fn get_project_modules_for_project_without_modules() {
        let _guard = home_lock().lock().unwrap();
        let project_dir = tempdir().unwrap();
        let home_dir = tempdir().unwrap();
        let original_home = env::var("HOME").ok();
        set_home(home_dir.path());

        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(project_dir.path()).unwrap();

        let pom_path = project_dir.path().join("pom.xml");
        {
            let mut pom_file = File::create(&pom_path).unwrap();
            use std::io::Write;
            pom_file
                .write_all(b"<project><groupId>com.example</groupId><artifactId>simple</artifactId></project>")
                .unwrap();
        }

        let (modules, project_root) = get_project_modules().unwrap();
        assert_eq!(modules, vec!["."]);
        assert_eq!(project_root, project_dir.path());

        env::set_current_dir(original_dir).unwrap();
        restore_home(original_home);
    }
}
