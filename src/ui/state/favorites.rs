//! Favorites management for TUI

use super::TuiState;

impl TuiState {
    /// Show save favorite dialog with current context
    pub fn show_save_favorite_dialog_from_current(&mut self) {
        if let Some(module) = self.selected_module() {
            let tab = self.get_active_tab();

            // Get active profiles
            let active_profiles: Vec<String> = tab
                .profiles
                .iter()
                .filter(|p| p.is_active())
                .map(|p| p.name.clone())
                .collect();

            // Get enabled flags
            let enabled_flags: Vec<String> = tab
                .flags
                .iter()
                .filter(|f| f.enabled)
                .map(|f| f.name.clone())
                .collect();

            // Create a pending favorite entry
            let entry = crate::features::history::HistoryEntry::new(
                self.get_active_tab().project_root.clone(),
                module.to_string(),
                "".to_string(), // Will be filled with goal when saving
                active_profiles,
                enabled_flags,
            );

            self.pending_favorite = Some(entry);
            self.favorite_name_input.clear();
            self.show_save_favorite_popup = true;
            log::info!("Opened save favorite dialog");
        }
    }

    /// Save the pending favorite with the entered name
    pub fn save_pending_favorite(&mut self, goal: String) {
        if let Some(mut entry) = self.pending_favorite.take() {
            entry.goal = goal;

            let favorite = crate::features::favorites::Favorite::new(
                self.favorite_name_input.clone(),
                entry.module,
                entry.goal,
                entry.profiles,
                entry.flags,
            );

            self.favorites.add(favorite);
            self.show_save_favorite_popup = false;
            self.favorite_name_input.clear();
            log::info!("Favorite saved successfully");
        }
    }

    /// Cancel saving favorite
    pub fn cancel_save_favorite(&mut self) {
        self.show_save_favorite_popup = false;
        self.favorite_name_input.clear();
        self.pending_favorite = None;
        log::info!("Canceled save favorite");
    }

    /// Apply a favorite: select module, set profiles, flags, and show in modules view
    pub fn apply_favorite(&mut self, favorite: &crate::features::favorites::Favorite) {
        log::info!("Applying favorite: {}", favorite.name);

        let tab = self.get_active_tab_mut();

        // Find and select the module
        if let Some(module_idx) = tab.modules.iter().position(|m| m == &favorite.module) {
            tab.modules_list_state.select(Some(module_idx));
            log::debug!("Selected module at index {}", module_idx);
        } else {
            log::warn!("Module '{}' not found in current project", favorite.module);
            tab.command_output = vec![format!(
                "Error: Module '{}' not found in current project",
                favorite.module
            )];
            return;
        }

        // Set profiles
        for profile in &mut tab.profiles {
            if favorite.profiles.contains(&profile.name) {
                if !profile.is_active() {
                    profile.state = crate::ui::state::ProfileState::ExplicitlyEnabled;
                }
            } else if profile.auto_activated {
                profile.state = crate::ui::state::ProfileState::ExplicitlyDisabled;
            } else {
                profile.state = crate::ui::state::ProfileState::Default;
            }
        }

        // Set flags
        for flag in &mut tab.flags {
            flag.enabled = favorite.flags.contains(&flag.name);
        }

        // Switch to modules view
        self.switch_to_modules();

        // Execute the command
        let goal_parts: Vec<&str> = favorite.goal.split_whitespace().collect();
        self.run_selected_module_command(&goal_parts);

        log::info!("Favorite applied and command executed");
    }

    /// Get filtered favorites based on current filter
    pub fn get_filtered_favorites(&self) -> Vec<crate::features::favorites::Favorite> {
        if self.favorites_filter.is_empty() {
            self.favorites.list().to_vec()
        } else {
            let filter = self.favorites_filter.to_lowercase();
            self.favorites
                .list()
                .iter()
                .filter(|fav| {
                    fav.name.to_lowercase().contains(&filter)
                        || fav.module.to_lowercase().contains(&filter)
                        || fav.goal.to_lowercase().contains(&filter)
                        || fav.profiles.iter().any(|p| p.to_lowercase().contains(&filter))
                        || fav.flags.iter().any(|f| f.to_lowercase().contains(&filter))
                })
                .cloned()
                .collect()
        }
    }

    /// Push character to favorites filter
    pub fn push_favorites_filter_char(&mut self, ch: char) {
        self.favorites_filter.push(ch);
        // Reset selection when filter changes
        if !self.get_filtered_favorites().is_empty() {
            self.favorites_list_state.select(Some(0));
        }
    }

    /// Pop character from favorites filter
    pub fn pop_favorites_filter_char(&mut self) {
        self.favorites_filter.pop();
        // Reset selection when filter changes
        if !self.get_filtered_favorites().is_empty() {
            self.favorites_list_state.select(Some(0));
        }
    }

    /// Delete the selected favorite
    pub fn delete_selected_favorite(&mut self) {
        if let Some(selected) = self.favorites_list_state.selected() {
            let filtered_favorites = self.get_filtered_favorites();
            
            // Get the favorite from the filtered list
            if let Some(favorite_to_delete) = filtered_favorites.get(selected) {
                // Find its index in the complete list
                if let Some(actual_index) = self.favorites.list().iter().position(|f| {
                    f.name == favorite_to_delete.name
                        && f.module == favorite_to_delete.module
                        && f.goal == favorite_to_delete.goal
                }) {
                    if let Some(removed) = self.favorites.remove(actual_index) {
                        log::info!("Deleted favorite: {}", removed.name);

                        // Adjust selection in filtered list
                        let new_filtered_len = self.get_filtered_favorites().len();
                        if new_filtered_len == 0 {
                            self.favorites_list_state.select(None);
                        } else if selected >= new_filtered_len {
                            self.favorites_list_state.select(Some(new_filtered_len - 1));
                        }
                    }
                }
            }
        }
    }
}
