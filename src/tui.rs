//! Main TUI drawing and coordination module
//!
//! This module provides the main drawing function that coordinates between
//! all the UI modules (panes, state, keybindings, etc.) to render the complete
//! terminal user interface.

use crate::ui::{
    keybindings,
    panes::{
        create_adaptive_layout, render_flags_pane, render_footer, render_modules_pane,
        render_output_pane, render_profiles_pane, render_projects_pane,
        render_starter_manager_popup, render_starter_selector_popup,
    },
};
use crossterm::event::KeyEvent;
use ratatui::{Terminal, backend::Backend};

/// Re-export commonly used types for backward compatibility
pub use crate::ui::keybindings::Focus;
pub use crate::ui::state::TuiState;

/// Main drawing function that renders the complete TUI
pub fn draw<B: Backend>(
    terminal: &mut Terminal<B>,
    state: &mut crate::ui::state::TuiState,
) -> Result<(), std::io::Error> {
    terminal.draw(|f| {
        let (projects_area, modules_area, profiles_area, flags_area, output_area, footer_area) =
            create_adaptive_layout(f.area(), Some(state.focus));

        // Get project root name for display
        let project_name = state
            .project_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        // Render all left panes
        render_projects_pane(
            f,
            projects_area,
            project_name,
            state.focus == Focus::Projects,
        );

        render_modules_pane(
            f,
            modules_area,
            &state.modules,
            &mut state.modules_list_state,
            state.focus == Focus::Modules,
        );

        render_profiles_pane(
            f,
            profiles_area,
            &state.profiles,
            &state.active_profiles,
            &mut state.profiles_list_state,
            state.focus == Focus::Profiles,
        );

        render_flags_pane(
            f,
            flags_area,
            &state.flags,
            &mut state.flags_list_state,
            state.focus == Focus::Flags,
        );

        // Update output metrics for proper scrolling calculations
        let inner_area = ratatui::widgets::Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .inner(output_area);
        state.update_output_metrics(inner_area.width);
        state.set_output_view_dimensions(inner_area.height, inner_area.width);

        // Render output pane
        render_output_pane(
            f,
            output_area,
            &state.command_output,
            state.output_offset,
            state.focus == Focus::Output,
            |line_index| state.search_line_style(line_index),
            state.search_mod.is_some(),
            state.selected_module(),
            state.current_output_context(),
            state.is_command_running,
            state.command_elapsed_seconds(),
        );

        // Render footer
        render_footer(
            f,
            footer_area,
            state.current_view,
            state.focus,
            state.selected_module(),
            &state.active_profiles,
            &state.enabled_flag_names(),
            state.search_status_line(),
        );

        // Render starter selector popup if shown
        if state.show_starter_selector {
            let candidates = state.get_filtered_starter_candidates();
            render_starter_selector_popup(
                f,
                &candidates,
                &state.starter_filter,
                &mut state.starters_list_state,
            );
        }

        // Render starter manager popup if shown
        if state.show_starter_manager {
            render_starter_manager_popup(
                f,
                &state.starters_cache.starters,
                &mut state.starters_list_state,
            );
        }
    })?;
    Ok(())
}

/// Handle key events by delegating to the keybindings module
pub fn handle_key_event(key: KeyEvent, state: &mut crate::ui::state::TuiState) {
    keybindings::handle_key_event(key, state);
}

