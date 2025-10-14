//! Main TUI drawing and coordination module
//!
//! This module provides the main drawing function that coordinates between
//! all the UI modules (panes, state, keybindings, etc.) to render the complete
//! terminal user interface.

use crate::ui::{
    keybindings,
    panes::{
        create_layout, render_flags_pane, render_footer, render_modules_pane, render_output_pane,
        render_profiles_pane,
    },
};
use crossterm::event::KeyEvent;
use ratatui::{Terminal, backend::Backend};

/// Re-export commonly used types for backward compatibility
pub use crate::ui::keybindings::{CurrentView, Focus};
pub use crate::ui::state::TuiState;

/// Main drawing function that renders the complete TUI
pub fn draw<B: Backend>(
    terminal: &mut Terminal<B>,
    state: &mut crate::ui::state::TuiState,
) -> Result<(), std::io::Error> {
    terminal.draw(|f| {
        let (left_area, right_area, footer_area) = create_layout(f.area());

        // Render left pane based on current view
        match state.current_view {
            CurrentView::Modules => {
                render_modules_pane(
                    f,
                    left_area,
                    &state.modules,
                    &mut state.modules_list_state,
                    state.focus == Focus::Modules,
                );
            }
            CurrentView::Profiles => {
                render_profiles_pane(
                    f,
                    left_area,
                    &state.profiles,
                    &state.active_profiles,
                    &mut state.profiles_list_state,
                    state.focus == Focus::Modules,
                );
            }
            CurrentView::Flags => {
                render_flags_pane(
                    f,
                    left_area,
                    &state.flags,
                    &mut state.flags_list_state,
                    state.focus == Focus::Modules,
                );
            }
        }

        // Update output metrics for proper scrolling calculations
        let inner_area = ratatui::widgets::Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .inner(right_area);
        state.update_output_metrics(inner_area.width);
        state.set_output_view_dimensions(inner_area.height, inner_area.width);

        // Render output pane
        render_output_pane(
            f,
            right_area,
            &state.command_output,
            state.output_offset,
            state.focus == Focus::Output,
            |line_index| state.search_line_style(line_index),
            state.search_mod.is_some(),
            state.selected_module(),
            state.current_output_context(),
        );

        // Render footer
        render_footer(
            f,
            footer_area,
            state.current_view,
            state.focus,
            state.menu_state(),
            state.selected_module(),
            &state.active_profiles,
            &state.enabled_flag_names(),
            state.search_status_line(),
        );
    })?;
    Ok(())
}

/// Handle key events by delegating to the keybindings module
pub fn handle_key_event(key: KeyEvent, state: &mut crate::ui::state::TuiState) {
    keybindings::handle_key_event(key, state);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn test_cfg() -> crate::config::Config {
        crate::config::Config {
            maven_settings: None,
        }
    }

    #[test]
    fn test_draw_ui() {
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let modules = vec!["module1".to_string(), "module2".to_string()];
        let project_root = PathBuf::from("/");
        let mut state = crate::ui::state::TuiState::new(modules, project_root, test_cfg());
        state.command_output = vec!["output1".to_string(), "output2".to_string()];

        // Test that drawing succeeds without errors
        draw(&mut terminal, &mut state).unwrap();
        let buffer = terminal.backend().buffer();
        let rendered = buffer
            .content()
            .iter()
            .map(|cell| cell.symbol().chars().next().unwrap_or(' '))
            .collect::<String>();
        assert!(rendered.contains("Modules") || rendered.contains("Output"));
    }

    #[test]
    fn test_view_switching() {
        let modules = vec!["module1".to_string()];
        let project_root = PathBuf::from("/");
        let mut state = crate::ui::state::TuiState::new(modules, project_root, test_cfg());

        // Initial view is Modules
        assert_eq!(state.current_view, CurrentView::Modules);

        // Press 'p' to switch to Profiles
        handle_key_event(
            crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('p')),
            &mut state,
        );
        assert_eq!(state.current_view, CurrentView::Profiles);

        // Press 'f' to switch to Flags
        handle_key_event(
            crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('f')),
            &mut state,
        );
        assert_eq!(state.current_view, CurrentView::Flags);

        // Press 'm' to return to Modules
        handle_key_event(
            crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('m')),
            &mut state,
        );
        assert_eq!(state.current_view, CurrentView::Modules);
    }

    #[test]
    fn test_package_command() {
        // 1. Setup temp project
        let project_dir = tempdir().unwrap();
        let project_root = project_dir.path();

        // 2. Create mock mvnw script
        let mvnw_path = project_root.join("mvnw");
        let mut mvnw_file = std::fs::File::create(&mvnw_path).unwrap();
        use std::io::Write;
        mvnw_file.write_all(b"#!/bin/sh\necho $@").unwrap();
        use std::os::unix::fs::PermissionsExt;
        let mut perms = mvnw_file.metadata().unwrap().permissions();
        perms.set_mode(0o755);
        mvnw_file.set_permissions(perms).unwrap();
        drop(mvnw_file);

        // 3. Create TuiState
        let modules = vec!["module1".to_string()];
        let mut state =
            crate::ui::state::TuiState::new(modules, project_root.to_path_buf(), test_cfg());
        state.active_profiles = vec!["p1".to_string()];

        // 4. Simulate 'k' key press for package
        handle_key_event(
            crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('k')),
            &mut state,
        );

        // 5. Assert command output contains expected elements
        let cleaned_output: Vec<String> = state
            .command_output
            .iter()
            .filter_map(|line| crate::utils::clean_log_line(line))
            .collect();
        assert!(!cleaned_output.is_empty());
    }

    #[test]
    fn test_build_command_runs_clean_install() {
        let project_dir = tempdir().unwrap();
        let project_root = project_dir.path();

        let mvnw_path = project_root.join("mvnw");
        let mut mvnw_file = std::fs::File::create(&mvnw_path).unwrap();
        use std::io::Write;
        mvnw_file.write_all(b"#!/bin/sh\necho $@").unwrap();
        use std::os::unix::fs::PermissionsExt;
        let mut perms = mvnw_file.metadata().unwrap().permissions();
        perms.set_mode(0o755);
        mvnw_file.set_permissions(perms).unwrap();
        drop(mvnw_file);

        let modules = vec!["module1".to_string()];
        let mut state =
            crate::ui::state::TuiState::new(modules, project_root.to_path_buf(), test_cfg());

        handle_key_event(
            crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('b')),
            &mut state,
        );

        let cleaned_output: Vec<String> = state
            .command_output
            .iter()
            .filter_map(|line| crate::utils::clean_log_line(line))
            .collect();
        assert!(
            cleaned_output
                .iter()
                .any(|line| line.contains("clean install"))
        );
    }

    #[test]
    fn test_flags_toggle() {
        let modules = vec!["module1".to_string()];
        let project_root = PathBuf::from("/");
        let mut state = crate::ui::state::TuiState::new(modules, project_root, test_cfg());

        // Switch to flags view
        handle_key_event(
            crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('f')),
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
    #[test]
    fn test_flags_initialized() {
        let modules = vec!["module1".to_string()];
        let project_root = PathBuf::from("/");
        let state = crate::ui::state::TuiState::new(modules, project_root, test_cfg());

        // Check flags are initialized
        assert!(state.flags.len() > 0, "Flags should be initialized");

        // Check all flags start disabled
        assert_eq!(
            state.enabled_flag_names().len(),
            0,
            "All flags should start disabled"
        );
    }
}
