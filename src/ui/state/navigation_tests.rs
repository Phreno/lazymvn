//! Tests for navigation and focus management

#[cfg(test)]
mod tests {
    use super::super::TuiState;
    use crate::core::config::Config;
    use crate::ui::keybindings::{CurrentView, Focus};
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

#[cfg(test)]
mod helper_tests {
    use super::super::navigation::{
        calculate_next_index, calculate_previous_index, is_navigation_debounced,
    };
    use std::time::Instant;

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
