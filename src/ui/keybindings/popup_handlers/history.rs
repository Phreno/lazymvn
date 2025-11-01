use crossterm::event::{KeyCode, KeyEvent};
use crate::ui::state::TuiState;

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
