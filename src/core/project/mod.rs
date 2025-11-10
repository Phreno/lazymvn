mod cache;
mod discovery;
mod parser;

use std::fs;
use std::path::{Path, PathBuf};

pub use discovery::find_pom;

type ModulesResult = Result<Option<(Vec<String>, PathBuf)>, Box<dyn std::error::Error>>;

/// Get project modules for the current working directory
pub fn get_project_modules() -> Result<(Vec<String>, PathBuf), Box<dyn std::error::Error>> {
    log::debug!("get_project_modules: Starting module discovery");
    let (cache_dir, cache_path) = cache::get_cache_paths()?;
    let cached_entry = load_cache_if_exists(&cache_path);
    let current_dir = std::env::current_dir()?;
    log::debug!("Current directory: {:?}", current_dir);

    if let Some(cache) = cached_entry.as_ref()
        && let Some(result) = try_use_cache(cache, &current_dir, &cache_dir, &cache_path)?
    {
        return Ok(result);
    }

    discover_and_cache_modules(&cache_dir, &cache_path)
}

/// Get project modules for a specific project path
/// This is used when opening projects in tabs
#[allow(dead_code)]
pub fn get_project_modules_for_path(
    project_path: &PathBuf,
) -> Result<(Vec<String>, PathBuf), Box<dyn std::error::Error>> {
    log::debug!(
        "get_project_modules_for_path: Loading project from {:?}",
        project_path
    );

    let pom_path = discovery::find_pom_in_path(project_path)
        .ok_or("pom.xml not found in project path or parent directories")?;

    let pom_content = fs::read_to_string(&pom_path)?;
    let modules = parser::normalize_modules(parser::parse_modules_from_str(&pom_content));
    let project_root = pom_path.parent().unwrap().to_path_buf();

    log::info!(
        "Loaded {} modules from project: {:?}",
        modules.len(),
        project_root
    );

    Ok((modules, project_root))
}

/// Load cache if it exists
fn load_cache_if_exists(cache_path: &Path) -> Option<cache::Cache> {
    if cache_path.exists() {
        log::debug!("Cache file exists, attempting to load");
        cache::load_cache(cache_path).ok()
    } else {
        log::debug!("No cache file found");
        None
    }
}

/// Try to use cached modules
fn try_use_cache(
    cache: &cache::Cache,
    current_dir: &Path,
    cache_dir: &Path,
    cache_path: &Path,
) -> ModulesResult {
    log::debug!("Checking cached project root: {:?}", cache.project_root);
    
    if !current_dir.starts_with(&cache.project_root) {
        log::debug!("Current dir not under cached project root, discovering new project");
        return Ok(None);
    }

    let pom_path = cache.project_root.join("pom.xml");
    if let Ok(pom_content) = fs::read_to_string(&pom_path) {
        let pom_hash = parser::compute_pom_hash(&pom_content);
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
            let modules = parser::normalize_modules(cache.modules.clone());
            return Ok(Some((modules, cache.project_root.clone())));
        }

        log::info!("POM changed, reparsing modules");
        return Ok(Some(update_cache_with_new_pom(
            &pom_content,
            &cache.project_root,
            cache_dir,
            cache_path,
        )?));
    }
    
    log::warn!("Could not read POM, using cached modules");
    let modules = parser::normalize_modules(cache.modules.clone());
    Ok(Some((modules, cache.project_root.clone())))
}

/// Update cache with new POM content
fn update_cache_with_new_pom(
    pom_content: &str,
    project_root: &Path,
    cache_dir: &Path,
    cache_path: &Path,
) -> Result<(Vec<String>, PathBuf), Box<dyn std::error::Error>> {
    let modules = parser::normalize_modules(parser::parse_modules_from_str(pom_content));
    let pom_hash = parser::compute_pom_hash(pom_content);
    log::debug!("Parsed {} modules from updated POM", modules.len());
    
    fs::create_dir_all(cache_dir)?;
    let updated_cache = cache::Cache {
        project_root: project_root.to_path_buf(),
        modules: modules.clone(),
        pom_hash: Some(pom_hash),
    };
    cache::save_cache(cache_path, &updated_cache)?;
    log::debug!("Updated cache saved");
    
    Ok((modules, project_root.to_path_buf()))
}

/// Discover modules and create cache
fn discover_and_cache_modules(
    cache_dir: &Path,
    cache_path: &Path,
) -> Result<(Vec<String>, PathBuf), Box<dyn std::error::Error>> {
    log::debug!("No valid cache, searching for pom.xml");
    let pom_path = find_pom().ok_or("pom.xml not found")?;
    let project_root = pom_path.parent().unwrap().to_path_buf();
    log::info!("Discovered project root: {:?}", project_root);

    let pom_content = fs::read_to_string(&pom_path).unwrap_or_default();
    let modules = parser::normalize_modules(parser::parse_modules_from_str(&pom_content));
    
    log_discovered_modules(&modules);
    
    let pom_hash = parser::compute_pom_hash(&pom_content);
    log::debug!("Computed POM hash: {}", pom_hash);

    save_new_cache(cache_dir, cache_path, &project_root, &modules, pom_hash)?;
    
    Ok((modules, project_root))
}

/// Log discovered modules
fn log_discovered_modules(modules: &[String]) {
    log::debug!("Parsed {} modules from POM", modules.len());
    for (i, module) in modules.iter().enumerate() {
        log::debug!("  Module {}: {}", i + 1, module);
    }
}

/// Save new cache
fn save_new_cache(
    cache_dir: &Path,
    cache_path: &Path,
    project_root: &Path,
    modules: &[String],
    pom_hash: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(cache_dir)?;
    let new_cache = cache::Cache {
        project_root: project_root.to_path_buf(),
        modules: modules.to_vec(),
        pom_hash: Some(pom_hash),
    };
    cache::save_cache(cache_path, &new_cache)?;
    log::debug!("New cache saved");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::fs_lock;
    use std::env;
    use std::fs::File;
    use tempfile::tempdir;

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
    #[cfg(unix)]
    fn get_project_modules_integration_test() {
        let _guard = fs_lock().lock().unwrap();
        let project_dir = tempdir().unwrap();
        let home_dir = tempdir().unwrap();
        let original_home = env::var("HOME").ok();
        set_home(home_dir.path());

        let pom_path = project_dir.path().join("pom.xml");
        let mut pom_file = File::create(&pom_path).unwrap();
        use std::io::Write;
        pom_file.write_all(b"<project><modules><module>module1</module><module>module2</module></modules></project>").unwrap();
        let pom_content = fs::read_to_string(&pom_path).unwrap();
        let expected_hash = parser::compute_pom_hash(&pom_content);

        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(project_dir.path()).unwrap();

        let (modules, project_root) = get_project_modules().unwrap();
        assert_eq!(modules, vec!["module1", "module2"]);
        assert_eq!(project_root, project_dir.path());

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

        std::fs::remove_file(&pom_path).unwrap();

        let (modules_from_cache, project_root_from_cache) = get_project_modules().unwrap();
        assert_eq!(modules_from_cache, vec!["module1", "module2"]);
        assert_eq!(project_root_from_cache, project_dir.path());

        env::set_current_dir(original_dir).unwrap();
        restore_home(original_home);
    }

    #[test]
    #[cfg(unix)]
    fn get_project_modules_refreshes_cache_when_pom_changes() {
        let _guard = fs_lock().lock().unwrap();
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
        let updated_pom = fs::read_to_string(&pom_path).unwrap();
        let expected_hash = parser::compute_pom_hash(&updated_pom);

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
        let _guard = fs_lock().lock().unwrap();
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
