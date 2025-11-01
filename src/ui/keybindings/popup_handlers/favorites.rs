use crossterm::event::{KeyCode, KeyEvent};
use crate::ui::state::TuiState;

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
