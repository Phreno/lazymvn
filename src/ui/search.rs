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
    command_output
        .iter()
        .enumerate()
        .flat_map(|(line_index, line)| find_matches_in_line(line, line_index, regex))
        .collect()
}

/// Find all matches in a single line
fn find_matches_in_line(line: &str, line_index: usize, regex: &Regex) -> Vec<SearchMatch> {
    let cleaned = crate::utils::clean_log_line(line).unwrap_or_default();
    regex
        .find_iter(&cleaned)
        .map(|mat| create_search_match(line_index, mat.start(), mat.end()))
        .collect()
}

/// Create a search match
fn create_search_match(line_index: usize, start: usize, end: usize) -> SearchMatch {
    SearchMatch {
        line_index,
        start,
        end,
    }
}

/// Generate styled spans for a line with search highlights
pub fn search_line_style(
    line_index: usize,
    search_state: &SearchState,
) -> Option<Vec<(Style, std::ops::Range<usize>)>> {
    let highlights = collect_highlights_for_line(line_index, search_state);
    
    if highlights.is_empty() {
        None
    } else {
        Some(sort_highlights(highlights))
    }
}

/// Collect all search highlights for a specific line
fn collect_highlights_for_line(
    line_index: usize,
    search_state: &SearchState,
) -> Vec<(Style, std::ops::Range<usize>)> {
    search_state
        .matches
        .iter()
        .enumerate()
        .filter(|(_, m)| m.line_index == line_index)
        .map(|(idx, m)| create_highlight(idx, m, search_state.current))
        .collect()
}

/// Create a highlight with appropriate style
fn create_highlight(
    match_idx: usize,
    search_match: &SearchMatch,
    current_match: usize,
) -> (Style, std::ops::Range<usize>) {
    let style = select_highlight_style(match_idx, current_match);
    (style, search_match.start..search_match.end)
}

/// Select highlight style based on whether it's the current match
fn select_highlight_style(match_idx: usize, current_match: usize) -> Style {
    if match_idx == current_match {
        Theme::CURRENT_SEARCH_MATCH_STYLE
    } else {
        Theme::SEARCH_MATCH_STYLE
    }
}

/// Sort highlights by start position
fn sort_highlights(
    mut highlights: Vec<(Style, std::ops::Range<usize>)>,
) -> Vec<(Style, std::ops::Range<usize>)> {
    highlights.sort_by_key(|(_, range)| range.start);
    highlights
}

/// Generate search status line for the footer
pub fn search_status_line(
    search_input: Option<&str>,
    search_error: Option<&str>,
    search_state: Option<&SearchState>,
) -> Option<Line<'static>> {
    if let Some(buffer) = search_input {
        return Some(format_search_input_line(buffer, search_state));
    }

    if let Some(error) = search_error {
        return Some(format_search_error_line(error));
    }

    search_state.map(format_search_result_line)
}

/// Format the search input line (during typing)
fn format_search_input_line(buffer: &str, search_state: Option<&SearchState>) -> Line<'static> {
    if buffer.is_empty() {
        return create_empty_search_prompt();
    }

    if let Some(search) = search_state {
        format_live_search_results(buffer, search)
    } else {
        create_typing_prompt(buffer)
    }
}

/// Create empty search prompt
fn create_empty_search_prompt() -> Line<'static> {
    Line::from(vec![
        Span::styled("/", Theme::INFO_STYLE),
        Span::raw("_ (type to search, Enter to confirm, Esc to cancel)"),
    ])
}

/// Format live search results
fn format_live_search_results(buffer: &str, search: &SearchState) -> Line<'static> {
    if search.has_matches() {
        let (current, total) = get_match_position(search);
        Line::from(vec![
            Span::styled("/", Theme::INFO_STYLE),
            Span::raw(format!(
                "{buffer}_ - {current}/{total} matches (Enter to confirm, Esc to cancel)"
            )),
        ])
    } else {
        Line::from(vec![
            Span::styled("/", Theme::INFO_STYLE),
            Span::raw(format!("{buffer}_ - ")),
            Span::styled("no matches", Theme::ERROR_STYLE),
            Span::raw(" (Enter to confirm, Esc to cancel)"),
        ])
    }
}

