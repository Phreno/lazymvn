use log::{LevelFilter, Metadata, Record, SetLoggerError};
use std::fs::{File, OpenOptions};
use std::io::Write;
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
            let log_line = format!(
                "[{}] {} - {}",
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

pub fn init(debug: bool) -> Result<(), SetLoggerError> {
    if debug {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("lazymvn-debug.log")
            .expect("Failed to open log file");

        let error_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("lazymvn-error.log")
            .expect("Failed to open error log file");

        *LOGGER.file.lock().unwrap() = Some(file);
        *LOGGER.error_file.lock().unwrap() = Some(error_file);

        log::set_logger(&LOGGER)?;
        log::set_max_level(LevelFilter::Debug);

        log::info!("Debug logging enabled");
        log::info!("Error logging enabled - check lazymvn-error.log");
    } else {
        log::set_max_level(LevelFilter::Off);
    }

    Ok(())
}
