use crossterm::event::{KeyCode, KeyEvent};
use crate::ui::keybindings::{KeybindingAction, CurrentView, Focus};
use crate::ui::state::TuiState;

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
