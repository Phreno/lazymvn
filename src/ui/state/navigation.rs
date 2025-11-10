//! Navigation and focus management
//!
//! This module handles navigation between items in lists (modules, profiles, flags)
//! and focus management between different UI panes.

use super::{CurrentView, Focus, TuiState};
use std::time::Instant;

impl TuiState {
    /// Check if navigation is allowed (with debouncing)
    pub(super) fn should_allow_navigation(&mut self) -> bool {
        let now = Instant::now();
        
        if is_navigation_debounced(self.last_nav_key_time, now, self.nav_debounce_duration) {
            log::trace!("Navigation debounced (too fast)");
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

                let new_index = calculate_next_index(
                    tab.modules_list_state.selected(),
                    tab.modules.len()
                );

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
                    let i = calculate_next_index(
                        tab.profiles_list_state.selected(),
                        tab.profiles.len()
                    );
                    tab.profiles_list_state.select(Some(i));
                    // Update output to show new profile XML
                    self.sync_selected_profile_output();
                }
            }
            Focus::Flags => {
                if !tab.flags.is_empty() {
                    let i = calculate_next_index(
                        tab.flags_list_state.selected(),
                        tab.flags.len()
                    );
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

                let new_index = calculate_previous_index(
                    tab.modules_list_state.selected(),
                    tab.modules.len()
                );

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
                    let i = calculate_previous_index(
                        tab.profiles_list_state.selected(),
                        tab.profiles.len()
                    );
                    tab.profiles_list_state.select(Some(i));
                    // Update output to show new profile XML
                    self.sync_selected_profile_output();
                }
            }
            Focus::Flags => {
                if !tab.flags.is_empty() {
                    let i = calculate_previous_index(
                        tab.flags_list_state.selected(),
                        tab.flags.len()
                    );
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;
    use std::path::PathBuf;

    fn create_test_state() -> TuiState {
        TuiState::new(
            vec![
                "module1".to_string(),
                "module2".to_string(),
                "module3".to_string(),
            ],
            PathBuf::from("/test"),
            Config::default(),
        )
    }

    #[test]
    fn test_selected_module_none() {
        let state = create_test_state();
        assert!(state.selected_module().is_some());
    }

    #[test]
    fn test_selected_module_first() {
        let state = create_test_state();
        assert_eq!(state.selected_module(), Some("module1"));
    }

    #[test]
    fn test_next_item_modules() {
        let mut state = create_test_state();
        state.focus = Focus::Modules;

        assert_eq!(state.selected_module(), Some("module1"));

        state.next_item();

        assert_eq!(state.selected_module(), Some("module2"));
    }

    #[test]
    fn test_next_item_modules_wraps() {
        let mut state = create_test_state();
        state.focus = Focus::Modules;

        // Go to last module
        {
            let tab = state.get_active_tab_mut();
            tab.modules_list_state.select(Some(2));
        }

        state.next_item();

        // Should wrap to first
        assert_eq!(state.selected_module(), Some("module1"));
    }

    #[test]
    fn test_previous_item_modules() {
        let mut state = create_test_state();
        state.focus = Focus::Modules;

        // Start at second module
        {
            let tab = state.get_active_tab_mut();
            tab.modules_list_state.select(Some(1));
        }

        state.previous_item();

        assert_eq!(state.selected_module(), Some("module1"));
    }

    #[test]
    fn test_previous_item_modules_wraps() {
        let mut state = create_test_state();
        state.focus = Focus::Modules;

        assert_eq!(state.selected_module(), Some("module1"));

        state.previous_item();

        // Should wrap to last
        assert_eq!(state.selected_module(), Some("module3"));
    }

    #[test]
    fn test_switch_to_projects() {
        let mut state = create_test_state();

        state.switch_to_projects();

        assert_eq!(state.current_view, CurrentView::Projects);
        assert_eq!(state.focus, Focus::Projects);
    }

    #[test]
    fn test_switch_to_modules() {
        let mut state = create_test_state();
        state.current_view = CurrentView::Projects;

        state.switch_to_modules();

        assert_eq!(state.current_view, CurrentView::Modules);
        assert_eq!(state.focus, Focus::Modules);
    }

    #[test]
    fn test_switch_to_profiles() {
        let mut state = create_test_state();

        state.switch_to_profiles();

        assert_eq!(state.current_view, CurrentView::Profiles);
        assert_eq!(state.focus, Focus::Profiles);
    }

    #[test]
    fn test_switch_to_flags() {
        let mut state = create_test_state();

        state.switch_to_flags();

        assert_eq!(state.current_view, CurrentView::Flags);
        assert_eq!(state.focus, Focus::Flags);
    }

    #[test]
    fn test_focus_output() {
        let mut state = create_test_state();
        state.focus = Focus::Modules;

        state.focus_output();

        assert_eq!(state.focus, Focus::Output);
    }

    #[test]
    fn test_cycle_focus_right() {
        let mut state = create_test_state();
        state.focus = Focus::Modules;

        state.cycle_focus_right();

        assert_eq!(state.focus, Focus::Profiles);
    }

    #[test]
    fn test_cycle_focus_left() {
        let mut state = create_test_state();
        state.focus = Focus::Profiles;

        state.cycle_focus_left();

        assert_eq!(state.focus, Focus::Modules);
    }

    #[test]
    fn test_cycle_focus_right_from_output() {
        let mut state = create_test_state();
        state.focus = Focus::Output;

        state.cycle_focus_right();

        // Should cycle back to Projects (not Modules)
        assert_eq!(state.focus, Focus::Projects);
    }

    #[test]
    fn test_cycle_focus_left_from_modules() {
        let mut state = create_test_state();
        state.focus = Focus::Modules;

        state.cycle_focus_left();

        // Should wrap to Projects (not Output)
        assert_eq!(state.focus, Focus::Projects);
    }

    #[test]
    fn test_next_item_empty_modules() {
        let mut state = TuiState::new(vec![], PathBuf::from("/test"), Config::default());
        state.focus = Focus::Modules;

        state.next_item();

        // Should handle empty list gracefully
        assert!(state.selected_module().is_none());
    }

    #[test]
    fn test_next_item_profiles() {
        let mut state = create_test_state();
        state.focus = Focus::Profiles;

        // Add some profiles
        {
            let tab = state.get_active_tab_mut();
            tab.profiles = vec![
                super::super::MavenProfile {
                    name: "dev".to_string(),
                    state: super::super::ProfileState::Default,
                    auto_activated: false,
                },
                super::super::MavenProfile {
                    name: "prod".to_string(),
                    state: super::super::ProfileState::Default,
                    auto_activated: false,
                },
            ];
            tab.profiles_list_state.select(Some(0));
        }

        state.next_item();

        assert_eq!(
            state.get_active_tab().profiles_list_state.selected(),
            Some(1)
        );
    }

    #[test]
    fn test_next_item_flags() {
        let mut state = create_test_state();
        state.focus = Focus::Flags;

        // Flags should already be populated
        let flags_count = state.get_active_tab().flags.len();
        if flags_count > 1 {
            {
                let tab = state.get_active_tab_mut();
                tab.flags_list_state.select(Some(0));
            }

            state.next_item();

            assert_eq!(state.get_active_tab().flags_list_state.selected(), Some(1));
        }
    }
}

/// Check if navigation should be debounced
fn is_navigation_debounced(
    last_time: Option<Instant>,
    now: Instant,
    debounce_duration: std::time::Duration
) -> bool {
    if let Some(last) = last_time {
        now.duration_since(last) < debounce_duration
    } else {
        false
    }
}

/// Calculate next index in a circular list
fn calculate_next_index(current: Option<usize>, list_len: usize) -> usize {
    match current {
        Some(i) => (i + 1) % list_len,
        None => 0,
    }
}

/// Calculate previous index in a circular list
fn calculate_previous_index(current: Option<usize>, list_len: usize) -> usize {
    match current {
        Some(i) => {
            if i == 0 {
                list_len - 1
            } else {
                i - 1
            }
        }
        None => 0,
    }
}

#[cfg(test)]
mod helper_tests {
    use super::*;

    #[test]
    fn test_is_navigation_debounced_none() {
        let now = Instant::now();
        let debounce = std::time::Duration::from_millis(100);
        assert!(!is_navigation_debounced(None, now, debounce));
    }

    #[test]
    fn test_is_navigation_debounced_too_fast() {
        let last = Instant::now();
        let now = last + std::time::Duration::from_millis(50);
        let debounce = std::time::Duration::from_millis(100);
        assert!(is_navigation_debounced(Some(last), now, debounce));
    }

    #[test]
    fn test_is_navigation_debounced_allowed() {
        let last = Instant::now();
        let now = last + std::time::Duration::from_millis(150);
        let debounce = std::time::Duration::from_millis(100);
        assert!(!is_navigation_debounced(Some(last), now, debounce));
    }

    #[test]
    fn test_calculate_next_index_none() {
        assert_eq!(calculate_next_index(None, 5), 0);
    }

    #[test]
    fn test_calculate_next_index_middle() {
        assert_eq!(calculate_next_index(Some(2), 5), 3);
    }

    #[test]
    fn test_calculate_next_index_wraps() {
        assert_eq!(calculate_next_index(Some(4), 5), 0);
    }

    #[test]
    fn test_calculate_previous_index_none() {
        assert_eq!(calculate_previous_index(None, 5), 0);
    }

    #[test]
    fn test_calculate_previous_index_middle() {
        assert_eq!(calculate_previous_index(Some(2), 5), 1);
    }

    #[test]
    fn test_calculate_previous_index_wraps() {
        assert_eq!(calculate_previous_index(Some(0), 5), 4);
    }

    #[test]
    fn test_calculate_next_index_single_item() {
        assert_eq!(calculate_next_index(Some(0), 1), 0);
    }

    #[test]
    fn test_calculate_previous_index_single_item() {
        assert_eq!(calculate_previous_index(Some(0), 1), 0);
    }
}
