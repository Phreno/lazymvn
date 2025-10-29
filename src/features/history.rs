use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Maximum number of commands to keep in history
const MAX_HISTORY_SIZE: usize = 100;

/// A command in the history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp: i64,
    pub project_root: PathBuf,  // Added to track which project this command belongs to
    pub module: String,
    pub goal: String,
    pub profiles: Vec<String>,
    pub flags: Vec<String>,
}

impl HistoryEntry {
    /// Create a new history entry
    pub fn new(
        project_root: PathBuf,
        module: String,
        goal: String,
        profiles: Vec<String>,
        flags: Vec<String>,
    ) -> Self {
        Self {
            timestamp: chrono::Utc::now().timestamp(),
            project_root,
            module,
            goal,
            profiles,
            flags,
        }
    }

    /// Format the entry for display
    pub fn format_command(&self) -> String {
        let mut parts = vec![self.goal.clone()];

        if !self.profiles.is_empty() {
            parts.push(format!("-P {}", self.profiles.join(",")));
        }

        parts.extend(self.flags.clone());

        let module_display = if self.module == "." {
            "(root)".to_string()
        } else {
            self.module.clone()
        };

        format!("[{}] {}", module_display, parts.join(" "))
    }

    /// Format timestamp for display
    pub fn format_time(&self) -> String {
        use chrono::TimeZone;
        let dt = chrono::Utc.timestamp_opt(self.timestamp, 0).unwrap();
        let local_dt = dt.with_timezone(&chrono::Local);
        local_dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}

/// Command history manager
#[derive(Debug, Default)]
pub struct CommandHistory {
    entries: Vec<HistoryEntry>,
    file_path: PathBuf,
}

impl CommandHistory {
    /// Load command history from disk
    pub fn load() -> Self {
        let file_path = Self::get_history_file_path();

        let entries = if file_path.exists() {
            match fs::read_to_string(&file_path) {
                Ok(contents) => serde_json::from_str(&contents).unwrap_or_else(|e| {
                    log::warn!("Failed to parse command history: {}", e);
                    Vec::new()
                }),
                Err(e) => {
                    log::warn!("Failed to read command history: {}", e);
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };

        Self { entries, file_path }
    }

    /// Add a command to history
    pub fn add(&mut self, entry: HistoryEntry) {
        // Add to beginning (most recent first)
        self.entries.insert(0, entry);

        // Trim to max size
        if self.entries.len() > MAX_HISTORY_SIZE {
            self.entries.truncate(MAX_HISTORY_SIZE);
        }

        // Save to disk
        self.save();
    }

    /// Get all history entries
    pub fn entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    /// Save history to disk
    fn save(&self) {
        // Ensure directory exists
        if let Some(parent) = self.file_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        match serde_json::to_string_pretty(&self.entries) {
            Ok(json) => {
                if let Err(e) = fs::write(&self.file_path, json) {
                    log::error!("Failed to save command history: {}", e);
                }
            }
            Err(e) => {
                log::error!("Failed to serialize command history: {}", e);
            }
        }
    }

    /// Get the path to the history file
    fn get_history_file_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("lazymvn");
        config_dir.join("command_history.json")
    }

    /// Clear all history
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.entries.clear();
        self.save();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_entry_format_command_with_profiles_and_flags() {
        let entry = HistoryEntry::new(
            PathBuf::from("/tmp/project"),
            "my-module".to_string(),
            "clean install".to_string(),
            vec!["dev".to_string(), "test".to_string()],
            vec!["--also-make".to_string(), "-DskipTests".to_string()],
        );

        let formatted = entry.format_command();
        assert!(formatted.contains("[my-module]"));
        assert!(formatted.contains("clean install"));
        assert!(formatted.contains("-P dev,test"));
        assert!(formatted.contains("--also-make"));
        assert!(formatted.contains("-DskipTests"));
    }

    #[test]
    fn history_entry_format_command_root_module() {
        let entry = HistoryEntry::new(
            PathBuf::from("/tmp/project"),
            ".".to_string(),
            "package".to_string(),
            vec![],
            vec![],
        );

        let formatted = entry.format_command();
        assert!(formatted.contains("(root)"));
        assert!(formatted.contains("package"));
    }

    #[test]
    fn command_history_add_maintains_order() {
        let mut history = CommandHistory::default();

        history.add(HistoryEntry::new(
            PathBuf::from("/tmp/project"),
            "module1".to_string(),
            "goal1".to_string(),
            vec![],
            vec![],
        ));

        history.add(HistoryEntry::new(
            PathBuf::from("/tmp/project"),
            "module2".to_string(),
            "goal2".to_string(),
            vec![],
            vec![],
        ));

        let entries = history.entries();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].module, "module2");
        assert_eq!(entries[1].module, "module1");
    }

    #[test]
    fn command_history_respects_max_size() {
        let mut history = CommandHistory::default();

        for i in 0..150 {
            history.add(HistoryEntry::new(
                PathBuf::from("/tmp/project"),
                format!("module{}", i),
                "goal".to_string(),
                vec![],
                vec![],
            ));
        }

        assert_eq!(history.entries().len(), MAX_HISTORY_SIZE);
    }
}
