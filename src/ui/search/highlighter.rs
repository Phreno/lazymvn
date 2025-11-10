use super::types::{SearchMatch, SearchState};
use crate::ui::theme::Theme;
use ratatui::style::Style;

pub fn search_line_style(
    line_index: usize,
    search_state: &SearchState,
) -> Option<Vec<(Style, std::ops::Range<usize>)>> {
    let highlights = collect_highlights_for_line(line_index, search_state);
    if highlights.is_empty() { None } else { Some(sort_highlights(highlights)) }
}

fn collect_highlights_for_line(
    line_index: usize,
    search_state: &SearchState,
) -> Vec<(Style, std::ops::Range<usize>)> {
    search_state.matches.iter().enumerate()
        .filter(|(_, m)| m.line_index == line_index)
        .map(|(idx, m)| create_highlight(idx, m, search_state.current))
        .collect()
}

fn create_highlight(match_idx: usize, search_match: &SearchMatch, current_match: usize) -> (Style, std::ops::Range<usize>) {
    let style = select_highlight_style(match_idx, current_match);
    (style, search_match.start..search_match.end)
}

fn select_highlight_style(match_idx: usize, current_match: usize) -> Style {
    if match_idx == current_match {
        Theme::CURRENT_SEARCH_MATCH_STYLE
    } else {
        Theme::SEARCH_MATCH_STYLE
    }
}

fn sort_highlights(mut highlights: Vec<(Style, std::ops::Range<usize>)>) -> Vec<(Style, std::ops::Range<usize>)> {
    highlights.sort_by_key(|(_, range)| range.start);
    highlights
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_line_style_with_matches() {
        let matches = vec![
            SearchMatch { line_index: 0, start: 5, end: 9 },
            SearchMatch { line_index: 0, start: 15, end: 19 },
            SearchMatch { line_index: 1, start: 0, end: 4 },
        ];
        let state = SearchState::new("test".to_string(), matches);
        let highlights = search_line_style(0, &state);
        assert!(highlights.is_some());
        let highlights = highlights.unwrap();
        assert_eq!(highlights.len(), 2);
        assert_eq!(highlights[0].1.start, 5);
        assert_eq!(highlights[1].1.start, 15);
    }

    #[test]
    fn test_search_line_style_no_matches_on_line() {
        let matches = vec![SearchMatch { line_index: 1, start: 0, end: 4 }];
        let state = SearchState::new("test".to_string(), matches);
        assert!(search_line_style(0, &state).is_none());
    }

    #[test]
    fn test_select_highlight_style_current() {
        assert_eq!(select_highlight_style(2, 2), Theme::CURRENT_SEARCH_MATCH_STYLE);
    }

    #[test]
    fn test_select_highlight_style_not_current() {
        assert_eq!(select_highlight_style(1, 2), Theme::SEARCH_MATCH_STYLE);
    }
}
