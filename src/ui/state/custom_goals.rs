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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::{Config, MavenConfig, CustomGoal};
    use std::path::PathBuf;

    #[test]
    fn test_show_custom_goals_popup_with_goals() {
        let config = Config {
            maven: Some(MavenConfig {
                custom_goals: vec![CustomGoal {
                    name: "Format".to_string(),
                    goal: "formatter:format".to_string(),
                }],
                custom_flags: vec![],
            }),
            ..Default::default()
        };
        
        let mut state = TuiState::new(vec![], PathBuf::from("/tmp"), config);
        
        assert!(!state.show_custom_goals_popup);
        state.show_custom_goals_popup();
        assert!(state.show_custom_goals_popup);
        
        let tab = state.get_active_tab();
        assert_eq!(tab.custom_goals_list_state.selected(), Some(0));
    }

    #[test]
    fn test_show_custom_goals_popup_no_goals() {
        let config = Config::default();
        let mut state = TuiState::new(vec![], PathBuf::from("/tmp"), config);
        
        state.show_custom_goals_popup();
        assert!(!state.show_custom_goals_popup);
        
        let tab = state.get_active_tab();
        assert!(tab.command_output.iter().any(|line| line.contains("No custom goals")));
    }

    #[test]
    fn test_close_custom_goals_popup() {
        let config = Config::default();
        let mut state = TuiState::new(vec![], PathBuf::from("/tmp"), config);
        
        state.show_custom_goals_popup = true;
        state.close_custom_goals_popup();
        assert!(!state.show_custom_goals_popup);
    }
}
