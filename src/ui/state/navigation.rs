//! Navigation and focus management
//!
//! This module handles navigation between items in lists (modules, profiles, flags)
//! and focus management between different UI panes.

use super::{TuiState, Focus, CurrentView};
use std::time::Instant;

impl TuiState {
    /// Check if navigation is allowed (with debouncing)
    pub(super) fn should_allow_navigation(&mut self) -> bool {
        let now = Instant::now();

        if let Some(last_time) = self.last_nav_key_time
            && now.duration_since(last_time) < self.nav_debounce_duration
        {
            log::debug!("Navigation debounced (too fast)");
            return false;
        }

        self.last_nav_key_time = Some(now);
        true
    }

    /// Navigate to the next item in the current focused list
    pub fn next_item(&mut self) {
        if !self.should_allow_navigation() {
            return;
        }

        let focus = self.focus;
        let tab = self.get_active_tab_mut();

        match focus {
            Focus::Projects => {
                // Projects view is static, no navigation needed
            }
            Focus::Modules => {
                let modules_empty = tab.modules.is_empty();
                if modules_empty {
                    return;
                }

                // Need to drop tab borrow to call save_module_preferences
                let new_index = {
                    match tab.modules_list_state.selected() {
                        Some(i) => (i + 1) % tab.modules.len(),
                        None => 0,
                    }
                };

                // Save current module preferences before switching
                self.save_module_preferences();

                // Now update the selection
                let tab = self.get_active_tab_mut();
                tab.modules_list_state.select(Some(new_index));

                self.sync_selected_module_output();

                // Load preferences for the new module
                self.load_module_preferences();
            }
            Focus::Profiles => {
                if !tab.profiles.is_empty() {
                    let i = match tab.profiles_list_state.selected() {
                        Some(i) => (i + 1) % tab.profiles.len(),
                        None => 0,
                    };
                    tab.profiles_list_state.select(Some(i));
                    // Update output to show new profile XML
                    self.sync_selected_profile_output();
                }
            }
            Focus::Flags => {
                if !tab.flags.is_empty() {
                    let i = match tab.flags_list_state.selected() {
                        Some(i) => (i + 1) % tab.flags.len(),
                        None => 0,
                    };
                    tab.flags_list_state.select(Some(i));
                }
            }
            Focus::Output => {
                // No item navigation in output
            }
        }
    }

    /// Navigate to the previous item in the current focused list
    pub fn previous_item(&mut self) {
        if !self.should_allow_navigation() {
            return;
        }

        let focus = self.focus;
        let tab = self.get_active_tab_mut();

        match focus {
            Focus::Projects => {
                // Projects view is static, no navigation needed
            }
            Focus::Modules => {
                let modules_empty = tab.modules.is_empty();
                if modules_empty {
                    return;
                }

                // Need to drop tab borrow to call save_module_preferences
                let new_index = {
                    match tab.modules_list_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                tab.modules.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    }
                };

                // Save current module preferences before switching
                self.save_module_preferences();

                // Now update the selection
                let tab = self.get_active_tab_mut();
                tab.modules_list_state.select(Some(new_index));

                self.sync_selected_module_output();

                // Load preferences for the new module
                self.load_module_preferences();
            }
            Focus::Profiles => {
                if !tab.profiles.is_empty() {
                    let i = match tab.profiles_list_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                tab.profiles.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    tab.profiles_list_state.select(Some(i));
                    // Update output to show new profile XML
                    self.sync_selected_profile_output();
                }
            }
            Focus::Flags => {
                if !tab.flags.is_empty() {
                    let i = match tab.flags_list_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                tab.flags.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    tab.flags_list_state.select(Some(i));
                }
            }
            Focus::Output => {
                // No item navigation in output
            }
        }
    }

    /// Get the currently selected module name
    pub fn selected_module(&self) -> Option<&str> {
        let tab = self.get_active_tab();
        tab.modules_list_state
            .selected()
            .and_then(|i| tab.modules.get(i))
            .map(|s| s.as_str())
    }

    /// Switch to the projects view
    pub fn switch_to_projects(&mut self) {
        self.current_view = CurrentView::Projects;
        self.focus = Focus::Projects;
    }

    /// Switch to the modules view
    pub fn switch_to_modules(&mut self) {
        self.current_view = CurrentView::Modules;
        self.focus = Focus::Modules;
        self.sync_selected_module_output();
    }

    /// Switch to the profiles view
    pub fn switch_to_profiles(&mut self) {
        self.current_view = CurrentView::Profiles;
        let tab = self.get_active_tab_mut();
        if tab.profiles_list_state.selected().is_none() && !tab.profiles.is_empty() {
            tab.profiles_list_state.select(Some(0));
        }
        self.focus = Focus::Profiles;
        // Sync profile XML to output
        self.sync_selected_profile_output();
    }

    /// Switch to the flags view
    pub fn switch_to_flags(&mut self) {
        self.current_view = CurrentView::Flags;
        let tab = self.get_active_tab_mut();
        if tab.flags_list_state.selected().is_none() && !tab.flags.is_empty() {
            tab.flags_list_state.select(Some(0));
        }
        self.focus = Focus::Flags;
    }

    /// Set focus to the output pane
    pub fn focus_output(&mut self) {
        self.focus = Focus::Output;
        self.ensure_current_match_visible();
    }

    /// Cycle focus to the next pane (right arrow)
    pub fn cycle_focus_right(&mut self) {
        let old_focus = self.focus;
        self.focus = self.focus.next();

        // When leaving Profiles focus, restore module output
        if old_focus == Focus::Profiles && self.focus != Focus::Profiles {
            self.sync_selected_module_output();
        }
        // When entering Profiles focus, show profile XML
        else if self.focus == Focus::Profiles {
            self.sync_selected_profile_output();
        }

        if self.focus == Focus::Output {
            self.ensure_current_match_visible();
        }
    }

    /// Cycle focus to the previous pane (left arrow)
    pub fn cycle_focus_left(&mut self) {
        let old_focus = self.focus;
        self.focus = self.focus.previous();

        // When leaving Profiles focus, restore module output
        if old_focus == Focus::Profiles && self.focus != Focus::Profiles {
            self.sync_selected_module_output();
        }
        // When entering Profiles focus, show profile XML
        else if self.focus == Focus::Profiles {
            self.sync_selected_profile_output();
        }

        if self.focus == Focus::Output {
            self.ensure_current_match_visible();
        }
    }
}
