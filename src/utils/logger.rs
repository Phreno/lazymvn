use log::{LevelFilter, Metadata, Record, SetLoggerError};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
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
            let log_line = format_log_line(record, &self.session_id.lock().unwrap());
            write_to_debug_file(&self.file, &log_line);
            
            if is_error_level(record) {
                write_to_error_file(&self.error_file, &log_line);
            }
        }
    }

    fn flush(&self) {
        flush_file(&self.file);
        flush_file(&self.error_file);
    }
}

/// Format a log line with session ID and timestamp
fn format_log_line(record: &Record, session_id: &Option<String>) -> String {
    let timestamp = get_current_timestamp();
    let session_prefix = get_session_prefix(session_id);
    format!(
        "{}[{}] {} - {}",
        session_prefix,
        timestamp,
        record.level(),
        record.args()
    )
}

/// Get current timestamp formatted for logs
fn get_current_timestamp() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string()
}

/// Get session prefix for log line
fn get_session_prefix(session_id: &Option<String>) -> String {
    session_id
        .as_ref()
        .map(|id| format!("[SESSION:{}] ", id))
        .unwrap_or_default()
}

/// Check if record is error level
fn is_error_level(record: &Record) -> bool {
    record.level() == log::Level::Error
}

/// Write log line to debug file
fn write_to_debug_file(file: &Mutex<Option<File>>, log_line: &str) {
    let mut file_guard = file.lock().unwrap();
    if let Some(f) = file_guard.as_mut() {
        let _ = writeln!(f, "{}", log_line);
    }
}

/// Write log line to error file
fn write_to_error_file(file: &Mutex<Option<File>>, log_line: &str) {
    let mut file_guard = file.lock().unwrap();
    if let Some(f) = file_guard.as_mut() {
        let _ = writeln!(f, "{}", log_line);
    }
}

/// Flush a file
fn flush_file(file: &Mutex<Option<File>>) {
    let mut file_guard = file.lock().unwrap();
    if let Some(f) = file_guard.as_mut() {
        let _ = f.flush();
    }
}

