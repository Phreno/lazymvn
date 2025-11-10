//! Keybinding handling for the TUI
//!
//! Processes keyboard events and updates application state accordingly.

mod command_keys;
mod helpers;
mod keybinding_data;
mod navigation_keys;
mod output_keys;
mod popup_handlers;
mod search_keys;
mod types;
mod ui_builders;

pub use keybinding_data::{get_all_keybindings, Keybinding, KeybindingAction};
pub use types::{CurrentView, Focus, SearchMode};
pub use ui_builders::{
    blank_line, build_navigation_line, simplified_footer_body, simplified_footer_title,
};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

/// Handle key events and update TUI state accordingly
pub fn handle_key_event(key: KeyEvent, state: &mut crate::ui::state::TuiState) {
    // Only process key press events, ignore release and repeat events
    // This prevents duplicate actions on Windows and some terminals
    if key.kind != KeyEventKind::Press {
        log::debug!("Ignoring non-press key event: {:?}", key.kind);
        return;
    }

    log::debug!("Key event: {:?}", key);

    // Handle save favorite popup separately
    if state.show_save_favorite_popup && popup_handlers::handle_save_favorite_popup(key, state) {
        return;
    }

    // Handle favorites popup separately
    if state.show_favorites_popup && popup_handlers::handle_favorites_popup(key, state) {
        return;
    }

    // Handle command history popup separately
    if state.show_history_popup && popup_handlers::handle_history_popup(key, state) {
        return;
    }

    // Handle projects popup separately
    if state.show_projects_popup && popup_handlers::handle_projects_popup(key, state) {
        return;
    }

    if let Some(search_mod) = state.search_mod.take() {
        log::debug!(
            "In search mode: {:?}",
            match search_mod {
                SearchMode::Input => "Input",
                SearchMode::Cycling => "Cycling",
            }
        );
        let handled = match search_mod {
            SearchMode::Input => search_keys::handle_search_input(key, state),
            SearchMode::Cycling => search_keys::handle_search_cycling(key, state),
        };

        if handled {
            return;
        }
        // If not handled (search cycling with unrecognized key), fall through to normal handling
    }

    // Handle starter selector popup
    if state.show_starter_selector && popup_handlers::handle_starter_selector(key, state) {
        return;
    }

    // Handle package selector popup
    if state.show_package_selector && popup_handlers::handle_package_selector(key, state) {
        return;
    }

    // Handle starter manager popup
    if state.show_starter_manager && popup_handlers::handle_starter_manager(key, state) {
        return;
    }

    // Handle custom goals popup
    if state.show_custom_goals_popup && popup_handlers::handle_custom_goals_popup(key, state) {
        return;
    }

    // Handle help popup
    if state.show_help_popup && popup_handlers::handle_help_popup(key, state) {
        return;
    }

    // Direct command execution - no menu navigation needed

    // Try Maven command keys first
    if command_keys::handle_maven_command(key, state) {
        return;
    }

    // Try tab operations (Ctrl+T/W/Left/Right)
    if navigation_keys::handle_tab_operations(key, state) {
        return;
    }

    // Try popup triggers (Ctrl+F/S/H/R/E)
    if navigation_keys::handle_popup_triggers(key, state) {
        return;
    }

    // Try view switching (0-4)
    if navigation_keys::handle_view_switching(key, state) {
        return;
    }

    // Try focus cycling (Left/Right arrows)
    if navigation_keys::handle_focus_cycling(key, state) {
        return;
    }

    // Try search operations (/, n, N)
    if navigation_keys::handle_search_operations(key, state) {
        return;
    }

    // Try special actions (Esc, Enter, Space)
    if navigation_keys::handle_special_actions(key, state) {
        return;
    }

    // Handle remaining output operations and scrolling
    match key.code {
        KeyCode::Down => output_keys::handle_scroll_down(state, state.focus),
        KeyCode::Up => output_keys::handle_scroll_up(state, state.focus),
        KeyCode::PageUp => output_keys::handle_page_up(state),
        KeyCode::PageDown => output_keys::handle_page_down(state),
        KeyCode::Home => output_keys::handle_scroll_to_start(state),
        KeyCode::End => output_keys::handle_scroll_to_end(state),
        KeyCode::Char('y') => output_keys::handle_yank_output(state),
        KeyCode::Char('Y') => output_keys::handle_yank_debug_info(state),
        _ => {}
    }
}


#[cfg(test)]
mod tests;
