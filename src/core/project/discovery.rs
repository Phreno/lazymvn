use std::path::{Path, PathBuf};

/// Find pom.xml starting from current directory, searching upward
pub fn find_pom() -> Option<PathBuf> {
    let current_dir = std::env::current_dir().ok()?;
    log::debug!("Searching for pom.xml starting from: {:?}", current_dir);
    search_pom_upward(current_dir)
}

/// Search for pom.xml in current and parent directories
fn search_pom_upward(mut current_dir: PathBuf) -> Option<PathBuf> {
    loop {
        let pom_path = current_dir.join("pom.xml");
        log::debug!("Checking path: {:?}", pom_path);
        
        if pom_path.exists() {
            log::info!("Found pom.xml at: {:?}", pom_path);
            return Some(pom_path);
        }
        
        if !has_parent_dir(&mut current_dir) {
            log::warn!("pom.xml not found in any parent directory");
            return None;
        }
    }
}

/// Check if directory has a parent and move to it
fn has_parent_dir(current_dir: &mut PathBuf) -> bool {
    current_dir.pop()
}

/// Find pom.xml in or above a specific path
pub fn find_pom_in_path(project_path: &Path) -> Option<PathBuf> {
    let mut current = project_path.to_path_buf();
    loop {
        let test_pom = current.join("pom.xml");
        if test_pom.exists() {
            log::debug!("Found pom.xml at: {:?}", test_pom);
            return Some(test_pom);
        }

        if !current.pop() {
            break;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::fs_lock;
    use std::env;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn find_pom_in_current_dir() {
        let _guard = fs_lock().lock().unwrap();
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
        let _guard = fs_lock().lock().unwrap();
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
    fn find_pom_in_path_finds_pom() {
        let dir = tempdir().unwrap();
        let pom_path = dir.path().join("pom.xml");
        File::create(&pom_path).unwrap();

        let result = find_pom_in_path(dir.path());
        assert_eq!(result, Some(pom_path));
    }

    #[test]
    fn find_pom_in_path_searches_parent() {
        let dir = tempdir().unwrap();
        let pom_path = dir.path().join("pom.xml");
        File::create(&pom_path).unwrap();

        let subdir = dir.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();

        let result = find_pom_in_path(&subdir);
        assert_eq!(result, Some(pom_path));
    }
}
