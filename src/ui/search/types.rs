/// Represents a search match within the output
#[derive(Clone, Debug)]
pub struct SearchMatch {
    pub line_index: usize,
    pub start: usize,
    pub end: usize,
}

/// State for managing search functionality
#[derive(Clone, Debug)]
pub struct SearchState {
    pub query: String,
    pub matches: Vec<SearchMatch>,
    pub current: usize,
}

impl SearchState {
    pub fn new(query: String, matches: Vec<SearchMatch>) -> Self {
        Self {
            query,
            matches,
            current: 0,
        }
    }

    pub fn has_matches(&self) -> bool {
        !self.matches.is_empty()
    }

    pub fn current_match(&self) -> Option<&SearchMatch> {
        self.matches.get(self.current)
    }

    pub fn total_matches(&self) -> usize {
        self.matches.len()
    }

    pub fn next_match(&mut self) {
        if !self.matches.is_empty() {
            self.current = (self.current + 1) % self.matches.len();
        }
    }

    pub fn previous_match(&mut self) {
        if !self.matches.is_empty() {
            self.current = if self.current == 0 {
                self.matches.len() - 1
            } else {
                self.current - 1
            };
        }
    }

    pub fn jump_to_match(&mut self, index: usize) {
        if index < self.matches.len() {
            self.current = index;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_state_new() {
        let matches = vec![
            SearchMatch {
                line_index: 0,
                start: 5,
                end: 10,
            },
            SearchMatch {
                line_index: 1,
                start: 15,
                end: 20,
            },
        ];
        let state = SearchState::new("test".to_string(), matches.clone());
        assert_eq!(state.query, "test");
        assert_eq!(state.matches.len(), 2);
        assert_eq!(state.current, 0);
    }

    #[test]
    fn test_search_state_has_matches() {
        let state_with_matches = SearchState::new(
            "test".to_string(),
            vec![SearchMatch {
                line_index: 0,
                start: 0,
                end: 4,
            }],
        );
        assert!(state_with_matches.has_matches());

        let state_without_matches = SearchState::new("test".to_string(), vec![]);
        assert!(!state_without_matches.has_matches());
    }

    #[test]
    fn test_search_state_current_match() {
        let matches = vec![
            SearchMatch {
                line_index: 0,
                start: 0,
                end: 4,
            },
            SearchMatch {
                line_index: 1,
                start: 5,
                end: 9,
            },
        ];
        let state = SearchState::new("test".to_string(), matches);
        let current = state.current_match().unwrap();
        assert_eq!(current.line_index, 0);
        assert_eq!(current.start, 0);
        assert_eq!(current.end, 4);
    }

    #[test]
    fn test_search_state_total_matches() {
        let matches = vec![
            SearchMatch {
                line_index: 0,
                start: 0,
                end: 4,
            },
            SearchMatch {
                line_index: 1,
                start: 5,
                end: 9,
            },
            SearchMatch {
                line_index: 2,
                start: 10,
                end: 14,
            },
        ];
        let state = SearchState::new("test".to_string(), matches);
        assert_eq!(state.total_matches(), 3);
    }

    #[test]
    fn test_search_state_next_match() {
        let matches = vec![
            SearchMatch {
                line_index: 0,
                start: 0,
                end: 4,
            },
            SearchMatch {
                line_index: 1,
                start: 5,
                end: 9,
            },
            SearchMatch {
                line_index: 2,
                start: 10,
                end: 14,
            },
        ];
        let mut state = SearchState::new("test".to_string(), matches);

        assert_eq!(state.current, 0);
        state.next_match();
        assert_eq!(state.current, 1);
        state.next_match();
        assert_eq!(state.current, 2);
        state.next_match();
        assert_eq!(state.current, 0);
    }

    #[test]
    fn test_search_state_previous_match() {
        let matches = vec![
            SearchMatch {
                line_index: 0,
                start: 0,
                end: 4,
            },
            SearchMatch {
                line_index: 1,
                start: 5,
                end: 9,
            },
            SearchMatch {
                line_index: 2,
                start: 10,
                end: 14,
            },
        ];
        let mut state = SearchState::new("test".to_string(), matches);

        assert_eq!(state.current, 0);
        state.previous_match();
        assert_eq!(state.current, 2);
        state.previous_match();
        assert_eq!(state.current, 1);
        state.previous_match();
        assert_eq!(state.current, 0);
    }

    #[test]
    fn test_search_state_jump_to_match() {
        let matches = vec![
            SearchMatch {
                line_index: 0,
                start: 0,
                end: 4,
            },
            SearchMatch {
                line_index: 1,
                start: 5,
                end: 9,
            },
            SearchMatch {
                line_index: 2,
                start: 10,
                end: 14,
            },
        ];
        let mut state = SearchState::new("test".to_string(), matches);

        state.jump_to_match(2);
        assert_eq!(state.current, 2);

        state.jump_to_match(0);
        assert_eq!(state.current, 0);

        state.jump_to_match(10);
        assert_eq!(state.current, 0);
    }

    #[test]
    fn test_search_state_next_match_empty() {
        let mut state = SearchState::new("test".to_string(), vec![]);
        state.next_match();
        assert_eq!(state.current, 0);
    }

    #[test]
    fn test_search_state_previous_match_empty() {
        let mut state = SearchState::new("test".to_string(), vec![]);
        state.previous_match();
        assert_eq!(state.current, 0);
    }
}
