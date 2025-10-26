//! Search functionality for navigating command output
//!
//! This module handles regex-based search across Maven command output,
//! including search history, match navigation, and visual highlighting.

use super::TuiState;
use crate::ui::search::{SearchMatch, SearchState, collect_search_matches};
use regex::Regex;

impl TuiState {
    /// Check if there are active search results
    pub fn has_search_results(&self) -> bool {
        self.search_state
            .as_ref()
            .map(|s| s.has_matches())
            .unwrap_or(false)
    }

    /// Perform live search without committing to history
    pub fn live_search(&mut self) {
        if let Some(pattern) = self.search_input.clone() {
            if pattern.is_empty() {
                self.search_state = None;
                self.search_error = None;
                return;
            }
            match self.apply_search_query(pattern.clone(), false) {
                Ok(_) => self.search_error = None,
                Err(e) => self.search_error = Some(e.to_string()),
            }
        }
    }

    /// Begin search input mode
    pub fn begin_search_input(&mut self) {
        self.search_input = Some(String::new());
        self.search_history_index = None;
        self.search_error = None;
    }

    /// Cancel search input mode
    pub fn cancel_search_input(&mut self) {
        self.search_input = None;
        self.search_history_index = None;
        self.search_error = None;
    }

    /// Add character to search input
    pub fn push_search_char(&mut self, ch: char) {
        if let Some(buffer) = self.search_input.as_mut() {
            buffer.push(ch);
            self.search_history_index = None;
        }
    }

    /// Remove last character from search input
    pub fn backspace_search_char(&mut self) {
        if let Some(buffer) = self.search_input.as_mut() {
            buffer.pop();
            self.search_history_index = None;
        }
    }

    /// Recall previous search from history (up arrow)
    pub fn recall_previous_search(&mut self) {
        if self.search_history.is_empty() {
            return;
        }
        let len = self.search_history.len();
        let next_index = match self.search_history_index {
            None => Some(len - 1),
            Some(0) => Some(0),
            Some(i) => Some(i - 1),
        };
        if let Some(idx) = next_index
            && let Some(query) = self.search_history.get(idx)
        {
            self.search_input = Some(query.clone());
            self.search_history_index = Some(idx);
        }
    }

    /// Recall next search from history (down arrow)
    pub fn recall_next_search(&mut self) {
        if self.search_history.is_empty() {
            return;
        }
        if let Some(idx) = self.search_history_index {
            if idx + 1 < self.search_history.len() {
                let query = self.search_history[idx + 1].clone();
                self.search_input = Some(query);
                self.search_history_index = Some(idx + 1);
            } else {
                self.search_input = Some(String::new());
                self.search_history_index = None;
            }
        }
    }

    /// Submit search query and add to history
    pub fn submit_search(&mut self) {
        if let Some(pattern) = self.search_input.clone() {
            if !pattern.is_empty() {
                match self.apply_search_query(pattern.clone(), false) {
                    Ok(_) => {
                        if !self.search_history.contains(&pattern) {
                            self.search_history.push(pattern);
                        }
                        self.search_input = None;
                        self.search_error = None;
                        self.search_history_index = None;
                    }
                    Err(e) => {
                        self.search_error = Some(e.to_string());
                    }
                }
            } else {
                self.search_input = None;
                self.search_state = None;
                self.search_error = None;
                self.search_history_index = None;
            }
        }
    }

    /// Apply search query with regex matching
    fn apply_search_query(
        &mut self,
        query: String,
        keep_current: bool,
    ) -> Result<(), regex::Error> {
        let regex = Regex::new(&query)?;
        let tab = self.get_active_tab();
        let matches = collect_search_matches(&tab.command_output, &regex);
        let mut current_index = 0usize;

        if keep_current && let Some(existing) = self.search_state.as_ref() {
            current_index = existing.current.min(matches.len().saturating_sub(1));
        }

        self.search_state = Some(SearchState::new(query, matches));

        let current_match = if let Some(search) = self.search_state.as_mut() {
            search.jump_to_match(current_index);
            search.current_match().cloned()
        } else {
            None
        };

        if let Some(match_to_center) = current_match {
            self.center_on_match(match_to_center);
        }

        Ok(())
    }

    /// Refresh search matches after output changes
    pub(super) fn refresh_search_matches(&mut self) {
        if let Some(existing) = self.search_state.as_ref().cloned() {
            let _ = self.apply_search_query(existing.query, true);
        } else {
            self.search_state = None;
        }
    }