/// Create typing prompt
fn create_typing_prompt(buffer: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled("/", Theme::INFO_STYLE),
        Span::raw(format!(
            "{buffer}_ (type to search, Enter to confirm, Esc to cancel)"
        )),
    ])
}

/// Format search error line
fn format_search_error_line(error: &str) -> Line<'static> {
    Line::from(vec![
        Span::raw("Search error: "),
        Span::styled(error.to_string(), Theme::ERROR_STYLE),
        Span::raw(" (Esc to dismiss)"),
    ])
}

/// Format search result line (after confirming search)
fn format_search_result_line(search: &SearchState) -> Line<'static> {
    if search.has_matches() {
        let (current, total) = get_match_position(search);
        Line::from(vec![
            Span::styled("Search", Theme::INFO_STYLE),
            Span::raw(format!(
                " Match {current}/{total}   /{} (n/N/Enter to exit)",
                search.query
            )),
        ])
    } else {
        Line::from(vec![
            Span::styled("Search", Theme::INFO_STYLE),
            Span::raw(format!(" No matches   /{} (Enter to exit)", search.query)),
        ])
    }
}

/// Get current match position (1-indexed)
fn get_match_position(search: &SearchState) -> (usize, usize) {
    (search.current + 1, search.total_matches())
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

    #[test]
    fn test_create_search_match() {
        let mat = create_search_match(5, 10, 15);
        assert_eq!(mat.line_index, 5);
        assert_eq!(mat.start, 10);
        assert_eq!(mat.end, 15);
    }

    #[test]
    fn test_select_highlight_style_current() {
        let style = select_highlight_style(2, 2);
        assert_eq!(style, Theme::CURRENT_SEARCH_MATCH_STYLE);
    }

    #[test]
    fn test_select_highlight_style_not_current() {
        let style = select_highlight_style(1, 2);
        assert_eq!(style, Theme::SEARCH_MATCH_STYLE);
    }

    #[test]
    fn test_get_match_position() {
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
        let mut state = SearchState::new("test".to_string(), matches);
        
        let (current, total) = get_match_position(&state);
        assert_eq!(current, 1);
        assert_eq!(total, 2);
        
        state.next_match();
        let (current, total) = get_match_position(&state);
        assert_eq!(current, 2);
        assert_eq!(total, 2);
    }

    #[test]
    fn test_search_status_line_empty_input() {
        let line = search_status_line(Some(""), None, None);
        assert!(line.is_some());
        let line = line.unwrap();
        assert!(line.to_string().contains("type to search"));
    }

    #[test]
    fn test_search_status_line_with_results() {
        let matches = vec![SearchMatch {
            line_index: 0,
            start: 0,
            end: 4,
        }];
        let state = SearchState::new("test".to_string(), matches);
        
        let line = search_status_line(Some("test"), None, Some(&state));
        assert!(line.is_some());
        let line = line.unwrap();
        assert!(line.to_string().contains("1/1"));
    }

    #[test]
    fn test_search_status_line_no_matches() {
        let state = SearchState::new("test".to_string(), vec![]);
        
        let line = search_status_line(Some("test"), None, Some(&state));
        assert!(line.is_some());
        let line = line.unwrap();
        assert!(line.to_string().contains("no matches"));
    }

    #[test]
    fn test_search_status_line_error() {
        let line = search_status_line(None, Some("Invalid regex"), None);
        assert!(line.is_some());
        let line = line.unwrap();
        assert!(line.to_string().contains("Invalid regex"));
    }

    #[test]
    fn test_search_status_line_confirmed_search() {
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
        
        let line = search_status_line(None, None, Some(&state));
        assert!(line.is_some());
        let line = line.unwrap();
        assert!(line.to_string().contains("1/2"));
        assert!(line.to_string().contains("/test"));
    }
}
