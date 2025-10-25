//! Search mode keyboard event handlers
//!
//! This module handles keyboard events for search functionality,
//! including input mode and results cycling mode.

use crate::ui::state::TuiState;
use crate::ui::keybindings::SearchMode;
use crossterm::event::{KeyCode, KeyEvent};

/// Handle keyboard events in search input mode
pub fn handle_search_input(key: KeyEvent, state: &mut TuiState) -> bool {
    match key.code {
        KeyCode::Char(ch) => {
            log::debug!("Search input: '{}'", ch);
            state.push_search_char(ch);
            state.live_search();
            state.search_mod = Some(SearchMode::Input);
        }
        KeyCode::Backspace => {
            log::debug!("Search backspace");
            state.backspace_search_char();
            state.live_search();
            state.search_mod = Some(SearchMode::Input);
        }
        KeyCode::Up => {
            log::debug!("Search recall previous");
            state.recall_previous_search();
            state.live_search();
            state.search_mod = Some(SearchMode::Input);
        }
        KeyCode::Down => {
            log::debug!("Search recall next");
            state.recall_next_search();
            state.live_search();
            state.search_mod = Some(SearchMode::Input);
        }
        KeyCode::Enter => {
            log::debug!("Search submit");
            state.submit_search();
            if state.has_search_results() {
                log::debug!("Search has results, entering cycling mode");
                state.search_mod = Some(SearchMode::Cycling);
            } else {
                log::debug!("No search results, exiting search");
                state.search_mod = None;
            }
        }
        KeyCode::Esc => {
            log::debug!("Search cancelled");
            state.cancel_search_input();
            state.search_mod = None;
        }
        _ => {
            state.search_mod = Some(SearchMode::Input);
        }
    }
    true
}

/// Handle keyboard events in search cycling mode (navigating results)
pub fn handle_search_cycling(key: KeyEvent, state: &mut TuiState) -> bool {
    match key.code {
        KeyCode::Char('n') => {
            state.next_search_match();
            state.search_mod = Some(SearchMode::Cycling);
        }
        KeyCode::Char('N') => {
            state.previous_search_match();
            state.search_mod = Some(SearchMode::Cycling);
        }
        KeyCode::Char('/') => {
            state.begin_search_input();
            state.search_mod = Some(SearchMode::Input);
        }
        KeyCode::Enter | KeyCode::Esc => {
            state.search_mod = None;
        }
        _ => {
            state.search_mod = None;
            // Re-handle the key event in normal mode
            return false;
        }
    }
    true
}
