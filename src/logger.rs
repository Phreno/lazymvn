use log::{LevelFilter, Metadata, Record, SetLoggerError};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

static LOGGER: Logger = Logger {
    file: Mutex::new(None),
    error_file: Mutex::new(None),
};

struct Logger {
    file: Mutex<Option<File>>,
    error_file: Mutex<Option<File>>,
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let log_line = format!("[{}] {} - {}", timestamp, record.level(), record.args());

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
    let dirs = directories::ProjectDirs::from("com", "lazymvn", "lazymvn")
        .ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine home directory",
        ))?;

    let log_dir = dirs.data_local_dir().join("logs");
    
    // Create the log directory if it doesn't exist
    std::fs::create_dir_all(&log_dir)?;
    
    Ok(log_dir)
}

/// Get the path to the debug log file
pub fn get_debug_log_path() -> Option<PathBuf> {
    get_log_dir().ok().map(|dir| dir.join("debug.log"))
}

/// Get the path to the error log file
pub fn get_error_log_path() -> Option<PathBuf> {
    get_log_dir().ok().map(|dir| dir.join("error.log"))
}

pub fn init(debug: bool) -> Result<(), SetLoggerError> {
    if debug {
        let log_dir = get_log_dir().expect("Failed to get log directory");
        
        let debug_log_path = log_dir.join("debug.log");
        let error_log_path = log_dir.join("error.log");

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

        *LOGGER.file.lock().unwrap() = Some(file);
        *LOGGER.error_file.lock().unwrap() = Some(error_file);

        log::set_logger(&LOGGER)?;
        log::set_max_level(LevelFilter::Debug);

        log::info!("Debug logging enabled");
        log::info!("Log directory: {}", log_dir.display());
        log::info!("Debug log: {}", debug_log_path.display());
        log::info!("Error log: {}", error_log_path.display());
    } else {
        log::set_max_level(LevelFilter::Off);
    }

    Ok(())
}
