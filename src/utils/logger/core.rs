use log::{Metadata, Record};
use std::fs::File;
use std::io::Write;
use std::sync::Mutex;

/// Logger implementation with file and error file support
pub struct Logger {
    pub file: Mutex<Option<File>>,
    pub error_file: Mutex<Option<File>>,
    pub session_id: Mutex<Option<String>>,
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let log_line = super::formatters::format_log_line(record, &self.session_id.lock().unwrap());
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
