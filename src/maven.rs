use std::path::Path;

pub fn get_maven_command(project_root: &Path) -> String {
    if project_root.join("mvnw").exists() {
        "./mvnw".to_string()
    } else {
        "mvn".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn get_maven_command_returns_mvnw_if_present() {
        let dir = tempdir().unwrap();
        let project_root = dir.path();

        // Test with mvnw present
        let mvnw_path = project_root.join("mvnw");
        File::create(&mvnw_path).unwrap();
        assert_eq!(get_maven_command(project_root), "./mvnw");

        // Test without mvnw present
        std::fs::remove_file(&mvnw_path).unwrap();
        assert_eq!(get_maven_command(project_root), "mvn");
    }
}
