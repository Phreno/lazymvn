use std::path::{Path, PathBuf};
use dirs;
use std::fs;
use quick_xml::Reader;
use quick_xml::events::Event;

pub fn find_pom() -> Option<PathBuf> {
    let mut current_dir = std::env::current_dir().ok()?;
    loop {
        let pom_path = current_dir.join("pom.xml");
        if pom_path.exists() {
            return Some(pom_path);
        }
        if !current_dir.pop() {
            return None;
        }
    }
}

pub fn get_modules(pom_path: &Path) -> Vec<String> {
    let content = fs::read_to_string(pom_path).unwrap_or_default();
    let mut reader = Reader::from_str(&content);
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

pub fn get_project_modules() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let cache_dir = home_dir.join(".config/lazymvn");
    let cache_path = cache_dir.join("cache.json");

    if cache_path.exists() {
        if let Ok(cache) = cache::load_cache(&cache_path) {
            return Ok(cache.modules);
        }
    }

    let pom_path = find_pom().ok_or("pom.xml not found")?;
    let modules = get_modules(&pom_path);

    fs::create_dir_all(&cache_dir)?;
    let cache = cache::Cache { modules: modules.clone() };
    cache::save_cache(&cache_path, &cache)?;

    Ok(modules)
}

pub mod cache {
    use serde::{Deserialize, Serialize};
    use std::path::Path;
    use std::fs;
    use serde_json;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    pub struct Cache {
        pub modules: Vec<String>,
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
    use std::fs::File;
    use std::env;
    use tempfile::tempdir;

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

        let modules = get_modules(&pom_path);
        assert_eq!(modules, vec!["module1", "module2"]);
    }

    #[test]
    fn cache_save_and_load() {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("cache.json");

        let cache_to_save = cache::Cache {
            modules: vec!["module1".to_string(), "module2".to_string()],
        };

        cache::save_cache(&cache_path, &cache_to_save).unwrap();

        let loaded_cache = cache::load_cache(&cache_path).unwrap();
        assert_eq!(cache_to_save, loaded_cache);
    }

    #[test]
    fn get_project_modules_integration_test() {
        // 1. Setup temp project and home directory
        let project_dir = tempdir().unwrap();
        let home_dir = tempdir().unwrap();
        unsafe {
            env::set_var("HOME", home_dir.path());
        }

        // 2. Create pom.xml in project
        let pom_path = project_dir.path().join("pom.xml");
        let mut pom_file = File::create(&pom_path).unwrap();
        use std::io::Write;
        pom_file.write_all(b"<project><modules><module>module1</module><module>module2</module></modules></project>").unwrap();

        // 3. Set current dir to project
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(project_dir.path()).unwrap();

        // 4. Call get_project_modules for the first time
        let modules = get_project_modules().unwrap();
        assert_eq!(modules, vec!["module1", "module2"]);

        // 5. Check that cache is created
        let cache_dir = home_dir.path().join(".config/lazymvn");
        let cache_path = cache_dir.join("cache.json");
        assert!(cache_path.exists());

        // 6. Delete pom.xml
        std::fs::remove_file(&pom_path).unwrap();

        // 7. Call get_project_modules for the second time
        let modules_from_cache = get_project_modules().unwrap();
        assert_eq!(modules_from_cache, vec!["module1", "module2"]);

        // 8. Cleanup
        env::set_current_dir(original_dir).unwrap();
    }
}
