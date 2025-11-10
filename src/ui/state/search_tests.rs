//! Tests for search functionality

#[cfg(test)]
mod tests {
    use super::super::TuiState;
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
