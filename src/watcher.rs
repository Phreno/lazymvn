//! File watching for auto-reload functionality

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    receiver: mpsc::Receiver<notify::Result<Event>>,
    last_event: Option<Instant>,
    debounce_duration: Duration,
}

impl FileWatcher {
    /// Create a new file watcher for the given directory
    pub fn new(watch_dir: &Path, debounce_ms: u64) -> Result<Self, notify::Error> {
        let (tx, rx) = mpsc::channel();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default(),
        )?;

        watcher.watch(watch_dir, RecursiveMode::Recursive)?;
        log::info!("File watcher initialized for: {:?}", watch_dir);

        Ok(Self {
            _watcher: watcher,
            receiver: rx,
            last_event: None,
            debounce_duration: Duration::from_millis(debounce_ms),
        })
    }

    /// Check if there are any file changes (with debouncing)
    /// Returns true if files changed and debounce period elapsed
    pub fn check_changes(&mut self) -> bool {
        let mut has_changes = false;

        // Drain all pending events
        while let Ok(event) = self.receiver.try_recv() {
            if let Ok(event) = event
                && is_relevant_event(&event)
            {
                log::debug!("File change detected: {:?}", event.paths);
                has_changes = true;
            }
        }

        if !has_changes {
            return false;
        }

        // Check debounce
        let now = Instant::now();
        if let Some(last) = self.last_event
            && now.duration_since(last) < self.debounce_duration
        {
            return false; // Too soon, still debouncing
        }

        self.last_event = Some(now);
        true
    }
}

/// Check if the event is relevant (modify, create, remove)
fn is_relevant_event(event: &Event) -> bool {
    use notify::EventKind;
    
    matches!(
        event.kind,
        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
    )
}

/// Check if a path matches any of the given patterns
#[allow(dead_code)]
pub fn matches_patterns(path: &Path, patterns: &[String]) -> bool {
    let path_str = path.to_string_lossy();
    
    for pattern in patterns {
        if matches_glob(&path_str, pattern) {
            return true;
        }
    }
    
    false
}

/// Simple glob pattern matching
/// Supports: *.ext, **/*.ext, path/to/*.ext
fn matches_glob(path: &str, pattern: &str) -> bool {
    // Convert glob pattern to regex-like matching
    if pattern.contains("**") {
        // Recursive wildcard: src/**/*.java
        let parts: Vec<&str> = pattern.split("**").collect();
        if parts.len() == 2 {
            let prefix = parts[0];
            let suffix = parts[1].trim_start_matches('/');
            
            if let Some(remaining) = path.strip_prefix(prefix) {
                return matches_glob(remaining, suffix);
            }
        }
    } else if pattern.contains('*') {
        // Single wildcard: *.java or path/*.java
        if pattern.starts_with('*') {
            // *.java
            let ext = pattern.trim_start_matches('*');
            return path.ends_with(ext);
        } else {
            // path/*.java
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                return path.starts_with(parts[0]) && path.ends_with(parts[1]);
            }
        }
    } else {
        // Exact match
        return path == pattern;
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_matching() {
        assert!(matches_glob("test.java", "*.java"));
        assert!(matches_glob("src/main/Test.java", "*.java"));
        assert!(!matches_glob("test.rs", "*.java"));
        
        assert!(matches_glob("src/main/java/Test.java", "src/**/*.java"));
        assert!(matches_glob("src/test/resources/app.properties", "src/**/*.properties"));
        assert!(!matches_glob("target/test.java", "src/**/*.java"));
    }
}
