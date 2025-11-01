use crossterm::event::{KeyCode, KeyEvent};
use crate::ui::keybindings::{KeybindingAction, CurrentView, Focus};
use crate::ui::state::TuiState;

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

/// Handle keyboard events for package selector
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