    /// Navigate to next search match
    pub fn next_search_match(&mut self) {
        let current_match = if let Some(search) = self.search_state.as_mut() {
            if search.has_matches() {
                search.next_match();
                search.current_match().cloned()
            } else {
                None
            }
        } else {
            None
        };

        if let Some(match_to_center) = current_match {
            self.center_on_match(match_to_center);
        }
    }

    /// Navigate to previous search match
    pub fn previous_search_match(&mut self) {
        let current_match = if let Some(search) = self.search_state.as_mut() {
            if search.has_matches() {
                search.previous_match();
                search.current_match().cloned()
            } else {
                None
            }
        } else {
            None
        };

        if let Some(match_to_center) = current_match {
            self.center_on_match(match_to_center);
        }
    }

    /// Center view on a specific search match
    fn center_on_match(&mut self, target: SearchMatch) {
        self.pending_center = Some(target);
        self.apply_pending_center();
    }

    /// Apply pending center operation after output metrics update
    pub(super) fn apply_pending_center(&mut self) {
        let target = match self.pending_center.clone() {
            Some(t) => t,
            None => return,
        };
        let tab = self.get_active_tab_mut();
        if tab.output_view_height == 0 || tab.output_area_width == 0 {
            return;
        }
        let metrics = match tab.output_metrics.as_ref() {
            Some(m) => m,
            None => return,
        };
        let total_rows = metrics.total_rows();
        if total_rows == 0 {
            self.pending_center = None;
            return;
        }
        if let Some(target_row) = metrics.row_for_match(&target) {
            let view_height = tab.output_view_height as usize;
            let desired_offset = target_row.saturating_sub(view_height / 2);
            let max_offset = total_rows.saturating_sub(view_height);
            tab.output_offset = desired_offset.min(max_offset);
            self.store_current_module_output();
        }
        self.pending_center = None;
    }

    /// Ensure current search match is visible in output view
    pub(super) fn ensure_current_match_visible(&mut self) {
        if let Some(search) = self.search_state.as_ref()
            && let Some(current_match) = search.current_match()
        {
            self.center_on_match(current_match.clone());
        }
    }

    /// Get styling information for search highlighting on a specific line
    pub fn search_line_style(
        &self,
        line_index: usize,
    ) -> Option<Vec<(ratatui::style::Style, std::ops::Range<usize>)>> {
        self.search_state
            .as_ref()
            .and_then(|search| crate::ui::search::search_line_style(line_index, search))
    }