/// Handle mouse events for pane navigation
pub fn handle_mouse_event(
    mouse: crossterm::event::MouseEvent,
    state: &mut crate::ui::state::TuiState,
) {
    use crossterm::event::{MouseButton, MouseEventKind};

    // Only handle left button clicks
    if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
        return;
    }

    log::debug!("Mouse click at ({}, {})", mouse.column, mouse.row);

    // Get the current layout areas to determine which pane was clicked
    // We need to calculate this based on terminal size
    let terminal_size = match crossterm::terminal::size() {
        Ok((cols, rows)) => (cols, rows),
        Err(_) => return,
    };

    // Calculate layout areas using same logic as draw function
    let total_area = ratatui::layout::Rect {
        x: 0,
        y: 0,
        width: terminal_size.0,
        height: terminal_size.1,
    };

    let (projects_area, modules_area, profiles_area, flags_area, output_area, _footer_area) =
        create_adaptive_layout(total_area, Some(state.focus));

    // Check which pane was clicked and set focus accordingly
    let click_pos = (mouse.column, mouse.row);

    if is_inside_area(click_pos, projects_area) {
        log::info!("Mouse clicked on Projects pane");
        state.switch_to_projects();
        handle_pane_item_click(mouse, projects_area, state, Focus::Projects);
    } else if is_inside_area(click_pos, modules_area) {
        log::info!("Mouse clicked on Modules pane");
        state.switch_to_modules();
        handle_pane_item_click(mouse, modules_area, state, Focus::Modules);
    } else if is_inside_area(click_pos, profiles_area) {
        log::info!("Mouse clicked on Profiles pane");
        state.switch_to_profiles();
        handle_pane_item_click(mouse, profiles_area, state, Focus::Profiles);
    } else if is_inside_area(click_pos, flags_area) {
        log::info!("Mouse clicked on Flags pane");
        state.switch_to_flags();
        handle_pane_item_click(mouse, flags_area, state, Focus::Flags);
    } else if is_inside_area(click_pos, output_area) {
        log::info!("Mouse clicked on Output pane");
        state.focus_output();
    }
}

/// Handle clicking on an item within a pane to select it
fn handle_pane_item_click(
    mouse: crossterm::event::MouseEvent,
    area: ratatui::layout::Rect,
    state: &mut crate::ui::state::TuiState,
    focus: Focus,
) {
    // Calculate which item was clicked based on row position within pane
    // Account for border (1 line top) and title
    if mouse.row <= area.y + 1 {
        return; // Clicked on border/title
    }

    let item_index = (mouse.row - area.y - 2) as usize; // -2 for border and title

    match focus {
        Focus::Modules => {
            if item_index < state.modules.len() {
                state.modules_list_state.select(Some(item_index));
                state.sync_selected_module_output();
                log::debug!("Selected module at index {}", item_index);
            }
        }
        Focus::Profiles => {
            if item_index < state.profiles.len() {
                state.profiles_list_state.select(Some(item_index));
                log::debug!("Selected profile at index {}", item_index);
            }
        }
        Focus::Flags => {
            if item_index < state.flags.len() {
                state.flags_list_state.select(Some(item_index));
                log::debug!("Selected flag at index {}", item_index);
            }
        }
        _ => {}
    }
}

