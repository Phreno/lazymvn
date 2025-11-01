use crossterm::event::{KeyCode, KeyEvent};
use crate::ui::state::TuiState;

pub fn handle_package_selector(key: KeyEvent, state: &mut TuiState) -> bool {
    match key.code {
        KeyCode::Char(ch)
            if !key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL) =>
        {
            log::debug!("Package filter input: '{}'", ch);
            state.push_package_filter_char(ch);
        }
        KeyCode::Backspace => {
            log::debug!("Package filter backspace");
            state.pop_package_filter_char();
        }
        KeyCode::Down => {
            log::debug!("Next package");
            state.next_package();
        }
        KeyCode::Up => {
            log::debug!("Previous package");
            state.previous_package();
        }
        KeyCode::Enter => {
            log::info!("Select and add package to config");
            state.select_and_add_package();
        }
        KeyCode::Esc => {
            log::info!("Cancel package selection");
            state.hide_package_selector();
        }
        _ => {}
    }
    true
}
