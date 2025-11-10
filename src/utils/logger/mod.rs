mod core;
mod formatters;
mod file_ops;
mod reader;

use log::{LevelFilter, SetLoggerError};
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use self::core::Logger;
use crate::utils::version;

static LOGGER: Logger = Logger {
    file: Mutex::new(None),
    error_file: Mutex::new(None),
    session_id: Mutex::new(None),
};

/// Get the current session ID
#[allow(dead_code)]
pub fn get_session_id() -> Option<String> {
    LOGGER.session_id.lock().ok()?.clone()
}

/// Get the path to the debug log file
pub fn get_debug_log_path() -> Option<PathBuf> {
    file_ops::get_debug_log_path()
}

/// Get the path to the error log file
pub fn get_error_log_path() -> Option<PathBuf> {
    file_ops::get_error_log_path()
}

/// Get concatenated logs from the current session (debug + error logs)
#[allow(dead_code)]
pub fn get_current_session_logs() -> Result<String, String> {
    let session_id = get_session_id().ok_or("No session ID available")?;
    reader::get_current_session_logs(&session_id)
}

/// Get all available logs (last 500 lines from debug and error logs)
#[allow(dead_code)]
pub fn get_all_logs() -> String {
    reader::get_all_logs()
}

/// Get logs for debug report (optimized for size)
#[allow(dead_code)]
pub fn get_logs_for_debug_report() -> String {
    let session_id = match get_session_id() {
        Some(id) => id,
        None => return "Error: No session ID available".to_string(),
    };
    reader::get_logs_for_debug_report(&session_id)
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
    let log_dir = file_ops::get_log_dir().expect("Failed to get log directory");
    let (debug_log_path, error_log_path) = file_ops::get_log_paths(&log_dir);

    prepare_log_files(&log_dir, &debug_log_path, &error_log_path);
    
    let (file, error_file) = open_log_files(&debug_log_path, &error_log_path);
    let session_id = generate_session_id();
    
    setup_logger(file, error_file, session_id.clone(), level_filter)?;
    log_session_start(&session_id, level_filter, &log_dir, &debug_log_path, &error_log_path);
    
    Ok(())
}

/// Prepare log files (cleanup and rotation)
fn prepare_log_files(log_dir: &PathBuf, debug_log_path: &PathBuf, error_log_path: &PathBuf) {
    let _ = file_ops::cleanup_old_logs(log_dir);
    let _ = file_ops::rotate_log_file(debug_log_path, 5);
    let _ = file_ops::rotate_log_file(error_log_path, 5);
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
