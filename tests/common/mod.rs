// Common test fixtures for Maven integration tests
use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

/// Global test lock to prevent concurrent test execution that might interfere
#[allow(dead_code)]
pub fn test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

/// Write a script file with appropriate permissions for the platform
#[allow(dead_code)]
pub fn write_script(path: &Path, content: &str) {
    fs::write(path, content).unwrap();
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).unwrap();
    }
    
    // On Windows, batch files (.bat, .cmd) are executable by default
    // For tests, we create both the script and a .bat version
    #[cfg(windows)]
    {
        // Create a .bat file for Windows
        let bat_path = path.with_extension("bat");
        // Convert basic shell echo to batch echo
        let bat_content = content
            .replace("#!/bin/sh\n", "")
            .replace("echo $@", "echo %*");
        fs::write(&bat_path, bat_content).unwrap();
    }
}
