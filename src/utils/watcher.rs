//! File watching for auto-reload functionality

use globset::{Glob, GlobSet, GlobSetBuilder};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    receiver: mpsc::Receiver<notify::Result<Event>>,
    last_event: Option<Instant>,
    debounce_duration: Duration,
    watch_root: PathBuf,
    patterns: Vec<String>,
    pattern_set: Option<GlobSet>,
}

impl FileWatcher {
    /// Create a new file watcher for the given directory
    pub fn new(
        watch_dir: &Path,
        patterns: Vec<String>,
        debounce_ms: u64,
    ) -> Result<Self, notify::Error> {
        let (tx, rx) = mpsc::channel();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default(),
        )?;

        watcher.watch(watch_dir, RecursiveMode::Recursive)?;
        log::info!("File watcher initialized for: {:?}", watch_dir);

        let normalized_patterns = normalize_patterns(patterns);
        let pattern_set = build_glob_set(&normalized_patterns);

        Ok(Self {
            _watcher: watcher,
            receiver: rx,
            last_event: None,
            debounce_duration: Duration::from_millis(debounce_ms),
            watch_root: watch_dir.to_path_buf(),
            patterns: normalized_patterns,
            pattern_set,
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
                if self.event_matches_patterns(&event) {
                    log::debug!("File change detected: {:?}", event.paths);
                    has_changes = true;
                } else if log::log_enabled!(log::Level::Trace) {
                    log::trace!("Ignoring file change (no pattern match): {:?}", event.paths);
                }
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

    fn event_matches_patterns(&self, event: &Event) -> bool {
        if self.patterns.is_empty() {
            return true;
        }

        for path in &event.paths {
            let relative = path.strip_prefix(&self.watch_root).unwrap_or(path);

            if let Some(set) = &self.pattern_set {
                if set.is_match(relative) {
                    return true;
                }
            } else if matches_patterns(relative, &self.patterns) {
                return true;
            }
        }

        false
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
    match build_glob_set(patterns) {
        Some(set) => set.is_match(path),
        None => true,
    }
}

fn normalize_patterns(patterns: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();

    for pattern in patterns {
        let mut pattern = pattern.replace('\\', "/").trim().to_string();
        if pattern.is_empty() {
            continue;
        }

        if pattern.starts_with("./") {
            pattern = pattern.trim_start_matches("./").to_string();
        }

        if !normalized.contains(&pattern) {
            normalized.push(pattern.clone());
        }

        if !pattern.starts_with("**/") {
            let prefixed = format!("**/{}", pattern);
            if !normalized.contains(&prefixed) {
                normalized.push(prefixed);
            }
        }
    }

    normalized
}

fn build_glob_set(patterns: &[String]) -> Option<GlobSet> {
    if patterns.is_empty() {
        return None;
    }

    let mut builder = GlobSetBuilder::new();
    let mut added = false;

    for pattern in patterns {
        match Glob::new(pattern) {
            Ok(glob) => {
                builder.add(glob);
                added = true;
            }
            Err(e) => {
                log::warn!("Invalid watch pattern '{}': {}", pattern, e);
            }
        }
    }

    if added {
        match builder.build() {
            Ok(set) => Some(set),
            Err(e) => {
                log::error!("Failed to build watch pattern set: {}", e);
                None
            }
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_patterns_supports_nested_paths() {
        let patterns = vec!["src/**/*.java".to_string()];
        assert!(matches_patterns(
            Path::new("src/main/java/App.java"),
            &patterns
        ));
        assert!(matches_patterns(
            Path::new("app/src/main/java/App.java"),
            &patterns
        ));
        assert!(!matches_patterns(
            Path::new("app/target/classes/App.class"),
            &patterns
        ));
    }

    #[test]
    fn matches_patterns_handles_windows_separators() {
        let patterns = vec!["src/**/*.yaml".to_string()];
        assert!(matches_patterns(
            Path::new("src\\main\\resources\\application.yaml"),
            &patterns
        ));
    }
}
