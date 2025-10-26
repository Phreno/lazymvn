//! Tab management for multi-project support
//!
//! This module handles creation, switching, and cleanup of project tabs.

use super::{TuiState, ProjectTab};
use std::path::PathBuf;

impl TuiState {
    /// Create a new tab for a project
    pub fn create_tab(&mut self, project_root: PathBuf) -> Result<usize, String> {
        // Check if project is already open in a tab
        if let Some(existing_index) = self.find_tab_by_project(&project_root) {
            log::info!(
                "Project already open in tab {}, switching to it",
                existing_index
            );
            self.active_tab_index = existing_index;
            return Ok(existing_index);
        }

        // Check maximum tabs limit
        const MAX_TABS: usize = 10;
        if self.tabs.len() >= MAX_TABS {
            return Err(format!(
                "Maximum {} onglets atteints. Fermez-en un avec Ctrl+W",
                MAX_TABS
            ));
        }

        // Load project modules
        let (modules, resolved_root) =
            crate::core::project::get_project_modules_for_path(&project_root)
                .map_err(|e| format!("Failed to load project: {}", e))?;

        // Load project config
        let config = crate::core::config::load_config(&resolved_root);

        // Create the tab
        let tab = ProjectTab::new(self.next_tab_id, resolved_root.clone(), modules, config);
        log::info!(
            "Created tab {} for project: {:?}",
            self.next_tab_id,
            resolved_root
        );

        self.next_tab_id += 1;
        self.tabs.push(tab);
        self.active_tab_index = self.tabs.len() - 1;

        // Add to recent projects
        let mut recent = crate::core::config::RecentProjects::load();
        recent.add(resolved_root);
        self.recent_projects = recent.get_projects();

        // Load profiles asynchronously for the new tab
        self.start_loading_profiles();

        Ok(self.active_tab_index)
    }

    /// Close a tab by index
    pub fn close_tab(&mut self, index: usize) -> Result<(), String> {
        if index >= self.tabs.len() {
            return Err("Tab index out of bounds".to_string());
        }

        if self.tabs.len() == 1 {
            return Err("Cannot close last tab".to_string());
        }

        // Cleanup the tab (kill processes, save preferences)
        self.tabs[index].cleanup();

        // Remove the tab
        let removed_tab = self.tabs.remove(index);
        log::info!(
            "Closed tab {} ({})",
            removed_tab.id,
            removed_tab.get_title()
        );

        // Adjust active tab index
        if self.active_tab_index >= self.tabs.len() {
            self.active_tab_index = self.tabs.len() - 1;
        } else if self.active_tab_index > index {
            self.active_tab_index -= 1;
        }

        Ok(())
    }

