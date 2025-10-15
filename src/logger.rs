use log::{LevelFilter, Metadata, Record, SetLoggerError};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Mutex;

static LOGGER: Logger = Logger {
    file: Mutex::new(None),
};

struct Logger {
    file: Mutex<Option<File>>,
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut file_guard = self.file.lock().unwrap();
            if let Some(file) = file_guard.as_mut() {
                let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
                let _ = writeln!(
                    file,
                    "[{}] {} - {}",
                    timestamp,
                    record.level(),
                    record.args()
                );
            }
        }
    }

    fn flush(&self) {
        let mut file_guard = self.file.lock().unwrap();
        if let Some(file) = file_guard.as_mut() {
            let _ = file.flush();
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

        *LOGGER.file.lock().unwrap() = Some(file);

        log::set_logger(&LOGGER)?;
        log::set_max_level(LevelFilter::Debug);

        log::info!("Debug logging enabled");
    } else {
        log::set_max_level(LevelFilter::Off);
    }

    Ok(())
}
