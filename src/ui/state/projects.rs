//! Recent projects management for TUI

use super::TuiState;
use crate::ui::keybindings::Focus;
use std::path::PathBuf;

impl TuiState {
    pub fn show_recent_projects(&mut self) {
        log::info!("Showing recent projects popup");
        self.show_projects_popup = true;
        self.projects_filter.clear();
        if self.focus != Focus::Projects {
            self.focus = Focus::Projects;
        }
    }

    pub fn hide_recent_projects(&mut self) {
        log::info!("Hiding recent projects popup");
        self.show_projects_popup = false;
        self.projects_filter.clear();
    }

    /// Get filtered recent projects based on current filter
    pub fn get_filtered_projects(&self) -> Vec<PathBuf> {
        if self.projects_filter.is_empty() {
            self.recent_projects.clone()
        } else {
            let filter = self.projects_filter.to_lowercase();
            self.recent_projects
                .iter()
                .filter(|p| {
                    p.to_string_lossy().to_lowercase().contains(&filter)
                })
                .cloned()
                .collect()
        }
    }

    /// Push character to projects filter
    pub fn push_projects_filter_char(&mut self, ch: char) {
        self.projects_filter.push(ch);
        // Reset selection when filter changes
        if !self.get_filtered_projects().is_empty() {
            self.projects_list_state.select(Some(0));
        }
    }

    /// Pop character from projects filter
    pub fn pop_projects_filter_char(&mut self) {
        self.projects_filter.pop();
        // Reset selection when filter changes
        if !self.get_filtered_projects().is_empty() {
            self.projects_list_state.select(Some(0));
        }
    }

    pub fn select_current_project(&mut self) {
        if let Some(idx) = self.projects_list_state.selected() {
            let filtered_projects = self.get_filtered_projects();
            if let Some(project) = filtered_projects.get(idx) {
                log::info!("Selected project: {:?}", project);
                match self.create_tab(project.clone()) {
                    Ok(tab_idx) => {
                        log::info!("Opened project in tab {}", tab_idx);
                    }
                    Err(e) => {
                        log::error!("Failed to create tab: {}", e);
                        if let Some(tab) = self.tabs.get_mut(self.active_tab_index) {
                            tab.command_output = vec![format!("❌ {}", e)];
                        }
                    }
                }
                self.hide_recent_projects();
            }
        }
    }

    pub fn next_project(&mut self) {
        let filtered = self.get_filtered_projects();
        if filtered.is_empty() {
            return;
        }
        let i = match self.projects_list_state.selected() {
            Some(i) => (i + 1) % filtered.len(),
            None => 0,
        };
        self.projects_list_state.select(Some(i));
    }

    pub fn previous_project(&mut self) {
        let filtered = self.get_filtered_projects();
        if filtered.is_empty() {
            return;
        }
        let i = match self.projects_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    filtered.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.projects_list_state.select(Some(i));
    }
}