    /// Switch to a specific tab by index
    #[allow(dead_code)] // Public API - may be used by future features
    pub fn switch_to_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            log::debug!(
                "Switching from tab {} to tab {}",
                self.active_tab_index,
                index
            );
            self.active_tab_index = index;
        }
    }

    /// Switch to next tab
    pub fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab_index = (self.active_tab_index + 1) % self.tabs.len();
            log::debug!("Switched to next tab: {}", self.active_tab_index);
        }
    }

    /// Switch to previous tab
    pub fn prev_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab_index = if self.active_tab_index == 0 {
                self.tabs.len() - 1
            } else {
                self.active_tab_index - 1
            };
            log::debug!("Switched to previous tab: {}", self.active_tab_index);
        }
    }

    /// Get the active tab (immutable)
    pub fn get_active_tab(&self) -> &ProjectTab {
        &self.tabs[self.active_tab_index]
    }

    /// Get the active tab (mutable)
    pub fn get_active_tab_mut(&mut self) -> &mut ProjectTab {
        &mut self.tabs[self.active_tab_index]
    }

    /// Get the number of tabs
    pub fn get_tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Get all tabs (for rendering)
    pub fn get_tabs(&self) -> &[ProjectTab] {
        &self.tabs
    }

    /// Get the active tab index
    pub fn get_active_tab_index(&self) -> usize {
        self.active_tab_index
    }

    /// Check if any tab has a running process
    #[allow(dead_code)] // Public API - may be used by future features
    pub fn has_running_processes(&self) -> bool {
        self.tabs.iter().any(|tab| tab.has_running_process())
    }

    /// Count running processes across all tabs
    #[allow(dead_code)] // Public API - may be used by future features
    pub fn count_running_processes(&self) -> usize {
        self.tabs
            .iter()
            .filter(|tab| tab.has_running_process())
            .count()
    }

    /// Find a tab by project root path
    fn find_tab_by_project(&self, project_root: &PathBuf) -> Option<usize> {
        self.tabs
            .iter()
            .position(|tab| &tab.project_root == project_root)
    }

    /// Cleanup all tabs (kill all processes, save all preferences)
    #[allow(dead_code)] // Public API - may be used by future features
    pub fn cleanup_all_tabs(&mut self) {
        log::info!("Cleaning up all {} tabs", self.tabs.len());

        for tab in &mut self.tabs {
            tab.cleanup();
        }

        log::info!("All tabs cleaned up");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;

    fn create_test_state() -> TuiState {
        TuiState::new(
            vec!["module1".to_string()],
            PathBuf::from("/test"),
            Config::default(),
        )
    }

    #[test]
    fn test_get_active_tab() {
        let state = create_test_state();
        let tab = state.get_active_tab();
        assert_eq!(tab.modules.len(), 1);
    }

    #[test]
    fn test_get_active_tab_mut() {
        let mut state = create_test_state();
        let tab = state.get_active_tab_mut();
        tab.modules.push("module2".to_string());
        assert_eq!(state.get_active_tab().modules.len(), 2);
    }

    #[test]
    fn test_get_tab_count() {
        let state = create_test_state();
        assert_eq!(state.get_tab_count(), 1);
    }

    #[test]
    fn test_get_tabs() {
        let state = create_test_state();
        let tabs = state.get_tabs();
        assert_eq!(tabs.len(), 1);
    }

    #[test]
    fn test_get_active_tab_index() {
        let state = create_test_state();
        assert_eq!(state.get_active_tab_index(), 0);
    }

    #[test]
    fn test_switch_to_tab() {
        let mut state = create_test_state();
        // Create with one tab, switch should be safe even with same index
        state.switch_to_tab(0);
        assert_eq!(state.active_tab_index, 0);
    }

    #[test]
    fn test_next_tab_single() {
        let mut state = create_test_state();
        state.next_tab();
        // With single tab, should stay at 0
        assert_eq!(state.active_tab_index, 0);
    }

    #[test]
    fn test_prev_tab_single() {
        let mut state = create_test_state();
        state.prev_tab();
        // With single tab, should stay at 0
        assert_eq!(state.active_tab_index, 0);
    }

    #[test]
    fn test_close_tab_last_tab_fails() {
        let mut state = create_test_state();
        let result = state.close_tab(0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cannot close last tab");
    }

    #[test]
    fn test_close_tab_out_of_bounds() {
        let mut state = create_test_state();
        let result = state.close_tab(5);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Tab index out of bounds");
    }

    #[test]
    fn test_has_running_processes_false() {
        let state = create_test_state();
        assert!(!state.has_running_processes());
    }

    #[test]
    fn test_count_running_processes_zero() {
        let state = create_test_state();
        assert_eq!(state.count_running_processes(), 0);
    }

    #[test]
    fn test_cleanup_all_tabs() {
        let mut state = create_test_state();
        // Should not panic
        state.cleanup_all_tabs();
    }

    #[test]
    fn test_find_tab_by_project_found() {
        let state = create_test_state();
        let result = state.find_tab_by_project(&PathBuf::from("/test"));
        assert_eq!(result, Some(0));
    }

    #[test]
    fn test_find_tab_by_project_not_found() {
        let state = create_test_state();
        let result = state.find_tab_by_project(&PathBuf::from("/other"));
        assert!(result.is_none());
    }
}