/// Get the system log directory for LazyMVN
fn get_log_dir() -> Result<PathBuf, std::io::Error> {
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

/// Clean up old rotated log files (older than 30 days)
fn cleanup_old_logs(log_dir: &PathBuf) -> Result<(), std::io::Error> {
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
fn rotate_log_file(log_path: &PathBuf, max_size_mb: u64) -> Result<(), std::io::Error> {
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
                let _ = std::fs::rename(&old_backup, &new_backup);
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
    use std::io::BufReader;

    let file = File::open(log_path)?;
    let reader = BufReader::new(file);
    let session_marker = build_session_marker(session_id);

    collect_session_lines(reader, &session_marker)
}

/// Build session marker string
fn build_session_marker(session_id: &str) -> String {
    format!("[SESSION:{}]", session_id)
}

/// Collect lines belonging to a session
fn collect_session_lines<R: std::io::BufRead>(
    reader: R,
    session_marker: &str,
) -> Result<Vec<String>, std::io::Error> {
    let mut session_logs = Vec::new();
    let mut in_session = false;

    for line in reader.lines() {
        let line = line?;

        if is_line_from_session(&line, session_marker) {
            in_session = true;
            session_logs.push(line);
        } else if in_session {
            if is_different_session_marker(&line, session_marker) {
                break;
            }
            session_logs.push(line);
        }
    }

    Ok(session_logs)
}

/// Check if line is from our session
fn is_line_from_session(line: &str, session_marker: &str) -> bool {
    line.contains(session_marker)
}

/// Check if line marks a different session
fn is_different_session_marker(line: &str, session_marker: &str) -> bool {
    line.contains("[SESSION:") && !line.contains(session_marker)
}

/// Get concatenated logs from the current session (debug + error logs)
#[allow(dead_code)]
pub fn get_current_session_logs() -> Result<String, String> {
    let session_id = get_session_id().ok_or("No session ID available")?;
    flush_and_wait_for_sync();
    
    let mut all_logs = build_log_header(&session_id);
    add_debug_logs(&mut all_logs, &session_id);
    add_error_logs(&mut all_logs, &session_id);

    Ok(all_logs.join("\n"))
}

/// Flush logs and wait for filesystem sync
fn flush_and_wait_for_sync() {
    log::logger().flush();
    std::thread::sleep(std::time::Duration::from_millis(10));
}

/// Build log header
fn build_log_header(session_id: &str) -> Vec<String> {
    vec![
        "=== LazyMVN Session Logs ===".to_string(),
        format!("Session ID: {}", session_id),
        format!(
            "Timestamp: {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        ),
        String::new(),
    ]
}

/// Add debug logs to output
fn add_debug_logs(all_logs: &mut Vec<String>, session_id: &str) {
    if let Some(debug_path) = get_debug_log_path()
        && debug_path.exists()
    {
        all_logs.push("=== Debug Logs ===".to_string());
        add_session_logs_from_file(all_logs, &debug_path, session_id, "debug");
        all_logs.push(String::new());
    }
}

/// Add error logs to output
fn add_error_logs(all_logs: &mut Vec<String>, session_id: &str) {
    if let Some(error_path) = get_error_log_path()
        && error_path.exists()
    {
        all_logs.push("=== Error Logs ===".to_string());
        add_session_logs_from_file(all_logs, &error_path, session_id, "errors");
        all_logs.push(String::new());
    }
}

/// Add session logs from a file to output
fn add_session_logs_from_file(
    all_logs: &mut Vec<String>,
    path: &PathBuf,
    session_id: &str,
    log_type: &str,
) {
    match extract_session_logs(path, session_id) {
        Ok(logs) => {
            if logs.is_empty() {
                all_logs.push(format!("(No {} for this session)", log_type));
            } else {
                all_logs.extend(logs);
            }
        }
        Err(e) => {
            all_logs.push(format!("Error reading {} logs: {}", log_type, e));
        }
    }
}

/// Initialize the logger with the specified log level
/// Level can be: "off", "error", "warn", "info", "debug", "trace"
/// In development mode, defaults to "debug" if None is provided
pub fn init(log_level: Option<&str>) -> Result<(), SetLoggerError> {
    let level_filter = determine_log_level(log_level);

    if level_filter != LevelFilter::Off {
        init_logger_files(level_filter)?;
    } else {
        log::set_max_level(LevelFilter::Off);
    }

    Ok(())
}

/// Determine the log level filter
fn determine_log_level(log_level: Option<&str>) -> LevelFilter {
    match log_level {
        Some("off") => LevelFilter::Off,
        Some("error") => LevelFilter::Error,
        Some("warn") => LevelFilter::Warn,
        Some("info") => LevelFilter::Info,
        Some("debug") => LevelFilter::Debug,
        Some("trace") => LevelFilter::Trace,
        None => get_default_log_level(),
        Some(other) => {
            eprintln!(
                "Warning: Unknown log level '{}', defaulting to 'info'",
                other
            );
            LevelFilter::Info
        }
    }
}

/// Get default log level based on build type
fn get_default_log_level() -> LevelFilter {
    if version::is_nightly() {
        LevelFilter::Debug
    } else {
        LevelFilter::Off
    }
}

/// Initialize logger files and configuration
fn init_logger_files(level_filter: LevelFilter) -> Result<(), SetLoggerError> {
    let log_dir = get_log_dir().expect("Failed to get log directory");
    let (debug_log_path, error_log_path) = get_log_paths(&log_dir);

    prepare_log_files(&log_dir, &debug_log_path, &error_log_path);
    
    let (file, error_file) = open_log_files(&debug_log_path, &error_log_path);
    let session_id = generate_session_id();
    
    setup_logger(file, error_file, session_id.clone(), level_filter)?;
    log_session_start(&session_id, level_filter, &log_dir, &debug_log_path, &error_log_path);
    
    Ok(())
}

/// Get log file paths
fn get_log_paths(log_dir: &Path) -> (PathBuf, PathBuf) {
    let debug_log_path = log_dir.join("debug.log");
    let error_log_path = log_dir.join("error.log");
    (debug_log_path, error_log_path)
}

/// Prepare log files (cleanup and rotation)
fn prepare_log_files(log_dir: &PathBuf, debug_log_path: &PathBuf, error_log_path: &PathBuf) {
    let _ = cleanup_old_logs(log_dir);
    let _ = rotate_log_file(debug_log_path, 5);
    let _ = rotate_log_file(error_log_path, 5);
}

/// Open log files
fn open_log_files(debug_log_path: &PathBuf, error_log_path: &PathBuf) -> (File, File) {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(debug_log_path)
        .expect("Failed to open log file");

    let error_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(error_log_path)
        .expect("Failed to open error log file");

    (file, error_file)
}

/// Generate a unique session ID
fn generate_session_id() -> String {
    format!("{}", chrono::Local::now().format("%Y%m%d-%H%M%S-%3f"))
}

/// Setup logger with files and level
fn setup_logger(
    file: File,
    error_file: File,
    session_id: String,
    level_filter: LevelFilter,
) -> Result<(), SetLoggerError> {
    *LOGGER.file.lock().unwrap() = Some(file);
    *LOGGER.error_file.lock().unwrap() = Some(error_file);
    *LOGGER.session_id.lock().unwrap() = Some(session_id);

    log::set_logger(&LOGGER)?;
    log::set_max_level(level_filter);
    
    Ok(())
}

/// Log session start information
fn log_session_start(
    session_id: &str,
    level_filter: LevelFilter,
    log_dir: &Path,
    debug_log_path: &Path,
    error_log_path: &Path,
) {
    log::info!("=== LazyMVN Session Started ===");
    log::info!("Session ID: {}", session_id);
    log::info!("Log level: {:?}", level_filter);
    log::info!("Log directory: {}", log_dir.display());
    log::info!("Debug log: {}", debug_log_path.display());
    log::info!("Error log: {}", error_log_path.display());
}

/// Read the last N lines from a file (tail-like functionality)
#[allow(dead_code)]
fn read_last_lines(path: &Path, max_lines: usize) -> Result<Vec<String>, std::io::Error> {
    use std::io::{BufRead, BufReader};

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let all_lines: Vec<String> = reader.lines().collect::<Result<Vec<_>, _>>()?;

    Ok(get_last_n_lines(all_lines, max_lines))
}

/// Get the last N lines from a vector
fn get_last_n_lines(all_lines: Vec<String>, max_lines: usize) -> Vec<String> {
    let start_idx = calculate_start_index(all_lines.len(), max_lines);
    all_lines[start_idx..].to_vec()
}

/// Calculate start index for last N lines
fn calculate_start_index(total_lines: usize, max_lines: usize) -> usize {
    total_lines.saturating_sub(max_lines)
}

/// Get all available logs (last 500 lines from debug and error logs)
#[allow(dead_code)]
pub fn get_all_logs() -> String {
    let mut output = Vec::new();
    add_debug_log_tail(&mut output);
    add_error_log_tail(&mut output);
    output.join("\n")
}

/// Add debug log tail to output
fn add_debug_log_tail(output: &mut Vec<String>) {
    if let Some(debug_path) = get_debug_log_path()
        && debug_path.exists()
    {
        output.push("=== Debug Logs (last 500 lines) ===".to_string());
        add_log_tail(output, &debug_path, "debug");
        output.push(String::new());
    }
}

/// Add error log tail to output
fn add_error_log_tail(output: &mut Vec<String>) {
    if let Some(error_path) = get_error_log_path()
        && error_path.exists()
    {
        output.push("=== Error Logs (last 500 lines) ===".to_string());
        add_log_tail(output, &error_path, "error");
    }
}

/// Add log tail to output
fn add_log_tail(output: &mut Vec<String>, path: &Path, log_type: &str) {
    match read_last_lines(path, 500) {
        Ok(lines) => {
            if lines.is_empty() {
                output.push(format!("(No {} logs)", log_type));
            } else {
                output.extend(lines);
            }
        }
        Err(e) => {
            output.push(format!("Error reading {} logs: {}", log_type, e));
        }
    }
}

/// Get logs for debug report (optimized for size)
/// - Only includes current session logs (across all rotated files)
/// - Filters out TRACE level logs (keeps DEBUG, INFO, WARN, ERROR)
/// - Limits to last 300 lines to keep report manageable
#[allow(dead_code)]
pub fn get_logs_for_debug_report() -> String {
    match get_current_session_logs() {
        Ok(session_logs) => format_debug_report(&session_logs),
        Err(e) => format!("Error getting session logs: {}", e),
    }
}

/// Format session logs for debug report
fn format_debug_report(session_logs: &str) -> String {
    let lines: Vec<&str> = session_logs.lines().collect();
    let filtered = filter_and_limit_logs(&lines);
    
    if filtered.is_empty() {
        "(No logs for current session)".to_string()
    } else {
        build_filtered_output(&lines, &filtered)
    }
}

/// Filter out TRACE logs and limit to 300 lines
fn filter_and_limit_logs(lines: &[&str]) -> Vec<String> {
    lines
        .iter()
        .filter(|line| !is_trace_log_line(line))
        .take(300)
        .map(|s| s.to_string())
        .collect()
}

/// Check if line is a TRACE level log
fn is_trace_log_line(line: &str) -> bool {
    line.contains("] TRACE - ")
}

/// Build output with filter information
fn build_filtered_output(original_lines: &[&str], filtered_lines: &[String]) -> String {
    let total_lines = original_lines.len();
    let filtered_count = filtered_lines.len();
    let mut output = Vec::new();
    
    if filtered_count < total_lines {
        output.push(format!(
            "(Filtered {} lines, showing {} lines - TRACE logs excluded)",
            total_lines, filtered_count
        ));
    }
    
    output.extend(filtered_lines.iter().cloned());
    output.join("\n")
}
