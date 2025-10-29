use crate::ui::keybindings::{Focus, SearchMode};
use crate::ui::state::TuiState;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle tab management operations (Ctrl+T, Ctrl+W, Ctrl+Left/Right)
pub fn handle_tab_operations(key: KeyEvent, state: &mut TuiState) -> bool {
    if !key.modifiers.contains(KeyModifiers::CONTROL) {
        return false;
    }

    match key.code {
        KeyCode::Char('t') => {
            log::info!("Create new tab (Ctrl+T)");
            state.show_recent_projects();
            true
        }
        KeyCode::Char('w') => {
            log::info!("Close current tab (Ctrl+W)");
            // Only close if not the last tab
            if state.get_tab_count() > 1 {
                let active_index = state.get_active_tab_index();
                match state.close_tab(active_index) {
                    Ok(()) => {
                        log::info!("Tab closed successfully");
                    }
                    Err(e) => {
                        log::error!("Failed to close tab: {}", e);
                    }
                }
            } else {
                log::warn!("Cannot close last tab");
            }
            true
        }
        KeyCode::Left => {
            log::info!("Switch to previous tab (Ctrl+Left)");
            state.prev_tab();
            true
        }
        KeyCode::Right => {
            log::info!("Switch to next tab (Ctrl+Right)");
            state.next_tab();
            true
        }
        _ => false,
    }
}

/// Handle popup and configuration operations (Ctrl+F, Ctrl+S, Ctrl+H, Ctrl+R, Ctrl+E, Ctrl+K)
pub fn handle_popup_triggers(key: KeyEvent, state: &mut TuiState) -> bool {
    if !key.modifiers.contains(KeyModifiers::CONTROL) {
        return false;
    }

    match key.code {
        KeyCode::Char('f') => {
            log::info!("Show favorites");
            state.show_favorites_popup = true;
            if !state.favorites.is_empty() {
                state.favorites_list_state.select(Some(0));
            }
            true
        }
        KeyCode::Char('s') => {
            log::info!("Save current as favorite");
            state.show_save_favorite_dialog_from_current();
            true
        }
        KeyCode::Char('h') => {
            log::info!("Show command history");
            state.show_history_popup = true;
            // Reset selection to first item
            if !state.command_history.entries().is_empty() {
                state.history_list_state.select(Some(0));
            }
            true
        }
        KeyCode::Char('r') => {
            log::info!("Show recent projects");
            state.show_recent_projects();
            true
        }
        KeyCode::Char('e') => {
            log::info!("Edit configuration");
            state.edit_config();
            true
        }
        KeyCode::Char('k') => {
            log::info!("Refresh caches (profiles and starters)");
            state.refresh_caches();
            true
        }
        KeyCode::Char('g') => {
            log::info!("Show custom goals popup");
            state.show_custom_goals_popup();
            true
        }
        _ => false,
    }
}

/// Handle view switching (0-4 keys) and help display (?)
pub fn handle_view_switching(key: KeyEvent, state: &mut TuiState) -> bool {
    match key.code {
        KeyCode::Char('0') => {
            log::info!("Focus output pane");
            state.focus_output();
            true
        }
        KeyCode::Char('1') => {
            log::info!("Switch to projects view");
            state.switch_to_projects();
            true
        }
        KeyCode::Char('2') => {
            log::info!("Switch to modules view");
            state.switch_to_modules();
            true
        }
        KeyCode::Char('3') => {
            log::info!("Switch to profiles view");
            state.switch_to_profiles();
            true
        }
        KeyCode::Char('4') => {
            log::info!("Switch to flags view");
            state.switch_to_flags();
            true
        }
        KeyCode::Char('?') => {
            log::info!("Show help popup");
            state.show_help_popup();
            true
        }
        _ => false,
    }
}

/// Handle focus cycling (Left/Right arrow keys without modifiers)
pub fn handle_focus_cycling(key: KeyEvent, state: &mut TuiState) -> bool {
    // Only handle if no modifiers
    if key.modifiers != KeyModifiers::NONE {
        return false;
    }

    match key.code {
        KeyCode::Left => {
            log::debug!("Cycle focus left");
            state.cycle_focus_left();
            true
        }
        KeyCode::Right => {
            log::debug!("Cycle focus right");
            state.cycle_focus_right();
            true
        }
        _ => false,
    }
}

/// Handle search operations (/, n, N)
pub fn handle_search_operations(key: KeyEvent, state: &mut TuiState) -> bool {
    match key.code {
        KeyCode::Char('/') => {
            log::info!("Begin search input");
            state.begin_search_input();
            state.search_mod = Some(SearchMode::Input);
            true
        }
        KeyCode::Char('n') => {
            log::debug!("Next search match");
            state.next_search_match();
            true
        }
        KeyCode::Char('N') => {
            log::debug!("Previous search match");
            state.previous_search_match();
            true
        }
        _ => false,
    }
}

/// Handle special actions (Esc, Enter, Space)
pub fn handle_special_actions(key: KeyEvent, state: &mut TuiState) -> bool {
    match key.code {
        KeyCode::Esc => {
            log::info!("Kill running process with Escape");
            state.kill_running_process();
            true
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            if state.focus == Focus::Profiles {
                state.toggle_profile();
                true
            } else if state.focus == Focus::Flags {
                state.toggle_flag();
                true
            } else {
                false
            }
        }
        _ => false,
    }
}
