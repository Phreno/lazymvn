use super::types::SearchState;
use crate::ui::theme::Theme;
use ratatui::text::{Line, Span};

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

fn create_empty_search_prompt() -> Line<'static> {
    Line::from(vec![
        Span::styled("/", Theme::INFO_STYLE),
        Span::raw("_ (type to search, Enter to confirm, Esc to cancel)"),
    ])
}

fn format_live_search_results(buffer: &str, search: &SearchState) -> Line<'static> {
    if search.has_matches() {
        let (current, total) = get_match_position(search);
        Line::from(vec![
            Span::styled("/", Theme::INFO_STYLE),
            Span::raw(format!("{buffer}_ - {current}/{total} matches (Enter to confirm, Esc to cancel)")),
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

fn create_typing_prompt(buffer: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled("/", Theme::INFO_STYLE),
        Span::raw(format!("{buffer}_ (type to search, Enter to confirm, Esc to cancel)")),
    ])
}

fn format_search_error_line(error: &str) -> Line<'static> {
    Line::from(vec![
        Span::raw("Search error: "),
        Span::styled(error.to_string(), Theme::ERROR_STYLE),
        Span::raw(" (Esc to dismiss)"),
    ])
}

fn format_search_result_line(search: &SearchState) -> Line<'static> {
    if search.has_matches() {
        let (current, total) = get_match_position(search);
        Line::from(vec![
            Span::styled("Search", Theme::INFO_STYLE),
            Span::raw(format!(" Match {current}/{total}   /{} (n/N/Enter to exit)", search.query)),
        ])
    } else {
        Line::from(vec![
            Span::styled("Search", Theme::INFO_STYLE),
            Span::raw(format!(" No matches   /{} (Enter to exit)", search.query)),
        ])
    }
}

fn get_match_position(search: &SearchState) -> (usize, usize) {
    (search.current + 1, search.total_matches())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::search::types::SearchMatch;

    #[test]
    fn test_get_match_position() {
        let matches = vec![
            SearchMatch { line_index: 0, start: 0, end: 4 },
            SearchMatch { line_index: 1, start: 5, end: 9 },
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
        assert!(line.unwrap().to_string().contains("type to search"));
    }

    #[test]
    fn test_search_status_line_with_results() {
        let matches = vec![SearchMatch { line_index: 0, start: 0, end: 4 }];
        let state = SearchState::new("test".to_string(), matches);
        let line = search_status_line(Some("test"), None, Some(&state));
        assert!(line.is_some());
        assert!(line.unwrap().to_string().contains("1/1"));
    }

    #[test]
    fn test_search_status_line_error() {
        let line = search_status_line(None, Some("Invalid regex"), None);
        assert!(line.is_some());
        assert!(line.unwrap().to_string().contains("Invalid regex"));
    }
}
