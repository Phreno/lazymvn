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
                    Span::raw(format!("{buffer}_ - {current}/{total} matches (Enter to confirm, Esc to cancel)")),
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
                Span::raw(format!("{buffer}_ (type to search, Enter to confirm, Esc to cancel)")),
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
                Span::raw(format!(" Match {current}/{total}   /{} (n/N/Enter to exit)", search.query)),
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