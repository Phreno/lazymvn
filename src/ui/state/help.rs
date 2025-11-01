//! Help popup management for TUI

use super::TuiState;

impl TuiState {
    pub fn show_help_popup(&mut self) {
        log::info!("Showing help popup");
        self.show_help_popup = true;
        self.help_search_query.clear();
        // Select first item
        self.help_list_state.select(Some(0));
    }

    pub fn hide_help_popup(&mut self) {
        log::info!("Hiding help popup");
        self.show_help_popup = false;
        self.help_search_query.clear();
    }
}
