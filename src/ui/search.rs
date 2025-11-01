use crate::ui::theme::Theme;
use ratatui::{
    style::Style,
    text::{Line, Span},
};
use regex::Regex;

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

/// Collect search matches from command output using a regex
pub fn collect_search_matches(command_output: &[String], regex: &Regex) -> Vec<SearchMatch> {
    let mut matches = Vec::new();
    for (line_index, line) in command_output.iter().enumerate() {
        let cleaned = crate::utils::clean_log_line(line).unwrap_or_default();
        for mat in regex.find_iter(&cleaned) {
            matches.push(SearchMatch {
                line_index,
                start: mat.start(),
                end: mat.end(),
            });
        }
    }
    matches
}

/// Generate styled spans for a line with search highlights
pub fn search_line_style(
    line_index: usize,
    search_state: &SearchState,
) -> Option<Vec<(Style, std::ops::Range<usize>)>> {
    let mut highlights = Vec::new();

    for (match_idx, search_match) in search_state.matches.iter().enumerate() {
        if search_match.line_index == line_index {
            let style = if match_idx == search_state.current {
                Theme::CURRENT_SEARCH_MATCH_STYLE
            } else {
                Theme::SEARCH_MATCH_STYLE
            };
            highlights.push((style, search_match.start..search_match.end));
        }
    }

    if highlights.is_empty() {
        None
    } else {
        // Sort highlights by start position
        highlights.sort_by_key(|(_, range)| range.start);
        Some(highlights)
    }
}

/// Generate search status line for the footer
pub fn search_status_line(
    search_input: Option<&str>,
    search_error: Option<&str>,
    search_state: Option<&SearchState>,
) -> Option<Line<'static>> {
    if let Some(buffer) = search_input {
        // Show live search feedback during input
        if buffer.is_empty() {
            return Some(Line::from(vec![
                Span::styled("/", Theme::INFO_STYLE),
                Span::raw("_ (type to search, Enter to confirm, Esc to cancel)"),
            ]));
        }

        // Show live search results
        if let Some(search) = search_state {
            if search.has_matches() {
                let current = search.current + 1;
                let total = search.total_matches();
                return Some(Line::from(vec![
                    Span::styled("/", Theme::INFO_STYLE),
                    Span::raw(format!(
                        "{buffer}_ - {current}/{total} matches (Enter to confirm, Esc to cancel)"
                    )),
                ]));
            } else {
                return Some(Line::from(vec![
                    Span::styled("/", Theme::INFO_STYLE),
                    Span::raw(format!("{buffer}_ - ")),
                    Span::styled("no matches", Theme::ERROR_STYLE),
                    Span::raw(" (Enter to confirm, Esc to cancel)"),
                ]));
            }
        } else {
            return Some(Line::from(vec![
                Span::styled("/", Theme::INFO_STYLE),
                Span::raw(format!(
                    "{buffer}_ (type to search, Enter to confirm, Esc to cancel)"
                )),
            ]));
        }
    }

    if let Some(error) = search_error {
        return Some(Line::from(vec![
            Span::raw("Search error: "),
            Span::styled(error.to_string(), Theme::ERROR_STYLE),
            Span::raw(" (Esc to dismiss)"),
        ]));
    }

    if let Some(search) = search_state {
        if search.has_matches() {
            let current = search.current + 1;
            let total = search.total_matches();
            return Some(Line::from(vec![
                Span::styled("Search", Theme::INFO_STYLE),
                Span::raw(format!(
                    " Match {current}/{total}   /{} (n/N/Enter to exit)",
                    search.query
                )),
            ]));
        } else {
            return Some(Line::from(vec![
                Span::styled("Search", Theme::INFO_STYLE),
                Span::raw(format!(" No matches   /{} (Enter to exit)", search.query)),
            ]));
        }
    }

    None
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
        // Should wrap around to 0
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
        // Should wrap to last match
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

        // Out of bounds should not change current
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

    #[test]
    fn test_collect_search_matches_basic() {
        let output = vec![
            "This is a test line".to_string(),
            "Another test here".to_string(),
            "No match on this line".to_string(),
        ];
        let regex = Regex::new("test").unwrap();
        let matches = collect_search_matches(&output, &regex);

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].line_index, 0);
        assert_eq!(matches[1].line_index, 1);
    }

    #[test]
    fn test_collect_search_matches_multiple_per_line() {
        let output = vec!["test test test".to_string()];
        let regex = Regex::new("test").unwrap();
        let matches = collect_search_matches(&output, &regex);

        assert_eq!(matches.len(), 3);
        assert_eq!(matches[0].line_index, 0);
        assert_eq!(matches[1].line_index, 0);
        assert_eq!(matches[2].line_index, 0);
    }

    #[test]
    fn test_collect_search_matches_case_sensitive() {
        let output = vec!["Test test TEST".to_string()];
        let regex = Regex::new("test").unwrap();
        let matches = collect_search_matches(&output, &regex);

        // Should only match lowercase "test"
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_collect_search_matches_case_insensitive() {
        let output = vec!["Test test TEST".to_string()];
        let regex = Regex::new("(?i)test").unwrap();
        let matches = collect_search_matches(&output, &regex);

        // Should match all three
        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn test_collect_search_matches_empty_output() {
        let output: Vec<String> = vec![];
        let regex = Regex::new("test").unwrap();
        let matches = collect_search_matches(&output, &regex);

        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_collect_search_matches_no_matches() {
        let output = vec!["No match here".to_string(), "Still no match".to_string()];
        let regex = Regex::new("test").unwrap();
        let matches = collect_search_matches(&output, &regex);

        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_search_line_style_with_matches() {
        let matches = vec![
            SearchMatch {
                line_index: 0,
                start: 5,
                end: 9,
            },
            SearchMatch {
                line_index: 0,
                start: 15,
                end: 19,
            },
            SearchMatch {
                line_index: 1,
                start: 0,
                end: 4,
            },
        ];
        let state = SearchState::new("test".to_string(), matches);

        let highlights = search_line_style(0, &state);
        assert!(highlights.is_some());
        let highlights = highlights.unwrap();
        assert_eq!(highlights.len(), 2);
        assert_eq!(highlights[0].1.start, 5);
        assert_eq!(highlights[0].1.end, 9);
        assert_eq!(highlights[1].1.start, 15);
        assert_eq!(highlights[1].1.end, 19);
    }

    #[test]
    fn test_search_line_style_no_matches_on_line() {
        let matches = vec![SearchMatch {
            line_index: 1,
            start: 0,
            end: 4,
        }];
        let state = SearchState::new("test".to_string(), matches);

        let highlights = search_line_style(0, &state);
        assert!(highlights.is_none());
    }

    #[test]
    fn test_search_line_style_highlights_current_match() {
        let matches = vec![
            SearchMatch {
                line_index: 0,
                start: 0,
                end: 4,
            },
            SearchMatch {
                line_index: 0,
                start: 10,
                end: 14,
            },
        ];
        let mut state = SearchState::new("test".to_string(), matches);
        state.current = 1; // Set second match as current

        let highlights = search_line_style(0, &state);
        assert!(highlights.is_some());
        let highlights = highlights.unwrap();
        assert_eq!(highlights.len(), 2);
        // Both matches on same line, second should be styled as current
        assert_eq!(highlights[1].0, Theme::CURRENT_SEARCH_MATCH_STYLE);
    }
}
