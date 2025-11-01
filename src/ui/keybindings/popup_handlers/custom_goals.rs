use crossterm::event::{KeyCode, KeyEvent};
use crate::ui::keybindings::{KeybindingAction, CurrentView, Focus};
use crate::ui::state::TuiState;

pub fn handle_custom_goals_popup(key: KeyEvent, state: &mut TuiState) -> bool {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            log::info!("Close custom goals popup");
            state.close_custom_goals_popup();
        }
        KeyCode::Down => {
            log::debug!("Navigate down in custom goals");
            let tab = state.get_active_tab();
            let len = tab.custom_goals.len();
            if len > 0 {
                let tab = state.get_active_tab_mut();
                let i = match tab.custom_goals_list_state.selected() {
                    Some(i) => {
                        if i >= len - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                tab.custom_goals_list_state.select(Some(i));
            }
        }
        KeyCode::Up => {
            log::debug!("Navigate up in custom goals");
            let tab = state.get_active_tab();
            let len = tab.custom_goals.len();
            if len > 0 {
                let tab = state.get_active_tab_mut();
                let i = match tab.custom_goals_list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            len - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                tab.custom_goals_list_state.select(Some(i));
            }
        }
        KeyCode::Enter => {
            log::info!("Execute custom goal");
            let tab = state.get_active_tab();
            if let Some(selected) = tab.custom_goals_list_state.selected()
                && selected < tab.custom_goals.len()
            {
                state.close_custom_goals_popup();
                state.run_custom_goal(selected);
            }
        }
        _ => {}
    }
    true
}
