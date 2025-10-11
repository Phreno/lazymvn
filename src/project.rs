use std::path::PathBuf;

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
}
