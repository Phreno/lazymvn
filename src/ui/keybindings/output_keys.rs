//! Output pane keyboard event handlers
//!
//! This module handles keyboard events specifically for the output pane,
//! including scrolling, clipboard operations, and navigation.

use crate::ui::keybindings::Focus;
use crate::ui::state::TuiState;

/// Handle scroll events based on focus
pub fn handle_scroll_down(state: &mut TuiState, focus: Focus) {
    match focus {
        Focus::Output => {
            log::debug!("Scroll output down");
            state.scroll_output_lines(1);
        }
        _ => {
            log::debug!("Navigate down in list");
            state.next_item();
        }
    }
}

/// Handle scroll up events based on focus
pub fn handle_scroll_up(state: &mut TuiState, focus: Focus) {
    match focus {
        Focus::Output => {
            log::debug!("Scroll output up");
            state.scroll_output_lines(-1);
        }
        _ => {
            log::debug!("Navigate up in list");
            state.previous_item();
        }
    }
}

/// Handle page up in output
pub fn handle_page_up(state: &mut TuiState) {
    log::debug!("Page up");
    state.scroll_output_pages(-1);
}

/// Handle page down in output
pub fn handle_page_down(state: &mut TuiState) {
    log::debug!("Page down");
    state.scroll_output_pages(1);
}

/// Handle scroll to start (Home key)
pub fn handle_scroll_to_start(state: &mut TuiState) {
    log::debug!("Scroll to start");
    state.scroll_output_to_start();
}

/// Handle scroll to end (End key)
pub fn handle_scroll_to_end(state: &mut TuiState) {
    log::debug!("Scroll to end");
    state.scroll_output_to_end();
}

/// Handle yank (copy) output to clipboard
pub fn handle_yank_output(state: &mut TuiState) {
    log::info!("Yank (copy) output to clipboard");
    state.yank_output();
}

/// Handle yank (copy) debug report to clipboard
pub fn handle_yank_debug_info(state: &mut TuiState) {
    log::info!("Yank (copy) debug report to clipboard");
    state.yank_debug_info();
}
