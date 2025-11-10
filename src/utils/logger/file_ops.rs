use std::fs;
use std::path::{Path, PathBuf};

/// Get the system log directory for LazyMVN
pub fn get_log_dir() -> Result<PathBuf, std::io::Error> {
    let log_dir = get_log_dir_path()?;
    ensure_dir_exists(&log_dir)?;
    Ok(log_dir)
}

/// Get the log directory path
fn get_log_dir_path() -> Result<PathBuf, std::io::Error> {
    let dirs = get_project_dirs()?;
    Ok(dirs.data_local_dir().join("logs"))
}

/// Get project directories
fn get_project_dirs() -> Result<directories::ProjectDirs, std::io::Error> {
    directories::ProjectDirs::from("com", "lazymvn", "lazymvn").ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine home directory",
        )
    })
}

/// Ensure directory exists
fn ensure_dir_exists(dir: &PathBuf) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(dir)
}

/// Get the path to the debug log file
pub fn get_debug_log_path() -> Option<PathBuf> {
    get_log_dir().ok().map(|dir| dir.join("debug.log"))
}

/// Get the path to the error log file
pub fn get_error_log_path() -> Option<PathBuf> {
    get_log_dir().ok().map(|dir| dir.join("error.log"))
}

/// Get log file paths
pub fn get_log_paths(log_dir: &Path) -> (PathBuf, PathBuf) {
    let debug_log_path = log_dir.join("debug.log");
    let error_log_path = log_dir.join("error.log");
    (debug_log_path, error_log_path)
}

/// Clean up old rotated log files (older than 30 days)
pub fn cleanup_old_logs(log_dir: &PathBuf) -> Result<(), std::io::Error> {
    let cutoff_time = get_thirty_days_ago();
    let entries = std::fs::read_dir(log_dir)?;

    for entry in entries.flatten() {
        if should_delete_old_log(&entry, cutoff_time)? {
            let _ = std::fs::remove_file(entry.path());
        }
    }

    Ok(())
}

/// Get timestamp for 30 days ago
fn get_thirty_days_ago() -> std::time::SystemTime {
    use std::time::{Duration, SystemTime};
    SystemTime::now() - Duration::from_secs(30 * 24 * 60 * 60)
}

/// Check if log entry should be deleted
fn should_delete_old_log(
    entry: &std::fs::DirEntry,
    cutoff_time: std::time::SystemTime,
) -> Result<bool, std::io::Error> {
    let path = entry.path();
    
    if !is_rotated_log_file(&path) {
        return Ok(false);
    }
    
    is_file_older_than(entry, cutoff_time)
}

/// Check if path is a rotated log file
fn is_rotated_log_file(path: &std::path::Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|filename| filename.contains(".log."))
        .unwrap_or(false)
}

/// Check if file is older than cutoff time
fn is_file_older_than(
    entry: &std::fs::DirEntry,
    cutoff_time: std::time::SystemTime,
) -> Result<bool, std::io::Error> {
    let metadata = entry.metadata()?;
    let modified = metadata.modified()?;
    Ok(modified < cutoff_time)
}

/// Rotate a log file if it exceeds the size limit
/// Keeps up to 5 rotated files (file.log.1, file.log.2, ..., file.log.5)
pub fn rotate_log_file(log_path: &PathBuf, max_size_mb: u64) -> Result<(), std::io::Error> {
    if !log_path.exists() {
        return Ok(());
    }

    if !needs_rotation(log_path, max_size_mb)? {
        return Ok(());
    }

    rotate_backups(log_path)?;
    move_current_to_backup(log_path)?;
    
    Ok(())
}

/// Check if file needs rotation
fn needs_rotation(log_path: &PathBuf, max_size_mb: u64) -> Result<bool, std::io::Error> {
    let metadata = std::fs::metadata(log_path)?;
    let size_mb = metadata.len() / (1024 * 1024);
    Ok(size_mb >= max_size_mb)
}

/// Rotate existing backup files
fn rotate_backups(log_path: &Path) -> Result<(), std::io::Error> {
    for i in (1..=5).rev() {
        let old_backup = log_path.with_extension(format!("log.{}", i));
        if i == 5 {
            let _ = std::fs::remove_file(&old_backup);
        } else {
            let new_backup = log_path.with_extension(format!("log.{}", i + 1));
            if old_backup.exists() {
                let _ = fs::rename(&old_backup, &new_backup);
            }
        }
    }
    Ok(())
}

/// Move current log to .log.1
fn move_current_to_backup(log_path: &PathBuf) -> Result<(), std::io::Error> {
    let backup = log_path.with_extension("log.1");
    std::fs::rename(log_path, &backup)
}
