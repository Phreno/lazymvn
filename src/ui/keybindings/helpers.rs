//! Helper functions for keybinding display formatting
//!
//! Provides utilities for rendering styled key hints and bracketed text.

use crate::ui::theme::Theme;
use ratatui::text::Span;

/// Create a styled span for a key token (e.g., "Ctrl+T", "↑↓")
pub(crate) fn key_token(text: &str) -> Span<'static> {
    Span::styled(text.to_string(), Theme::KEY_HINT_STYLE)
}

/// Append a bracketed word to spans vector
///
/// Creates text in the format: `prefix[key]suffix`
/// For example: "pac[k]age" or "[b]uild"
///
/// # Arguments
/// * `spans` - Vector to append styled spans to
/// * `prefix` - Text before the bracketed key (can be empty)
/// * `key` - The key character to display in brackets
/// * `suffix` - Text after the bracketed key (can be empty)
pub(crate) fn append_bracketed_word(
    spans: &mut Vec<Span<'static>>,
    prefix: &str,
    key: &str,
    suffix: &str,
) {
    let key_style = Theme::KEY_HINT_STYLE;
    let text_style = Theme::DEFAULT_STYLE;

    if !prefix.is_empty() {
        spans.push(Span::styled(prefix.to_string(), text_style));
    }

    spans.push(Span::styled("[", text_style));
    spans.push(Span::styled(key.to_string(), key_style));
    spans.push(Span::styled("]", text_style));

    if !suffix.is_empty() {
        spans.push(Span::styled(suffix.to_string(), text_style));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_token_creates_styled_span() {
        let span = key_token("Ctrl+T");
        assert_eq!(span.content, "Ctrl+T");
        assert_eq!(span.style, Theme::KEY_HINT_STYLE);
    }

    #[test]
    fn test_append_bracketed_word_with_prefix_and_suffix() {
        let mut spans = Vec::new();
        append_bracketed_word(&mut spans, "pac", "k", "age");

        assert_eq!(spans.len(), 5);
        assert_eq!(spans[0].content, "pac");
        assert_eq!(spans[1].content, "[");
        assert_eq!(spans[2].content, "k");
        assert_eq!(spans[3].content, "]");
        assert_eq!(spans[4].content, "age");
    }

    #[test]
    fn test_append_bracketed_word_without_prefix() {
        let mut spans = Vec::new();
        append_bracketed_word(&mut spans, "", "b", "uild");

        assert_eq!(spans.len(), 4);
        assert_eq!(spans[0].content, "[");
        assert_eq!(spans[1].content, "b");
        assert_eq!(spans[2].content, "]");
        assert_eq!(spans[3].content, "uild");
    }

    #[test]
    fn test_append_bracketed_word_without_suffix() {
        let mut spans = Vec::new();
        append_bracketed_word(&mut spans, "test", "x", "");

        assert_eq!(spans.len(), 4);
        assert_eq!(spans[0].content, "test");
        assert_eq!(spans[1].content, "[");
        assert_eq!(spans[2].content, "x");
        assert_eq!(spans[3].content, "]");
    }

    #[test]
    fn test_append_bracketed_word_key_only() {
        let mut spans = Vec::new();
        append_bracketed_word(&mut spans, "", "q", "");

        assert_eq!(spans.len(), 3);
        assert_eq!(spans[0].content, "[");
        assert_eq!(spans[1].content, "q");
        assert_eq!(spans[2].content, "]");
    }
}
