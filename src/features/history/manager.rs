use std::fs;
use std::path::{Path, PathBuf};
use super::entry::HistoryEntry;

/// Maximum number of commands to keep in history
const MAX_HISTORY_SIZE: usize = 100;

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
    /// If a matching command already exists, it will be moved to the top instead of creating a duplicate
    pub fn add(&mut self, entry: HistoryEntry) {
        remove_duplicate_entry(&mut self.entries, &entry);
        add_entry_to_top(&mut self.entries, entry);
        trim_history(&mut self.entries);
        self.save();
    }

    /// Get all history entries
    pub fn entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    /// Save history to disk
    fn save(&self) {
        save_history_to_file(&self.file_path, &self.entries);
    }

    /// Get the path to the history file
    fn get_history_file_path() -> PathBuf {
        get_config_file_path("command_history.json")
    }

    /// Clear all history
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.entries.clear();
        self.save();
    }
}

/// Remove duplicate entry from history if it exists
fn remove_duplicate_entry(entries: &mut Vec<HistoryEntry>, new_entry: &HistoryEntry) {
    if let Some(existing_idx) = find_matching_entry_index(entries, new_entry) {
        entries.remove(existing_idx);
        log::debug!(
            "Removed duplicate history entry at index {} (moving to top)",
            existing_idx
        );
    }
}

/// Find index of matching entry in history
fn find_matching_entry_index(entries: &[HistoryEntry], target: &HistoryEntry) -> Option<usize> {
    entries.iter().position(|e| e.matches(target))
}

/// Add entry to top of history
fn add_entry_to_top(entries: &mut Vec<HistoryEntry>, entry: HistoryEntry) {
    entries.insert(0, entry);
}

/// Trim history to maximum size
fn trim_history(entries: &mut Vec<HistoryEntry>) {
    if entries.len() > MAX_HISTORY_SIZE {
        entries.truncate(MAX_HISTORY_SIZE);
    }
}

/// Save history entries to file
fn save_history_to_file(file_path: &PathBuf, entries: &[HistoryEntry]) {
    ensure_parent_dir_exists(file_path);
    
    match serde_json::to_string_pretty(entries) {
        Ok(json) => write_json_to_file(file_path, &json),
        Err(e) => log::error!("Failed to serialize command history: {}", e),
    }
}

/// Ensure parent directory exists
fn ensure_parent_dir_exists(file_path: &Path) {
    if let Some(parent) = file_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
}

/// Write JSON string to file
fn write_json_to_file(file_path: &PathBuf, json: &str) {
    if let Err(e) = fs::write(file_path, json) {
        log::error!("Failed to save command history: {}", e);
    }
}

/// Get path to config file
fn get_config_file_path(filename: &str) -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("lazymvn");
    config_dir.join(filename)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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

    #[test]
    fn command_history_deduplicates_entries() {
        let mut history = CommandHistory::default();

        history.add(HistoryEntry::new(
            PathBuf::from("/tmp/project"),
            "module1".to_string(),
            "test".to_string(),
            vec!["dev".to_string()],
            vec!["-X".to_string()],
        ));

        history.add(HistoryEntry::new(
            PathBuf::from("/tmp/project"),
            "module2".to_string(),
            "package".to_string(),
            vec![],
            vec![],
        ));

        history.add(HistoryEntry::new(
            PathBuf::from("/tmp/project"),
            "module1".to_string(),
            "test".to_string(),
            vec!["dev".to_string()],
            vec!["-X".to_string()],
        ));

        let entries = history.entries();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].module, "module1");
        assert_eq!(entries[0].goal, "test");
        assert_eq!(entries[1].module, "module2");
        assert_eq!(entries[1].goal, "package");
    }

    #[test]
    fn command_history_deduplication_updates_position() {
        let mut history = CommandHistory::default();

        history.add(HistoryEntry::new(
            PathBuf::from("/tmp/project"),
            "module1".to_string(),
            "test".to_string(),
            vec![],
            vec![],
        ));

        history.add(HistoryEntry::new(
            PathBuf::from("/tmp/project"),
            "module2".to_string(),
            "compile".to_string(),
            vec![],
            vec![],
        ));

        history.add(HistoryEntry::new(
            PathBuf::from("/tmp/project"),
            "module3".to_string(),
            "package".to_string(),
            vec![],
            vec![],
        ));

        history.add(HistoryEntry::new(
            PathBuf::from("/tmp/project"),
            "module1".to_string(),
            "test".to_string(),
            vec![],
            vec![],
        ));

        let entries = history.entries();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].module, "module1");
        assert_eq!(entries[1].module, "module3");
        assert_eq!(entries[2].module, "module2");
    }

    #[test]
    fn test_trim_history_under_limit() {
        let mut entries = vec![
            HistoryEntry::new(PathBuf::from("/p"), "m".to_string(), "g".to_string(), vec![], vec![]),
        ];
        trim_history(&mut entries);
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_trim_history_at_limit() {
        let mut entries: Vec<HistoryEntry> = (0..MAX_HISTORY_SIZE)
            .map(|_| HistoryEntry::new(PathBuf::from("/p"), "m".to_string(), "g".to_string(), vec![], vec![]))
            .collect();
        trim_history(&mut entries);
        assert_eq!(entries.len(), MAX_HISTORY_SIZE);
    }

    #[test]
    fn test_trim_history_over_limit() {
        let mut entries: Vec<HistoryEntry> = (0..MAX_HISTORY_SIZE + 10)
            .map(|_| HistoryEntry::new(PathBuf::from("/p"), "m".to_string(), "g".to_string(), vec![], vec![]))
            .collect();
        trim_history(&mut entries);
        assert_eq!(entries.len(), MAX_HISTORY_SIZE);
    }

    #[test]
    fn test_find_matching_entry_index_found() {
        let entry1 = HistoryEntry::new(
            PathBuf::from("/p"),
            "m1".to_string(),
            "g".to_string(),
            vec![],
            vec![],
        );
        let entry2 = HistoryEntry::new(
            PathBuf::from("/p"),
            "m2".to_string(),
            "g".to_string(),
            vec![],
            vec![],
        );
        let entries = vec![entry1.clone(), entry2.clone()];
        
        assert_eq!(find_matching_entry_index(&entries, &entry1), Some(0));
        assert_eq!(find_matching_entry_index(&entries, &entry2), Some(1));
    }

    #[test]
    fn test_find_matching_entry_index_not_found() {
        let entry1 = HistoryEntry::new(
            PathBuf::from("/p"),
            "m1".to_string(),
            "g".to_string(),
            vec![],
            vec![],
        );
        let entry2 = HistoryEntry::new(
            PathBuf::from("/p"),
            "m2".to_string(),
            "g".to_string(),
            vec![],
            vec![],
        );
        let entries = vec![entry1];
        
        assert_eq!(find_matching_entry_index(&entries, &entry2), None);
    }
}
