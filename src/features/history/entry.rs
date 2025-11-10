use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A command in the history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp: i64,
    pub project_root: PathBuf,
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
        use super::formatters::{build_command_parts, format_module_name};
        let parts = build_command_parts(&self.goal, &self.profiles, &self.flags);
        let module_display = format_module_name(&self.module);
        format!("[{}] {}", module_display, parts.join(" "))
    }

    /// Format timestamp for display
    pub fn format_time(&self) -> String {
        super::formatters::format_timestamp(self.timestamp)
    }

    /// Check if this entry matches another (ignoring timestamp)
    pub fn matches(&self, other: &HistoryEntry) -> bool {
        self.project_root == other.project_root
            && self.module == other.module
            && self.goal == other.goal
            && self.profiles == other.profiles
            && self.flags == other.flags
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
    fn history_entry_matches_ignores_timestamp() {
        let entry1 = HistoryEntry {
            timestamp: 1000,
            project_root: PathBuf::from("/tmp/project"),
            module: "module1".to_string(),
            goal: "test".to_string(),
            profiles: vec!["dev".to_string()],
            flags: vec!["-X".to_string()],
        };

        let entry2 = HistoryEntry {
            timestamp: 2000,
            project_root: PathBuf::from("/tmp/project"),
            module: "module1".to_string(),
            goal: "test".to_string(),
            profiles: vec!["dev".to_string()],
            flags: vec!["-X".to_string()],
        };

        assert!(entry1.matches(&entry2));
    }

    #[test]
    fn history_entry_does_not_match_different_module() {
        let entry1 = HistoryEntry::new(
            PathBuf::from("/tmp/project"),
            "module1".to_string(),
            "test".to_string(),
            vec![],
            vec![],
        );

        let entry2 = HistoryEntry::new(
            PathBuf::from("/tmp/project"),
            "module2".to_string(),
            "test".to_string(),
            vec![],
            vec![],
        );

        assert!(!entry1.matches(&entry2));
    }

    #[test]
    fn history_entry_does_not_match_different_profiles() {
        let entry1 = HistoryEntry::new(
            PathBuf::from("/tmp/project"),
            "module1".to_string(),
            "test".to_string(),
            vec!["dev".to_string()],
            vec![],
        );

        let entry2 = HistoryEntry::new(
            PathBuf::from("/tmp/project"),
            "module1".to_string(),
            "test".to_string(),
            vec!["prod".to_string()],
            vec![],
        );

        assert!(!entry1.matches(&entry2));
    }
}
