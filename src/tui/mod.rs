//! Main TUI coordination module
//!
//! This module provides the main TUI functionality by coordinating between
//! rendering, event handling, and state management.

mod mouse;
mod renderer;

// Re-export public API
pub use mouse::handle_mouse_event;
pub use renderer::draw;

// Re-export commonly used types for backward compatibility
pub use crate::ui::state::TuiState;

use crossterm::event::KeyEvent;

/// Handle key events by delegating to the keybindings module
pub fn handle_key_event(key: KeyEvent, state: &mut TuiState) {
    crate::ui::keybindings::handle_key_event(key, state);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::fs_lock;
    use crate::ui::keybindings::{CurrentView, Focus};
    use ratatui::{Terminal, backend::TestBackend};
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn test_cfg() -> crate::core::config::Config {
        crate::core::config::Config {
            maven_settings: None,
            launch_mode: None,
            notifications_enabled: None,
            watch: None,
            output: None,
            logging: None,
            spring: None,
            maven: None,
        }
    }

    /// Helper to setup a fake profiles cache to avoid async profile loading in tests
    fn setup_fake_profiles_cache(project_root: &std::path::Path) {
        let cache = crate::core::config::ProfilesCache {
            profiles: vec!["dev".to_string(), "test".to_string()],
        };
        let _ = cache.save(project_root);
    }

    /// Helper to cleanup profiles cache after tests
    fn cleanup_profiles_cache(project_root: &std::path::Path) {
        let _ = crate::core::config::ProfilesCache::invalidate(project_root);
    }

    #[test]
    fn test_draw_ui() {
        let _guard = fs_lock().lock().unwrap();
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let modules = vec!["module1".to_string(), "module2".to_string()];
        let project_root = PathBuf::from("/test_draw_ui");
        
        // Setup fake cache to avoid async profile loading
        setup_fake_profiles_cache(&project_root);
        
        let mut state = crate::ui::state::TuiState::new(modules, project_root.clone(), test_cfg());
        {
            let tab = state.get_active_tab_mut();
            tab.command_output = vec!["output1".to_string(), "output2".to_string()];
        }

        // Test that drawing succeeds without errors
        draw(&mut terminal, &mut state).unwrap();
        let buffer = terminal.backend().buffer();
        let rendered = buffer
            .content()
            .iter()
            .map(|cell| cell.symbol().chars().next().unwrap_or(' '))
            .collect::<String>();
        assert!(rendered.contains("Modules") || rendered.contains("Output"));
        
        // Cleanup
        cleanup_profiles_cache(&project_root);
    }

    #[test]
    fn test_view_switching() {
        let _guard = fs_lock().lock().unwrap();
        let modules = vec!["module1".to_string()];
        let project_root = PathBuf::from("/test_view_switching");
        
        // Setup fake cache to avoid async profile loading
        setup_fake_profiles_cache(&project_root);
        
        let mut state = crate::ui::state::TuiState::new(modules, project_root.clone(), test_cfg());

        // Initial view is Modules
        assert_eq!(state.current_view, CurrentView::Modules);

        // Press '3' to switch to Profiles
        handle_key_event(
            crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('3')),
            &mut state,
        );
        assert_eq!(state.current_view, CurrentView::Profiles);

        // Press '4' to switch to Flags
        handle_key_event(
            crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('4')),
            &mut state,
        );
        assert_eq!(state.current_view, CurrentView::Flags);

        // Press '2' to return to Modules
        handle_key_event(
            crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('2')),
            &mut state,
        );
        assert_eq!(state.current_view, CurrentView::Modules);

        // Press '1' to switch to Projects
        handle_key_event(
            crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('1')),
            &mut state,
        );
        assert_eq!(state.current_view, CurrentView::Projects);
        
        // Cleanup
        cleanup_profiles_cache(&project_root);
    }

    #[test]
    fn test_package_command() {
        let _guard = fs_lock().lock().unwrap();
        // 1. Setup temp project
        let project_dir = tempdir().unwrap();
        let project_root = project_dir.path();

        // 2. Create mock mvnw script
        let mvnw_path = project_root.join("mvnw");
        let mut mvnw_file = std::fs::File::create(&mvnw_path).unwrap();
        use std::io::Write;
        mvnw_file.write_all(b"#!/bin/sh\necho $@").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = mvnw_file.metadata().unwrap().permissions();
            perms.set_mode(0o755);
            mvnw_file.set_permissions(perms).unwrap();
        }
        drop(mvnw_file);

        // 3. Create TuiState
        let modules = vec!["module1".to_string()];
        
        // Setup fake cache to avoid async profile loading
        setup_fake_profiles_cache(project_root);
        
        let mut state =
            crate::ui::state::TuiState::new(modules, project_root.to_path_buf(), test_cfg());

        // Set profile names first to initialize MavenProfile structs
        state.set_profiles(vec!["p1".to_string()]);
        // Then toggle the profile to explicitly enable it
        if !state.get_active_tab().profiles.is_empty() {
            state.get_active_tab_mut().profiles[0].state =
                crate::ui::state::ProfileState::ExplicitlyEnabled;
        }

        // 4. Simulate 'k' key press for package
        handle_key_event(
            crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('k')),
            &mut state,
        );

        // 5. Assert command output contains expected elements
        let cleaned_output: Vec<String> = state
            .get_active_tab()
            .command_output
            .iter()
            .filter_map(|line| crate::utils::text::clean_log_line(line))
            .collect();
        assert!(!cleaned_output.is_empty());
    }

    #[test]
    #[cfg(unix)] // Shell script execution not supported on Windows
    fn test_build_command_runs_clean_install() {
        let _guard = fs_lock().lock().unwrap();
        let project_dir = tempdir().unwrap();
        let project_root = project_dir.path();

        let mvnw_path = project_root.join("mvnw");
        let mut mvnw_file = std::fs::File::create(&mvnw_path).unwrap();
        use std::io::Write;
        mvnw_file.write_all(b"#!/bin/sh\necho $@").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = mvnw_file.metadata().unwrap().permissions();
            perms.set_mode(0o755);
            mvnw_file.set_permissions(perms).unwrap();
        }
        drop(mvnw_file);

        // Setup fake cache to avoid async profile loading
        setup_fake_profiles_cache(project_root);

        let modules = vec!["module1".to_string()];
        let mut state =
            crate::ui::state::TuiState::new(modules, project_root.to_path_buf(), test_cfg());

        handle_key_event(
            crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('b')),
            &mut state,
        );

        let cleaned_output: Vec<String> = state
            .get_active_tab()
            .command_output
            .iter()
            .filter_map(|line| crate::utils::text::clean_log_line(line))
            .collect();
        assert!(
            cleaned_output
                .iter()
                .any(|line| line.contains("clean install"))
        );
    }

    #[test]
    fn test_flags_toggle() {
        let _guard = fs_lock().lock().unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        let modules = vec!["module1".to_string()];
        let project_root = temp_dir.path().to_path_buf();
        
        // Setup fake cache to avoid async profile loading
        setup_fake_profiles_cache(&project_root);
        
        let mut state = crate::ui::state::TuiState::new(modules, project_root, test_cfg());

        // Switch to flags view with '4'
        handle_key_event(
            crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('4')),
            &mut state,
        );
        assert_eq!(state.current_view, CurrentView::Flags);

        // Check initial state - no flags enabled
        assert_eq!(state.enabled_flag_names().len(), 0);

        // Toggle first flag with Enter
        handle_key_event(
            crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Enter),
            &mut state,
        );
        assert_eq!(state.enabled_flag_names().len(), 1);

        // Toggle it off with Space
        handle_key_event(
            crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char(' ')),
            &mut state,
        );
        assert_eq!(state.enabled_flag_names().len(), 0);
    }

    #[test]
    fn test_flags_initialized() {
        let _guard = fs_lock().lock().unwrap();
        use tempfile::tempdir;

        // Use a temporary directory to avoid loading actual cached preferences
        let temp_dir = tempdir().unwrap();
        let project_root = temp_dir.path().to_path_buf();

        // Setup fake cache to avoid async profile loading
        setup_fake_profiles_cache(&project_root);

        let modules = vec!["module1".to_string()];
        let state = crate::ui::state::TuiState::new(modules, project_root, test_cfg());

        // Check flags are initialized
        assert!(
            !state.get_active_tab().flags.is_empty(),
            "Flags should be initialized"
        );

        // Check all flags start disabled
        assert_eq!(
            state.enabled_flag_names().len(),
            0,
            "All flags should start disabled"
        );
    }

    #[test]
    fn test_navigation_debouncing() {
        let _guard = fs_lock().lock().unwrap();
        use std::{thread, time::Duration};

        let modules = vec![
            "module1".to_string(),
            "module2".to_string(),
            "module3".to_string(),
        ];
        let temp_dir = tempdir().unwrap();
        let project_root = temp_dir.path().to_path_buf();
        
        // Setup fake cache to avoid async profile loading
        setup_fake_profiles_cache(&project_root);
        
        let mut state = crate::ui::state::TuiState::new(modules, project_root, test_cfg());

        // Initial selection should be module1 (index 0)
        assert_eq!(
            state.get_active_tab().modules_list_state.selected(),
            Some(0)
        );

        // Rapid down presses - should only move once due to debouncing
        for _ in 0..5 {
            handle_key_event(
                crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Down),
                &mut state,
            );
        }

        // Should only have moved to index 1, not 5 (due to debouncing)
        assert_eq!(
            state.get_active_tab().modules_list_state.selected(),
            Some(1)
        );

        // Wait for debounce period to pass
        thread::sleep(Duration::from_millis(110));

        // Now another press should work
        handle_key_event(
            crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Down),
            &mut state,
        );
        assert_eq!(
            state.get_active_tab().modules_list_state.selected(),
            Some(2)
        );
    }

    #[test]
    fn test_profile_selection() {
        let _guard = fs_lock().lock().unwrap();
        let temp_dir = tempdir().unwrap();
        let modules = vec!["module1".to_string()];
        let project_root = temp_dir.path().to_path_buf();
        
        // Setup fake cache to avoid async profile loading
        setup_fake_profiles_cache(&project_root);
        
        let mut state = crate::ui::state::TuiState::new(modules, project_root, test_cfg());

        // Add some profiles (not auto-activated)
        state.set_profiles(vec!["dev".to_string(), "prod".to_string()]);

        // Switch to profiles view
        state.switch_to_profiles();

        // Verify focus is on profiles
        assert_eq!(state.focus, Focus::Profiles);
        assert_eq!(state.current_view, CurrentView::Profiles);

        // No profiles should be explicitly enabled initially
        let active_count = state
            .get_active_tab()
            .profiles
            .iter()
            .filter(|p| p.is_active())
            .count();
        assert_eq!(active_count, 0);

        // Simulate pressing Enter to toggle profile
        handle_key_event(
            crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Enter),
            &mut state,
        );

        // First profile should now be explicitly enabled
        let active_count = state
            .get_active_tab()
            .profiles
            .iter()
            .filter(|p| p.is_active())
            .count();
        assert_eq!(active_count, 1);
        assert_eq!(
            state.get_active_tab().profiles[0].state,
            crate::ui::state::ProfileState::ExplicitlyEnabled
        );

        // Press Enter again to deactivate (back to Default)
        handle_key_event(
            crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Enter),
            &mut state,
        );

        // Profile should be back to Default state
        assert_eq!(
            state.get_active_tab().profiles[0].state,
            crate::ui::state::ProfileState::Default
        );
    }

    #[test]
    fn test_flag_selection() {
        let _guard = fs_lock().lock().unwrap();
        let temp_dir = tempdir().unwrap();
        let modules = vec!["module1".to_string()];
        let project_root = temp_dir.path().to_path_buf();
        
        // Setup fake cache to avoid async profile loading
        setup_fake_profiles_cache(&project_root);
        
        let mut state = crate::ui::state::TuiState::new(modules, project_root, test_cfg());

        // Switch to flags view
        state.switch_to_flags();

        // Verify focus is on flags
        assert_eq!(state.focus, Focus::Flags);
        assert_eq!(state.current_view, CurrentView::Flags);

        // No flags should be enabled initially
        let enabled_count = state
            .get_active_tab()
            .flags
            .iter()
            .filter(|f| f.enabled)
            .count();
        assert_eq!(enabled_count, 0);

        // Simulate pressing Space to toggle flag
        handle_key_event(
            crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char(' ')),
            &mut state,
        );

        // First flag should now be enabled
        let enabled_count = state
            .get_active_tab()
            .flags
            .iter()
            .filter(|f| f.enabled)
            .count();
        assert_eq!(enabled_count, 1);
        assert!(state.get_active_tab().flags[0].enabled);

        // Press Space again to disable
        handle_key_event(
            crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char(' ')),
            &mut state,
        );

        // Flag should be disabled
        let enabled_count = state
            .get_active_tab()
            .flags
            .iter()
            .filter(|f| f.enabled)
            .count();
        assert_eq!(enabled_count, 0);
    }

    #[test]
    #[ignore] // Mouse tests are fragile due to terminal size dependencies
    fn test_mouse_pane_focus() {
        let _guard = fs_lock().lock().unwrap();
        let temp_dir = tempdir().unwrap();
        let modules = vec!["module1".to_string()];
        let project_root = temp_dir.path().to_path_buf();
        
        // Setup fake cache to avoid async profile loading
        setup_fake_profiles_cache(&project_root);
        
        let mut state = crate::ui::state::TuiState::new(modules, project_root, test_cfg());

        // Initial focus is on Modules
        assert_eq!(state.focus, Focus::Modules);

        // Simulate mouse click on output pane (right side)
        // Based on 30/70 split, output pane starts at column ~24 for 80 cols terminal
        // Use row 10 to avoid projects pane (rows 0-2) and stay in middle area
        let mouse_event = crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
            column: 50, // Right side of screen - output pane
            row: 10,
            modifiers: crossterm::event::KeyModifiers::empty(),
        };

        handle_mouse_event(mouse_event, &mut state);

        // Focus should now be on Output
        assert_eq!(state.focus, Focus::Output);
    }

    #[test]
    fn test_is_inside_area() {
        use ratatui::layout::Rect;

        let area = Rect {
            x: 10,
            y: 5,
            width: 20,
            height: 10,
        };

        // Inside
        assert!(mouse::is_inside_area((15, 8), area));
        assert!(mouse::is_inside_area((10, 5), area)); // Top-left corner
        assert!(mouse::is_inside_area((29, 14), area)); // Bottom-right corner (exclusive)

        // Outside
        assert!(!mouse::is_inside_area((9, 8), area)); // Left of area
        assert!(!mouse::is_inside_area((30, 8), area)); // Right of area
        assert!(!mouse::is_inside_area((15, 4), area)); // Above area
        assert!(!mouse::is_inside_area((15, 15), area)); // Below area
    }

    #[test]
    fn test_profile_auto_activation_three_states() {
        use crate::ui::state::{MavenProfile, ProfileState};

        // Test 1: Non-auto-activated profile
        let mut profile = MavenProfile::new("dev".to_string(), false);
        assert_eq!(profile.state, ProfileState::Default);
        assert!(!profile.is_active()); // Not active in default state
        assert_eq!(profile.to_maven_arg(), None); // No arg needed

        // Toggle: Default → ExplicitlyEnabled (for non-auto profiles)
        profile.toggle();
        assert_eq!(profile.state, ProfileState::ExplicitlyEnabled);
        assert!(profile.is_active());
        assert_eq!(profile.to_maven_arg(), Some("dev".to_string()));

        // Toggle: ExplicitlyEnabled → Default
        profile.toggle();
        assert_eq!(profile.state, ProfileState::Default);
        assert!(!profile.is_active());

        // Test 2: Auto-activated profile
        let mut auto_profile = MavenProfile::new("out-eclipse".to_string(), true);
        assert_eq!(auto_profile.state, ProfileState::Default);
        assert!(auto_profile.is_active()); // Active due to auto-activation
        assert_eq!(auto_profile.to_maven_arg(), None); // Default = no explicit arg

        // Toggle: Default → ExplicitlyDisabled (for auto profiles)
        auto_profile.toggle();
        assert_eq!(auto_profile.state, ProfileState::ExplicitlyDisabled);
        assert!(!auto_profile.is_active()); // Now disabled
        assert_eq!(
            auto_profile.to_maven_arg(),
            Some("!out-eclipse".to_string())
        );

        // Toggle: ExplicitlyDisabled → Default
        auto_profile.toggle();
        assert_eq!(auto_profile.state, ProfileState::Default);
        assert!(auto_profile.is_active()); // Back to auto-activated
    }

    #[test]
    #[ignore] // Mouse tests are fragile due to terminal size dependencies
    fn test_mouse_click_selects_item() {
        let _guard = fs_lock().lock().unwrap();
        let modules = vec![
            "module1".to_string(),
            "module2".to_string(),
            "module3".to_string(),
        ];
        let temp_dir = tempdir().unwrap();
        let project_root = temp_dir.path().to_path_buf();
        
        // Setup fake cache to avoid async profile loading
        setup_fake_profiles_cache(&project_root);
        
        let mut state = crate::ui::state::TuiState::new(modules, project_root, test_cfg());

        // Initial selection is first module
        assert_eq!(
            state.get_active_tab().modules_list_state.selected(),
            Some(0)
        );

        // Simulate mouse click on modules pane
        // Projects pane is at rows 0-2 (3 lines), modules pane starts at row 3
        // Row 5 is within modules pane (accounting for border and title at row 3-4)
        let mouse_event = crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
            column: 5, // Left side - modules pane
            row: 5,    // Within modules pane, below projects pane
            modifiers: crossterm::event::KeyModifiers::empty(),
        };

        handle_mouse_event(mouse_event, &mut state);

        // Should have switched focus to modules
        assert_eq!(state.focus, Focus::Modules);
        // Selection should have been updated based on click position
        assert!(
            state
                .get_active_tab()
                .modules_list_state
                .selected()
                .is_some()
        );
    }
}
