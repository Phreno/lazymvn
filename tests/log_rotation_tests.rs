#[cfg(test)]
mod log_rotation_tests {
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Test helper to create a log file of specified size (in MB)
    fn create_test_log(path: &PathBuf, size_mb: u64) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        
        // Write size_mb worth of data
        let chunk = vec![b'A'; 1024 * 1024]; // 1 MB
        for _ in 0..size_mb {
            file.write_all(&chunk)?;
        }
        file.flush()?;
        Ok(())
    }

    /// Copy the rotate_log_file function for testing
    fn rotate_log_file(log_path: &PathBuf, max_size_mb: u64) -> Result<(), std::io::Error> {
        if !log_path.exists() {
            return Ok(());
        }

        let metadata = std::fs::metadata(log_path)?;
        let size_mb = metadata.len() / (1024 * 1024);

        if size_mb < max_size_mb {
            return Ok(());
        }

        // Rotate existing backups
        for i in (1..=5).rev() {
            let old_backup = log_path.with_extension(format!("log.{}", i));
            if i == 5 {
                let _ = std::fs::remove_file(&old_backup);
            } else {
                let new_backup = log_path.with_extension(format!("log.{}", i + 1));
                if old_backup.exists() {
                    let _ = std::fs::rename(&old_backup, &new_backup);
                }
            }
        }

        let backup = log_path.with_extension("log.1");
        std::fs::rename(log_path, &backup)?;

        Ok(())
    }

    #[test]
    fn test_rotation_not_needed_when_under_limit() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");

        // Create 3 MB log (under 5 MB limit)
        create_test_log(&log_path, 3).unwrap();
        assert!(log_path.exists());

        // Rotation should not happen
        rotate_log_file(&log_path, 5).unwrap();

        // Original file should still exist
        assert!(log_path.exists());
        // No backup should be created
        assert!(!log_path.with_extension("log.1").exists());
    }

    #[test]
    fn test_rotation_happens_when_over_limit() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");

        // Create 6 MB log (over 5 MB limit)
        create_test_log(&log_path, 6).unwrap();
        assert!(log_path.exists());

        let size_before = std::fs::metadata(&log_path).unwrap().len();
        assert!(size_before > 5 * 1024 * 1024);

        // Rotation should happen
        rotate_log_file(&log_path, 5).unwrap();

        // Original file should be moved to .log.1
        let backup = log_path.with_extension("log.1");
        assert!(backup.exists());
        
        // Backup should have the original size
        let backup_size = std::fs::metadata(&backup).unwrap().len();
        assert_eq!(backup_size, size_before);

        // Original file should not exist (or be recreated as empty)
        assert!(!log_path.exists() || std::fs::metadata(&log_path).unwrap().len() == 0);
    }

    #[test]
    fn test_multiple_rotations() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");

        // First rotation
        create_test_log(&log_path, 6).unwrap();
        rotate_log_file(&log_path, 5).unwrap();
        assert!(log_path.with_extension("log.1").exists());

        // Second rotation
        create_test_log(&log_path, 6).unwrap();
        rotate_log_file(&log_path, 5).unwrap();
        assert!(log_path.with_extension("log.1").exists());
        assert!(log_path.with_extension("log.2").exists());

        // Third rotation
        create_test_log(&log_path, 6).unwrap();
        rotate_log_file(&log_path, 5).unwrap();
        assert!(log_path.with_extension("log.1").exists());
        assert!(log_path.with_extension("log.2").exists());
        assert!(log_path.with_extension("log.3").exists());
    }

    #[test]
    fn test_max_backups_limit() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");

        // Create 6 rotations (should keep only 5)
        for _ in 0..6 {
            create_test_log(&log_path, 6).unwrap();
            rotate_log_file(&log_path, 5).unwrap();
        }

        // Should have backups 1-5
        assert!(log_path.with_extension("log.1").exists());
        assert!(log_path.with_extension("log.2").exists());
        assert!(log_path.with_extension("log.3").exists());
        assert!(log_path.with_extension("log.4").exists());
        assert!(log_path.with_extension("log.5").exists());
        
        // Should NOT have .log.6 or higher
        assert!(!log_path.with_extension("log.6").exists());
    }

    #[test]
    fn test_rotation_with_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("nonexistent.log");

        // Should not fail
        let result = rotate_log_file(&log_path, 5);
        assert!(result.is_ok());
    }
}