    /// Get formatted status line for search UI
    pub fn search_status_line(&self) -> Option<ratatui::text::Line<'static>> {
        crate::ui::search::search_status_line(
            self.search_input.as_deref(),
            self.search_error.as_deref(),
            self.search_state.as_ref(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;
    use std::path::PathBuf;

    fn create_test_state() -> TuiState {
        TuiState::new(
            vec!["module1".to_string()],
            PathBuf::from("/test"),
            Config::default(),
        )
    }

    fn setup_output(state: &mut TuiState, lines: Vec<&str>) {
        let tab = state.get_active_tab_mut();
        tab.command_output = lines.iter().map(|s| s.to_string()).collect();
    }

    #[test]
    fn test_has_search_results_false() {
        let state = create_test_state();
        assert!(!state.has_search_results());
    }

    #[test]
    fn test_begin_search_input() {
        let mut state = create_test_state();

        state.begin_search_input();

        assert!(state.search_input.is_some());
        assert_eq!(state.search_input.as_ref().unwrap(), "");
        assert!(state.search_history_index.is_none());
        assert!(state.search_error.is_none());
    }

    #[test]
    fn test_cancel_search_input() {
        let mut state = create_test_state();
        state.search_input = Some("test".to_string());
        state.search_history_index = Some(0);
        state.search_error = Some("error".to_string());

        state.cancel_search_input();

        assert!(state.search_input.is_none());
        assert!(state.search_history_index.is_none());
        assert!(state.search_error.is_none());
    }

    #[test]
    fn test_push_search_char() {
        let mut state = create_test_state();
        state.begin_search_input();

        state.push_search_char('t');
        state.push_search_char('e');
        state.push_search_char('s');
        state.push_search_char('t');

        assert_eq!(state.search_input.as_ref().unwrap(), "test");
    }

    #[test]
    fn test_backspace_search_char() {
        let mut state = create_test_state();
        state.search_input = Some("test".to_string());

        state.backspace_search_char();

        assert_eq!(state.search_input.as_ref().unwrap(), "tes");
    }

    #[test]
    fn test_backspace_search_char_empty() {
        let mut state = create_test_state();
        state.search_input = Some("".to_string());

        state.backspace_search_char();

        assert_eq!(state.search_input.as_ref().unwrap(), "");
    }

    #[test]
    fn test_submit_search_adds_to_history() {
        let mut state = create_test_state();
        setup_output(&mut state, vec!["ERROR: test failed", "SUCCESS: all ok"]);

        state.search_input = Some("ERROR".to_string());
        state.submit_search();

        assert!(state.search_history.contains(&"ERROR".to_string()));
        assert!(state.search_input.is_none());
    }

    #[test]
    fn test_submit_search_empty_clears_state() {
        let mut state = create_test_state();
        state.search_input = Some("".to_string());

        state.submit_search();

        assert!(state.search_input.is_none());
        assert!(state.search_state.is_none());
    }

    #[test]
    fn test_submit_search_invalid_regex() {
        let mut state = create_test_state();
        state.search_input = Some("[invalid".to_string());

        state.submit_search();

        assert!(state.search_error.is_some());
        assert!(state.search_input.is_some()); // Input retained on error
    }

    #[test]
    fn test_live_search_empty_clears() {
        let mut state = create_test_state();
        state.search_input = Some("".to_string());

        state.live_search();

        assert!(state.search_state.is_none());
        assert!(state.search_error.is_none());
    }

    #[test]
    fn test_live_search_with_pattern() {
        let mut state = create_test_state();
        setup_output(&mut state, vec!["ERROR: test", "INFO: ok"]);
        state.search_input = Some("ERROR".to_string());

        state.live_search();

        assert!(state.search_state.is_some());
        assert!(state.search_error.is_none());
    }

    #[test]
    fn test_recall_previous_search_empty_history() {
        let mut state = create_test_state();
        state.begin_search_input();

        state.recall_previous_search();

        assert_eq!(state.search_input.as_ref().unwrap(), "");
    }

    #[test]
    fn test_recall_previous_search() {
        let mut state = create_test_state();
        state.search_history.push("query1".to_string());
        state.search_history.push("query2".to_string());
        state.search_input = Some("".to_string());

        state.recall_previous_search();

        assert_eq!(state.search_input.as_ref().unwrap(), "query2");
        assert_eq!(state.search_history_index, Some(1));
    }

    #[test]
    fn test_recall_previous_search_twice() {
        let mut state = create_test_state();
        state.search_history.push("query1".to_string());
        state.search_history.push("query2".to_string());
        state.search_input = Some("".to_string());

        state.recall_previous_search();
        state.recall_previous_search();

        assert_eq!(state.search_input.as_ref().unwrap(), "query1");
        assert_eq!(state.search_history_index, Some(0));
    }

    #[test]
    fn test_recall_next_search() {
        let mut state = create_test_state();
        state.search_history.push("query1".to_string());
        state.search_history.push("query2".to_string());
        state.search_history_index = Some(0);
        state.search_input = Some("query1".to_string());

        state.recall_next_search();

        assert_eq!(state.search_input.as_ref().unwrap(), "query2");
        assert_eq!(state.search_history_index, Some(1));
    }

    #[test]
    fn test_next_search_match_no_results() {
        let mut state = create_test_state();

        state.next_search_match();

        // Should not panic
        assert!(state.search_state.is_none());
    }

    #[test]
    fn test_previous_search_match_no_results() {
        let mut state = create_test_state();

        state.previous_search_match();

        // Should not panic
        assert!(state.search_state.is_none());
    }

    #[test]
    fn test_has_search_results_after_search() {
        let mut state = create_test_state();
        setup_output(&mut state, vec!["ERROR: test", "INFO: ok"]);
        state.search_input = Some("ERROR".to_string());

        state.submit_search();

        assert!(state.has_search_results());
    }

    #[test]
    fn test_search_history_no_duplicates() {
        let mut state = create_test_state();
        setup_output(&mut state, vec!["ERROR: test"]);

        state.search_input = Some("ERROR".to_string());
        state.submit_search();

        state.search_input = Some("ERROR".to_string());
        state.submit_search();

        // Should only appear once
        assert_eq!(
            state
                .search_history
                .iter()
                .filter(|q| *q == "ERROR")
                .count(),
            1
        );
    }

    #[test]
    fn test_push_char_clears_history_index() {
        let mut state = create_test_state();
        state.search_history.push("old".to_string());
        state.search_input = Some("".to_string());
        state.search_history_index = Some(0);

        state.push_search_char('n');

        assert!(state.search_history_index.is_none());
    }

    #[test]
    fn test_backspace_clears_history_index() {
        let mut state = create_test_state();
        state.search_history.push("old".to_string());
        state.search_input = Some("test".to_string());
        state.search_history_index = Some(0);

        state.backspace_search_char();

        assert!(state.search_history_index.is_none());
    }
}
