//! Command history management for TUI

use super::TuiState;

impl TuiState {
    pub fn apply_history_entry(&mut self, entry: crate::features::history::HistoryEntry) {
        log::info!(
            "Applying history entry for project: {:?}, module: {}",
            entry.project_root,
            entry.module
        );

        // Check if we need to switch project context
        let current_project_root = self.get_active_tab().project_root.clone();

        if current_project_root != entry.project_root {
            log::info!(
                "History entry is for a different project. Current: {:?}, Required: {:?}",
                current_project_root,
                entry.project_root
            );

            // Check if there's already a tab with this project
            let existing_tab_index = self
                .tabs
                .iter()
                .position(|tab| tab.project_root == entry.project_root);

            if let Some(tab_index) = existing_tab_index {
                // Switch to existing tab
                log::info!("Switching to existing tab at index {}", tab_index);
                self.active_tab_index = tab_index;
            } else {
                // Try to open a new tab with this project
                log::info!("Opening new tab for project: {:?}", entry.project_root);

                // Load project modules
                match crate::core::project::get_project_modules_for_path(&entry.project_root) {
                    Ok((modules, root)) => {
                        // Load config for this project
                        let config = crate::core::config::load_config(&root);

                        // Create new tab - ProjectTab::new doesn't return Result, it's infallible
                        let new_tab_id = self.tabs.len();
                        let new_tab = crate::ui::state::ProjectTab::new(
                            new_tab_id,
                            root,
                            modules,
                            config,
                        );

                        if self.tabs.len() < 10 {
                            self.tabs.push(new_tab);
                            self.active_tab_index = self.tabs.len() - 1;
                            log::info!("New tab created successfully");
                        } else {
                            log::error!("Cannot create new tab: maximum 10 tabs reached");
                            let tab = self.get_active_tab_mut();
                            tab.command_output = vec![
                                format!(
                                    "Error: Cannot switch to project {:?}",
                                    entry.project_root
                                ),
                                "Maximum 10 tabs reached. Close some tabs first.".to_string(),
                            ];
                            return;
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to load project modules: {}", e);
                        let tab = self.get_active_tab_mut();
                        tab.command_output = vec![
                            format!("Error: Failed to load project {:?}: {}", entry.project_root, e),
                        ];
                        return;
                    }
                }
            }
        }

        // Now we're in the correct project context, apply the command
        let tab = self.get_active_tab_mut();

        // Find and select the module
        if let Some(module_idx) = tab.modules.iter().position(|m| m == &entry.module) {
            tab.modules_list_state.select(Some(module_idx));
            log::debug!("Selected module at index {}", module_idx);
        } else {
            log::warn!("Module '{}' not found in project", entry.module);
            tab.command_output = vec![format!(
                "Error: Module '{}' not found in this project",
                entry.module
            )];
            return;
        }

        // Set profiles
        for profile in &mut tab.profiles {
            if entry.profiles.contains(&profile.name) {
                // Should be enabled
                if !profile.is_active() {
                    profile.state = crate::ui::state::ProfileState::ExplicitlyEnabled;
                }
            } else {
                // Should be disabled or default
                if profile.auto_activated {
                    // If auto-activated but not in history, explicitly disable
                    profile.state = crate::ui::state::ProfileState::ExplicitlyDisabled;
                } else {
                    // Otherwise set to default
                    profile.state = crate::ui::state::ProfileState::Default;
                }
            }
        }

        // Set flags
        for flag in &mut tab.flags {
            flag.enabled = entry.flags.contains(&flag.name);
        }

        // Switch to modules view
        self.switch_to_modules();

        // Execute the command
        let goal_parts: Vec<&str> = entry.goal.split_whitespace().collect();
        self.run_selected_module_command(&goal_parts);

        log::info!("History entry applied and command executed");
    }

    /// Get filtered history entries based on current filter
    pub fn get_filtered_history(&self) -> Vec<crate::features::history::HistoryEntry> {
        if self.history_filter.is_empty() {
            self.command_history.entries().to_vec()
        } else {
            let filter = self.history_filter.to_lowercase();
            self.command_history
                .entries()
                .iter()
                .filter(|entry| {
                    entry.module.to_lowercase().contains(&filter)
                        || entry.goal.to_lowercase().contains(&filter)
                        || entry.profiles.iter().any(|p| p.to_lowercase().contains(&filter))
                        || entry.flags.iter().any(|f| f.to_lowercase().contains(&filter))
                })
                .cloned()
                .collect()
        }
    }

    /// Push character to history filter
    pub fn push_history_filter_char(&mut self, ch: char) {
        self.history_filter.push(ch);
        // Reset selection when filter changes
        if !self.get_filtered_history().is_empty() {
            self.history_list_state.select(Some(0));
        }
    }

    /// Pop character from history filter
    pub fn pop_history_filter_char(&mut self) {
        self.history_filter.pop();
        // Reset selection when filter changes
        if !self.get_filtered_history().is_empty() {
            self.history_list_state.select(Some(0));
        }
    }
}