/// Check if a position (column, row) is inside a Rect area
fn is_inside_area(pos: (u16, u16), area: ratatui::layout::Rect) -> bool {
    let (col, row) = pos;
    col >= area.x && col < area.x + area.width && row >= area.y && row < area.y + area.height
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::keybindings::CurrentView;
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
    #[cfg(unix)] // Shell script execution not supported on Windows
    fn test_build_command_runs_clean_install() {
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
        let modules = vec!["module1".to_string()];
        let project_root = PathBuf::from("/");
        let state = crate::ui::state::TuiState::new(modules, project_root, test_cfg());

        // Check flags are initialized
        assert!(!state.flags.is_empty(), "Flags should be initialized");

        // Check all flags start disabled
        assert_eq!(
            state.enabled_flag_names().len(),
            0,
            "All flags should start disabled"
        );
    }

    #[test]
    fn test_navigation_debouncing() {
        use std::{thread, time::Duration};

        let modules = vec![
            "module1".to_string(),
            "module2".to_string(),
            "module3".to_string(),
        ];
        let project_root = PathBuf::from("/");
        let mut state = crate::ui::state::TuiState::new(modules, project_root, test_cfg());

        // Initial selection should be module1 (index 0)
        assert_eq!(state.modules_list_state.selected(), Some(0));

        // Rapid down presses - should only move once due to debouncing
        for _ in 0..5 {
            handle_key_event(
                crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Down),
                &mut state,
            );
        }

        // Should only have moved to index 1, not 5 (due to debouncing)
        assert_eq!(state.modules_list_state.selected(), Some(1));

        // Wait for debounce period to pass
        thread::sleep(Duration::from_millis(110));

        // Now another press should work
        handle_key_event(
            crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Down),
            &mut state,
        );
        assert_eq!(state.modules_list_state.selected(), Some(2));
    }

    #[test]
    fn test_profile_selection() {
        let modules = vec!["module1".to_string()];
        let project_root = PathBuf::from("/");
        let mut state = crate::ui::state::TuiState::new(modules, project_root, test_cfg());

        // Add some profiles
        state.set_profiles(vec!["dev".to_string(), "prod".to_string()]);

        // Switch to profiles view
        state.switch_to_profiles();

        // Verify focus is on profiles
        assert_eq!(state.focus, Focus::Profiles);
        assert_eq!(state.current_view, CurrentView::Profiles);

        // No profiles should be active initially
        assert_eq!(state.active_profiles.len(), 0);

        // Simulate pressing Enter to toggle profile
        handle_key_event(
            crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Enter),
            &mut state,
        );

        // First profile should now be active
        assert_eq!(state.active_profiles.len(), 1);
        assert_eq!(state.active_profiles[0], "dev");

        // Press Enter again to deactivate
        handle_key_event(
            crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Enter),
            &mut state,
        );

        // Profile should be deactivated
        assert_eq!(state.active_profiles.len(), 0);
    }

    #[test]
    fn test_flag_selection() {
        let modules = vec!["module1".to_string()];
        let project_root = PathBuf::from("/");
        let mut state = crate::ui::state::TuiState::new(modules, project_root, test_cfg());

        // Switch to flags view
        state.switch_to_flags();

        // Verify focus is on flags
        assert_eq!(state.focus, Focus::Flags);
        assert_eq!(state.current_view, CurrentView::Flags);

        // No flags should be enabled initially
        let enabled_count = state.flags.iter().filter(|f| f.enabled).count();
        assert_eq!(enabled_count, 0);

        // Simulate pressing Space to toggle flag
        handle_key_event(
            crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char(' ')),
            &mut state,
        );

        // First flag should now be enabled
        let enabled_count = state.flags.iter().filter(|f| f.enabled).count();
        assert_eq!(enabled_count, 1);
        assert!(state.flags[0].enabled);

        // Press Space again to disable
        handle_key_event(
            crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char(' ')),
            &mut state,
        );

        // Flag should be disabled
        let enabled_count = state.flags.iter().filter(|f| f.enabled).count();
        assert_eq!(enabled_count, 0);
    }

    #[test]
    fn test_mouse_pane_focus() {
        let modules = vec!["module1".to_string()];
        let project_root = PathBuf::from("/");
        let mut state = crate::ui::state::TuiState::new(modules, project_root, test_cfg());

        // Initial focus is on Modules
        assert_eq!(state.focus, Focus::Modules);

        // Simulate mouse click on output pane (right side)
        // Based on 30/70 split, output pane starts at column ~24 for 80 cols terminal
        let mouse_event = crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
            column: 50, // Right side of screen - output pane
            row: 5,
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
        assert!(is_inside_area((15, 8), area));
        assert!(is_inside_area((10, 5), area)); // Top-left corner
        assert!(is_inside_area((29, 14), area)); // Bottom-right corner (exclusive)

        // Outside
        assert!(!is_inside_area((9, 8), area)); // Left of area
        assert!(!is_inside_area((30, 8), area)); // Right of area
        assert!(!is_inside_area((15, 4), area)); // Above area
        assert!(!is_inside_area((15, 15), area)); // Below area
    }

    #[test]
    fn test_mouse_click_selects_item() {
        let modules = vec![
            "module1".to_string(),
            "module2".to_string(),
            "module3".to_string(),
        ];
        let project_root = PathBuf::from("/");
        let mut state = crate::ui::state::TuiState::new(modules, project_root, test_cfg());

        // Initial selection is first module
        assert_eq!(state.modules_list_state.selected(), Some(0));

        // Simulate mouse click on modules pane, row 5 (which would be ~3rd item)
        // Assuming modules pane starts at y=3, row 5 would be item at index 0 (5-3-2=0)
        let mouse_event = crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
            column: 5, // Left side - modules pane
            row: 6,    // Within modules pane
            modifiers: crossterm::event::KeyModifiers::empty(),
        };

        handle_mouse_event(mouse_event, &mut state);

        // Should have switched focus to modules
        assert_eq!(state.focus, Focus::Modules);
        // Selection should have been updated based on click position
        assert!(state.modules_list_state.selected().is_some());
    }
}
