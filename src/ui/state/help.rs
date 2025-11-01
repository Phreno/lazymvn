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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;
    use std::path::PathBuf;

    #[test]
    fn test_show_help_popup() {
        let config = Config::default();
        let mut state = TuiState::new(vec![], PathBuf::from("/tmp"), config);
        
        assert!(!state.show_help_popup);
        state.show_help_popup();
        assert!(state.show_help_popup);
        assert_eq!(state.help_search_query, "");
        assert_eq!(state.help_list_state.selected(), Some(0));
    }

    #[test]
    fn test_hide_help_popup() {
        let config = Config::default();
        let mut state = TuiState::new(vec![], PathBuf::from("/tmp"), config);
        
        state.show_help_popup = true;
        state.help_search_query = "test".to_string();
        
        state.hide_help_popup();
        assert!(!state.show_help_popup);
        assert_eq!(state.help_search_query, "");
    }
}
