use log::{LevelFilter, Metadata, Record, SetLoggerError};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use super::version;

static LOGGER: Logger = Logger {
    file: Mutex::new(None),
    error_file: Mutex::new(None),
    session_id: Mutex::new(None),
};

struct Logger {
    file: Mutex<Option<File>>,
    error_file: Mutex<Option<File>>,
    session_id: Mutex<Option<String>>,
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let session_id = self.session_id.lock().unwrap();
            let session_prefix = session_id
                .as_ref()
                .map(|id| format!("[SESSION:{}] ", id))
                .unwrap_or_default();
            let log_line = format!(
                "{}[{}] {} - {}",
                session_prefix,
                timestamp,
                record.level(),
                record.args()
            );

            // Log to main debug file
            let mut file_guard = self.file.lock().unwrap();
            if let Some(file) = file_guard.as_mut() {
                let _ = writeln!(file, "{}", log_line);
            }

            // Also log errors to dedicated error file
            if record.level() == log::Level::Error {
                let mut error_file_guard = self.error_file.lock().unwrap();
                if let Some(error_file) = error_file_guard.as_mut() {
                    let _ = writeln!(error_file, "{}", log_line);
                }
            }
        }
    }

    fn flush(&self) {
        let mut file_guard = self.file.lock().unwrap();
        if let Some(file) = file_guard.as_mut() {
            let _ = file.flush();
        }
        let mut error_file_guard = self.error_file.lock().unwrap();
        if let Some(error_file) = error_file_guard.as_mut() {
            let _ = error_file.flush();
        }
    }
}

/// Get the system log directory for LazyMVN
fn get_log_dir() -> Result<PathBuf, std::io::Error> {
    let dirs = directories::ProjectDirs::from("com", "lazymvn", "lazymvn").ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine home directory",
        )
    })?;

    let log_dir = dirs.data_local_dir().join("logs");

    // Create the log directory if it doesn't exist
    std::fs::create_dir_all(&log_dir)?;

    Ok(log_dir)
}

/// Clean up old rotated log files (older than 30 days)
fn cleanup_old_logs(log_dir: &PathBuf) -> Result<(), std::io::Error> {
    use std::time::{Duration, SystemTime};

    let thirty_days_ago = SystemTime::now() - Duration::from_secs(30 * 24 * 60 * 60);

    // Read all files in the log directory
    let entries = std::fs::read_dir(log_dir)?;

    for entry in entries.flatten() {
        let path = entry.path();
        
        // Only process rotated log files (*.log.1, *.log.2, etc.)
        if let Some(filename) = path.file_name().and_then(|n| n.to_str())
            && filename.contains(".log.")
        {
            // Check file modification time
            if let Ok(metadata) = entry.metadata()
                && let Ok(modified) = metadata.modified()
                && modified < thirty_days_ago
            {
                // Delete old rotated log
                let _ = std::fs::remove_file(&path);
            }
        }
    }

    Ok(())
}

/// Rotate a log file if it exceeds the size limit
/// Keeps up to 5 rotated files (file.log.1, file.log.2, ..., file.log.5)
fn rotate_log_file(log_path: &PathBuf, max_size_mb: u64) -> Result<(), std::io::Error> {
    // Check if file exists and its size
    if !log_path.exists() {
        return Ok(());
    }

    let metadata = std::fs::metadata(log_path)?;
    let size_mb = metadata.len() / (1024 * 1024);

    // If file is under the limit, no rotation needed
    if size_mb < max_size_mb {
        return Ok(());
    }

    // Rotate existing backups (5 -> delete, 4 -> 5, 3 -> 4, 2 -> 3, 1 -> 2)
    for i in (1..=5).rev() {
        let old_backup = log_path.with_extension(format!("log.{}", i));
        if i == 5 {
            // Delete the oldest backup
            let _ = std::fs::remove_file(&old_backup);
        } else {
            // Rename to next number
            let new_backup = log_path.with_extension(format!("log.{}", i + 1));
            if old_backup.exists() {
                let _ = std::fs::rename(&old_backup, &new_backup);
            }
        }
    }

    // Move current log to .log.1
    let backup = log_path.with_extension("log.1");
    std::fs::rename(log_path, &backup)?;

    Ok(())
}

