//! Mouse event handling for TUI
//!
//! This module provides mouse event handling functionality for the TUI,
//! allowing users to click on panes to switch focus and select items.

use crate::ui::{keybindings::Focus, panes::create_adaptive_layout, state::TuiState};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

/// Handle mouse events for pane navigation
pub fn handle_mouse_event(mouse: MouseEvent, state: &mut TuiState) {
    // Only handle left button clicks
    if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
        return;
    }

    log::debug!("Mouse click at ({}, {})", mouse.column, mouse.row);

    // Get the current layout areas to determine which pane was clicked
    // We need to calculate this based on terminal size
    // Use default size if terminal size cannot be determined (e.g., in tests)
    let terminal_size = match crossterm::terminal::size() {
        Ok((cols, rows)) => (cols, rows),
        Err(_) => (80, 24), // Default terminal size for tests
    };

    // Calculate layout areas using same logic as draw function
    let total_area = ratatui::layout::Rect {
        x: 0,
        y: 0,
        width: terminal_size.0,
        height: terminal_size.1,
    };

    let (
        _tab_bar_area,
        projects_area,
        modules_area,
        profiles_area,
        flags_area,
        output_area,
        _footer_area,
    ) = create_adaptive_layout(total_area, Some(state.focus));

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
    mouse: MouseEvent,
    area: ratatui::layout::Rect,
    state: &mut TuiState,
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
            let needs_sync = {
                let tab = state.get_active_tab_mut();
                if item_index < tab.modules.len() {
                    tab.modules_list_state.select(Some(item_index));
                    log::debug!("Selected module at index {}", item_index);
                    true
                } else {
                    false
                }
            };
            if needs_sync {
                state.sync_selected_module_output();
            }
        }
        Focus::Profiles => {
            let tab = state.get_active_tab_mut();
            if item_index < tab.profiles.len() {
                tab.profiles_list_state.select(Some(item_index));
                log::debug!("Selected profile at index {}", item_index);
            }
        }
        Focus::Flags => {
            let tab = state.get_active_tab_mut();
            if item_index < tab.flags.len() {
                tab.flags_list_state.select(Some(item_index));
                log::debug!("Selected flag at index {}", item_index);
            }
        }
        _ => {}
    }
}

/// Check if a position (column, row) is inside a Rect area
pub(crate) fn is_inside_area(pos: (u16, u16), area: ratatui::layout::Rect) -> bool {
    let (col, row) = pos;
    col >= area.x && col < area.x + area.width && row >= area.y && row < area.y + area.height
}
