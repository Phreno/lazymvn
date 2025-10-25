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
    pub fn has_running_processes(&self) -> bool {
        self.tabs.iter().any(|tab| tab.has_running_process())
    }

    /// Count running processes across all tabs
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
    pub fn cleanup_all_tabs(&mut self) {
        log::info!("Cleaning up all {} tabs", self.tabs.len());

        for tab in &mut self.tabs {
            tab.cleanup();
        }

        log::info!("All tabs cleaned up");
    }
}
