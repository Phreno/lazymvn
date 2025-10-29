//! Popup keyboard event handlers
//!
//! This module handles keyboard events for all popup dialogs.

use crate::ui::state::TuiState;
use crossterm::event::{KeyCode, KeyEvent};

/// Handle keyboard events for save favorite popup
pub fn handle_save_favorite_popup(key: KeyEvent, state: &mut TuiState) -> bool {
    match key.code {
        KeyCode::Char(ch) => {
            state.favorite_name_input.push(ch);
        }
        KeyCode::Backspace => {
            state.favorite_name_input.pop();
        }
        KeyCode::Enter => {
            if !state.favorite_name_input.trim().is_empty() {
                // Get current goal from the last executed command or default
                let goal = state
                    .get_last_executed_command()
                    .map(|(cmd, _, _)| cmd)
                    .unwrap_or_else(|| "install".to_string());
                state.save_pending_favorite(goal);
                log::info!("Favorite saved");
            }
        }
        KeyCode::Esc => {
            state.cancel_save_favorite();
        }
        _ => {}
    }
    true
}

/// Handle keyboard events for favorites popup
pub fn handle_favorites_popup(key: KeyEvent, state: &mut TuiState) -> bool {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            log::info!("Close favorites popup");
            state.show_favorites_popup = false;
            state.favorites_filter.clear();
        }
        KeyCode::Delete | KeyCode::Char('d') => {
            log::info!("Delete favorite");
            state.delete_selected_favorite();
        }
        KeyCode::Char(ch)
            if !key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL) =>
        {
            log::debug!("Favorites filter input: '{}'", ch);
            state.push_favorites_filter_char(ch);
        }
        KeyCode::Backspace => {
            log::debug!("Favorites filter backspace");
            state.pop_favorites_filter_char();
        }
        KeyCode::Down => {
            log::debug!("Navigate down in favorites");
            let len = state.get_filtered_favorites().len();
            if len > 0 {
                let i = match state.favorites_list_state.selected() {
                    Some(i) => {
                        if i >= len - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                state.favorites_list_state.select(Some(i));
            }
        }
        KeyCode::Up => {
            log::debug!("Navigate up in favorites");
            let len = state.get_filtered_favorites().len();
            if len > 0 {
                let i = match state.favorites_list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            len - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                state.favorites_list_state.select(Some(i));
            }
        }
        KeyCode::Enter => {
            log::info!("Execute favorite");
            if let Some(selected) = state.favorites_list_state.selected()
                && let Some(fav) = state.get_filtered_favorites().get(selected).cloned()
            {
                state.apply_favorite(&fav);
                state.show_favorites_popup = false;
                state.favorites_filter.clear();
            }
        }
        _ => {}
    }
    true
}

/// Handle keyboard events for history popup
pub fn handle_history_popup(key: KeyEvent, state: &mut TuiState) -> bool {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            log::info!("Close history popup");
            state.show_history_popup = false;
            state.history_filter.clear();
        }
        KeyCode::Char('s')
            if key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL) =>
        {
            log::info!("Save selected history entry as favorite");
            if let Some(selected) = state.history_list_state.selected()
                && let Some(entry) = state.get_filtered_history().get(selected).cloned()
            {
                state.pending_favorite = Some(entry);
                state.show_history_popup = false;
                state.show_save_favorite_popup = true;
                state.favorite_name_input.clear();
            }
        }
        KeyCode::Char(ch)
            if !key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL) =>
        {
            log::debug!("History filter input: '{}'", ch);
            state.push_history_filter_char(ch);
        }
        KeyCode::Backspace => {
            log::debug!("History filter backspace");
            state.pop_history_filter_char();
        }
        KeyCode::Down => {
            log::debug!("Navigate down in history");
            let len = state.get_filtered_history().len();
            if len > 0 {
                let i = match state.history_list_state.selected() {
                    Some(i) => {
                        if i >= len - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                state.history_list_state.select(Some(i));
            }
        }
        KeyCode::Up => {
            log::debug!("Navigate up in history");
            let len = state.get_filtered_history().len();
            if len > 0 {
                let i = match state.history_list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            len - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                state.history_list_state.select(Some(i));
            }
        }
        KeyCode::Enter => {
            log::info!("Execute command from history");
            if let Some(selected) = state.history_list_state.selected()
                && let Some(entry) = state.get_filtered_history().get(selected)
            {
                // Apply the command's configuration
                state.apply_history_entry(entry.clone());
                state.show_history_popup = false;
            }
        }
        _ => {}
    }
    true
}

/// Handle keyboard events for projects popup
pub fn handle_projects_popup(key: KeyEvent, state: &mut TuiState) -> bool {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            log::info!("Cancel project selection");
            state.hide_recent_projects();
        }
        KeyCode::Char(ch)
            if !key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL) =>
        {
            log::debug!("Projects filter input: '{}'", ch);
            state.push_projects_filter_char(ch);
        }
        KeyCode::Backspace => {
            log::debug!("Projects filter backspace");
            state.pop_projects_filter_char();
        }
        KeyCode::Down => {
            log::debug!("Navigate down in projects list");
            state.next_project();
        }
        KeyCode::Up => {
            log::debug!("Navigate up in projects list");
            state.previous_project();
        }
        KeyCode::Enter => {
            log::info!("Select project from recent list");
            state.select_current_project();
        }
        _ => {}
    }
    true
}

/// Handle keyboard events for starter selector
pub fn handle_starter_selector(key: KeyEvent, state: &mut TuiState) -> bool {
    match key.code {
        KeyCode::Char(ch)
            if !key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL) =>
        {
            log::debug!("Starter filter input: '{}'", ch);
            state.push_starter_filter_char(ch);
        }
        KeyCode::Backspace => {
            log::debug!("Starter filter backspace");
            state.pop_starter_filter_char();
        }
        KeyCode::Down => {
            log::debug!("Next starter");
            state.next_starter();
        }
        KeyCode::Up => {
            log::debug!("Previous starter");
            state.previous_starter();
        }
        KeyCode::Enter => {
            log::info!("Select and run starter");
            state.select_and_run_starter();
        }
        KeyCode::Esc => {
            log::info!("Cancel starter selection");
            state.hide_starter_selector();
        }
        _ => {}
    }
    true
}

/// Handle keyboard events for starter manager
pub fn handle_starter_manager(key: KeyEvent, state: &mut TuiState) -> bool {
    match key.code {
        KeyCode::Down => {
            log::debug!("Next starter in manager");
            state.next_starter();
        }
        KeyCode::Up => {
            log::debug!("Previous starter in manager");
            state.previous_starter();
        }
        KeyCode::Enter => {
            log::info!("Run selected starter from manager");
            let tab = state.get_active_tab();
            if let Some(idx) = state.starters_list_state.selected()
                && let Some(starter) = tab.starters_cache.starters.get(idx)
            {
                let fqcn = starter.fully_qualified_class_name.clone();
                state.run_spring_boot_starter(&fqcn);
                state.hide_starter_manager();
            }
        }
        KeyCode::Char(' ') => {
            log::info!("Toggle starter default");
            state.toggle_starter_default();
        }
        KeyCode::Char('d') | KeyCode::Delete => {
            log::info!("Delete starter");
            state.remove_selected_starter();
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            log::info!("Close starter manager");
            state.hide_starter_manager();
        }
        _ => {}
    }
    true
}

/// Handle keyboard events for help popup
pub fn handle_help_popup(key: KeyEvent, state: &mut TuiState) -> bool {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
            log::info!("Close help popup");
            state.hide_help_popup();
        }
        _ => {}
    }
    true
}
