//! Custom goals popup management

use super::TuiState;

impl TuiState {
    pub fn show_custom_goals_popup(&mut self) {
        log::info!("Showing custom goals popup");
        
        let tab = self.get_active_tab_mut();
        if tab.custom_goals.is_empty() {
            log::warn!("No custom goals defined in configuration");
            tab.command_output = vec![
                "No custom goals defined.".to_string(),
                "Add goals to your lazymvn.toml config:".to_string(),
                "[maven]".to_string(),
                "custom_goals = [".to_string(),
                "  { name = \"Format\", goal = \"formatter:format\" }".to_string(),
                "]".to_string(),
            ];
            return;
        }

        self.show_custom_goals_popup = true;

        // Select first goal by default
        let tab = self.get_active_tab_mut();
        if !tab.custom_goals.is_empty() {
            tab.custom_goals_list_state.select(Some(0));
        }
    }

    pub fn close_custom_goals_popup(&mut self) {
        log::info!("Closing custom goals popup");
        self.show_custom_goals_popup = false;
    }
}