/// Get the path to the debug log file
pub fn get_debug_log_path() -> Option<PathBuf> {
    get_log_dir().ok().map(|dir| dir.join("debug.log"))
}

/// Get the path to the error log file
pub fn get_error_log_path() -> Option<PathBuf> {
    get_log_dir().ok().map(|dir| dir.join("error.log"))
}

/// Get the current session ID
#[allow(dead_code)]
pub fn get_session_id() -> Option<String> {
    LOGGER.session_id.lock().ok()?.clone()
}

/// Extract logs for the current session from a log file
#[allow(dead_code)]
fn extract_session_logs(
    log_path: &PathBuf,
    session_id: &str,
) -> Result<Vec<String>, std::io::Error> {
    use std::io::{BufRead, BufReader};

    let file = File::open(log_path)?;
    let reader = BufReader::new(file);
    let session_marker = format!("[SESSION:{}]", session_id);

    let mut session_logs = Vec::new();
    let mut in_session = false;

    for line in reader.lines() {
        let line = line?;

        // Check if this line belongs to our session
        if line.contains(&session_marker) {
            in_session = true;
            session_logs.push(line);
        } else if in_session {
            // Check if we've hit a new session
            if line.contains("[SESSION:") && !line.contains(&session_marker) {
                break;
            }
            session_logs.push(line);
        }
    }

    Ok(session_logs)
}

/// Get concatenated logs from the current session (debug + error logs)
#[allow(dead_code)]
pub fn get_current_session_logs() -> Result<String, String> {
    let session_id = get_session_id().ok_or("No session ID available")?;

    let mut all_logs = Vec::new();

    // Add header
    all_logs.push("=== LazyMVN Session Logs ===".to_string());
    all_logs.push(format!("Session ID: {}", session_id));
    all_logs.push(format!(
        "Timestamp: {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    ));
    all_logs.push(String::new());

    // Extract debug logs
    if let Some(debug_path) = get_debug_log_path()
        && debug_path.exists()
    {
        all_logs.push("=== Debug Logs ===".to_string());
        match extract_session_logs(&debug_path, &session_id) {
            Ok(logs) => {
                if logs.is_empty() {
                    all_logs.push("(No debug logs for this session)".to_string());
                } else {
                    all_logs.extend(logs);
                }
            }
            Err(e) => {
                all_logs.push(format!("Error reading debug logs: {}", e));
            }
        }
        all_logs.push(String::new());
    }

    // Extract error logs
    if let Some(error_path) = get_error_log_path()
        && error_path.exists()
    {
        all_logs.push("=== Error Logs ===".to_string());
        match extract_session_logs(&error_path, &session_id) {
            Ok(logs) => {
                if logs.is_empty() {
                    all_logs.push("(No errors for this session)".to_string());
                } else {
                    all_logs.extend(logs);
                }
            }
            Err(e) => {
                all_logs.push(format!("Error reading error logs: {}", e));
            }
        }
    }

    Ok(all_logs.join("\n"))
}

/// Initialize the logger with the specified log level
/// Level can be: "off", "error", "warn", "info", "debug", "trace"
/// In development mode, defaults to "debug" if None is provided
pub fn init(log_level: Option<&str>) -> Result<(), SetLoggerError> {
    // Determine the log level filter
    let level_filter = match log_level {
        Some("off") => LevelFilter::Off,
        Some("error") => LevelFilter::Error,
        Some("warn") => LevelFilter::Warn,
        Some("info") => LevelFilter::Info,
        Some("debug") => LevelFilter::Debug,
        Some("trace") => LevelFilter::Trace,
        None => {
            // Default to Debug in development/nightly builds
            if version::is_nightly() {
                LevelFilter::Debug
            } else {
                LevelFilter::Off
            }
        }
        Some(other) => {
            eprintln!(
                "Warning: Unknown log level '{}', defaulting to 'info'",
                other
            );
            LevelFilter::Info
        }
    };

    if level_filter != LevelFilter::Off {
        let log_dir = get_log_dir().expect("Failed to get log directory");

        let debug_log_path = log_dir.join("debug.log");
        let error_log_path = log_dir.join("error.log");

        // Clean up old rotated logs (older than 30 days)
        let _ = cleanup_old_logs(&log_dir);

        // Rotate logs if they're too large (5 MB limit per file)
        let _ = rotate_log_file(&debug_log_path, 5);
        let _ = rotate_log_file(&error_log_path, 5);

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&debug_log_path)
            .expect("Failed to open log file");

        let error_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&error_log_path)
            .expect("Failed to open error log file");

        // Generate a unique session ID
        let session_id = format!("{}", chrono::Local::now().format("%Y%m%d-%H%M%S-%3f"));

        *LOGGER.file.lock().unwrap() = Some(file);
        *LOGGER.error_file.lock().unwrap() = Some(error_file);
        *LOGGER.session_id.lock().unwrap() = Some(session_id.clone());

        log::set_logger(&LOGGER)?;
        log::set_max_level(level_filter);

        log::info!("=== LazyMVN Session Started ===");
        log::info!("Session ID: {}", session_id);
        log::info!("Log level: {:?}", level_filter);
        log::info!("Log directory: {}", log_dir.display());
        log::info!("Debug log: {}", debug_log_path.display());
        log::info!("Error log: {}", error_log_path.display());
    } else {
        log::set_max_level(LevelFilter::Off);
    }

    Ok(())
}

/// Read the last N lines from a file (tail-like functionality)
#[allow(dead_code)]
fn read_last_lines(path: &PathBuf, max_lines: usize) -> Result<Vec<String>, std::io::Error> {
    use std::io::{BufRead, BufReader};

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read all lines and keep the last N
    let all_lines: Vec<String> = reader.lines().collect::<Result<Vec<_>, _>>()?;

    let start_idx = if all_lines.len() > max_lines {
        all_lines.len() - max_lines
    } else {
        0
    };

    Ok(all_lines[start_idx..].to_vec())
}

/// Get all available logs (last 500 lines from debug and error logs)
#[allow(dead_code)]
pub fn get_all_logs() -> String {
    let mut output = Vec::new();

    // Add debug logs
    if let Some(debug_path) = get_debug_log_path()
        && debug_path.exists()
    {
        output.push("=== Debug Logs (last 500 lines) ===".to_string());
        match read_last_lines(&debug_path, 500) {
            Ok(lines) => {
                if lines.is_empty() {
                    output.push("(No debug logs)".to_string());
                } else {
                    output.extend(lines);
                }
            }
            Err(e) => {
                output.push(format!("Error reading debug logs: {}", e));
            }
        }
        output.push(String::new());
    }

    // Add error logs
    if let Some(error_path) = get_error_log_path()
        && error_path.exists()
    {
        output.push("=== Error Logs (last 500 lines) ===".to_string());
        match read_last_lines(&error_path, 500) {
            Ok(lines) => {
                if lines.is_empty() {
                    output.push("(No error logs)".to_string());
                } else {
                    output.extend(lines);
                }
            }
            Err(e) => {
                output.push(format!("Error reading error logs: {}", e));
            }
        }
    }

    output.join("\n")
}

/// Get logs for debug report (optimized for size)
/// - Only includes current session logs (across all rotated files)
/// - Filters out TRACE level logs (keeps DEBUG, INFO, WARN, ERROR)
/// - Limits to last 300 lines to keep report manageable
#[allow(dead_code)]
pub fn get_logs_for_debug_report() -> String {
    // Get current session logs (already handles rotation)
    match get_current_session_logs() {
        Ok(session_logs) => {
            let lines: Vec<&str> = session_logs.lines().collect();
            
            // Filter out TRACE level logs and limit size
            let filtered: Vec<String> = lines
                .iter()
                .filter(|line| {
                    // Keep lines that don't contain TRACE level
                    // TRACE logs look like: [SESSION:...] [timestamp] TRACE - ...
                    !line.contains("] TRACE - ")
                })
                .take(300) // Limit to 300 lines
                .map(|s| s.to_string())
                .collect();
            
            if filtered.is_empty() {
                "(No logs for current session)".to_string()
            } else {
                let total_lines = lines.len();
                let filtered_lines = filtered.len();
                let mut output = Vec::new();
                
                if filtered_lines < total_lines {
                    output.push(format!(
                        "(Filtered {} lines, showing {} lines - TRACE logs excluded)",
                        total_lines, filtered_lines
                    ));
                }
                
                output.extend(filtered);
                output.join("\n")
            }
        }
        Err(e) => {
            format!("Error getting session logs: {}", e)
        }
    }
}
