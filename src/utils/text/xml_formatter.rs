use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

pub fn colorize_xml_line(text: &str) -> Line<'static> {
    let mut spans = Vec::new();
    let mut chars = text.chars().peekable();
    let mut current = String::new();

    while let Some(ch) = chars.next() {
        if ch == '<' {
            flush_current_text(&mut spans, &mut current);
            process_xml_tag(&mut chars, &mut spans, &mut current, ch);
        } else {
            current.push(ch);
        }
    }

    flush_final_text(&mut spans, current, text);
    Line::from(spans)
}

/// Flush current accumulated text as a span
fn flush_current_text(spans: &mut Vec<Span<'static>>, current: &mut String) {
    if !current.is_empty() {
        spans.push(Span::raw(current.clone()));
        current.clear();
    }
}

/// Process an XML tag
fn process_xml_tag(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    spans: &mut Vec<Span<'static>>,
    current: &mut String,
    ch: char,
) {
    current.push(ch);
    
    let is_closing = chars.peek() == Some(&'/');
    let is_comment = chars.peek() == Some(&'!');
    
    consume_tag_content(chars, current);
    
    if is_comment {
        add_comment_span(spans, current);
    } else {
        colorize_xml_tag(current, spans, is_closing);
    }
    
    current.clear();
}

/// Consume tag content until closing bracket
fn consume_tag_content(chars: &mut std::iter::Peekable<std::str::Chars>, current: &mut String) {
    let mut in_quotes = false;
    let mut quote_char = ' ';

    while let Some(&next_ch) = chars.peek() {
        chars.next();
        current.push(next_ch);

        if should_toggle_quotes(next_ch, in_quotes, quote_char) {
            (in_quotes, quote_char) = toggle_quote_state(next_ch, in_quotes, quote_char);
        }

        if next_ch == '>' && !in_quotes {
            break;
        }
    }
}

/// Check if quote state should toggle
fn should_toggle_quotes(ch: char, _in_quotes: bool, _quote_char: char) -> bool {
    ch == '"' || ch == '\''
}

/// Toggle quote state
fn toggle_quote_state(ch: char, in_quotes: bool, quote_char: char) -> (bool, char) {
    if in_quotes && quote_char == ch {
        (false, ' ')
    } else if !in_quotes {
        (true, ch)
    } else {
        (in_quotes, quote_char)
    }
}

/// Add comment span with dark gray color
fn add_comment_span(spans: &mut Vec<Span<'static>>, current: &str) {
    spans.push(Span::styled(
        current.to_string(),
        Style::default().fg(Color::DarkGray),
    ));
}

/// Flush remaining text
fn flush_final_text(spans: &mut Vec<Span<'static>>, current: String, fallback: &str) {
    if !current.is_empty() {
        spans.push(Span::raw(current));
    }

    if spans.is_empty() {
        spans.push(Span::raw(fallback.to_string()));
    }
}

/// Helper to colorize XML tag components
fn colorize_xml_tag(tag: &str, spans: &mut Vec<Span<'static>>, is_closing: bool) {
    let content = strip_tag_brackets(tag);

    if is_xml_declaration(content) {
        add_xml_declaration_span(spans, tag);
        return;
    }

    let mut parts = content.split_whitespace();

    if let Some(tag_name) = parts.next() {
        add_tag_components(spans, tag_name, is_closing, &mut parts);
    } else {
        spans.push(Span::raw(tag.to_string()));
    }
}

/// Strip angle brackets from tag
fn strip_tag_brackets(tag: &str) -> &str {
    tag.trim_start_matches('<').trim_end_matches('>')
}

/// Check if content is XML declaration
fn is_xml_declaration(content: &str) -> bool {
    content.starts_with('?')
}

/// Add XML declaration span
fn add_xml_declaration_span(spans: &mut Vec<Span<'static>>, tag: &str) {
    spans.push(Span::styled(
        tag.to_string(),
        Style::default().fg(Color::LightMagenta),
    ));
}

/// Add tag components (brackets, name, attributes)
fn add_tag_components(
    spans: &mut Vec<Span<'static>>,
    tag_name: &str,
    is_closing: bool,
    parts: &mut std::str::SplitWhitespace,
) {
    add_opening_bracket(spans);
    add_tag_name(spans, tag_name, is_closing);
    add_tag_attributes(spans, parts);
    add_closing_bracket(spans);
}

/// Add opening bracket
fn add_opening_bracket(spans: &mut Vec<Span<'static>>) {
    spans.push(Span::styled(
        "<".to_string(),
        Style::default().fg(Color::DarkGray),
    ));
}

/// Add tag name with appropriate color
fn add_tag_name(spans: &mut Vec<Span<'static>>, tag_name: &str, is_closing: bool) {
    let tag_color = if is_closing {
        Color::LightRed
    } else {
        Color::LightBlue
    };
    spans.push(Span::styled(
        tag_name.to_string(),
        Style::default().fg(tag_color),
    ));
}

/// Add tag attributes if present
fn add_tag_attributes(spans: &mut Vec<Span<'static>>, parts: &mut std::str::SplitWhitespace) {
    let remainder: Vec<&str> = parts.collect();
    if !remainder.is_empty() {
        let attrs = remainder.join(" ");
        colorize_xml_attributes(&attrs, spans);
    }
}

/// Add closing bracket
fn add_closing_bracket(spans: &mut Vec<Span<'static>>) {
    spans.push(Span::styled(
        ">".to_string(),
        Style::default().fg(Color::DarkGray),
    ));
}

/// Helper to colorize XML attributes
fn colorize_xml_attributes(attrs: &str, spans: &mut Vec<Span<'static>>) {
    let chars = attrs.chars();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut in_value = false;

    for ch in chars {
        if ch == '=' && !in_quotes {
            // Attribute name in light yellow
            if !current.trim().is_empty() {
                spans.push(Span::raw(" ".to_string()));
                spans.push(Span::styled(
                    current.trim().to_string(),
                    Style::default().fg(Color::LightYellow),
                ));
                current.clear();
            }
            spans.push(Span::styled(
                "=".to_string(),
                Style::default().fg(Color::DarkGray),
            ));
            in_value = true;
        } else if ch == '"' || ch == '\'' {
            if !in_quotes {
                // Start of value
                in_quotes = true;
                current.push(ch);
            } else {
                // End of value - in light green
                current.push(ch);
                spans.push(Span::styled(
                    current.clone(),
                    Style::default().fg(Color::LightGreen),
                ));
                current.clear();
                in_quotes = false;
                in_value = false;
            }
        } else if ch.is_whitespace() && !in_quotes && !in_value {
            if !current.is_empty() {
                spans.push(Span::raw(current.clone()));
                current.clear();
            }
        } else {
            current.push(ch);
        }
    }

    if !current.is_empty() {
        spans.push(Span::raw(current));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colorize_plain_text() {
        let line = colorize_xml_line("plain text");
        assert_eq!(line.spans.len(), 1);
    }

    #[test]
    fn test_colorize_simple_tag() {
        let line = colorize_xml_line("<tag>");
        assert!(!line.spans.is_empty());
        // Should have bracket, tag name, bracket
        assert!(line.spans.len() >= 3);
    }

    #[test]
    fn test_colorize_closing_tag() {
        let line = colorize_xml_line("</tag>");
        assert!(!line.spans.is_empty());
        assert!(line.spans.len() >= 3);
    }

    #[test]
    fn test_colorize_tag_with_attributes() {
        let line = colorize_xml_line("<tag attr=\"value\">");
        assert!(!line.spans.is_empty());
        // Should have multiple spans for tag, attr, value
        assert!(line.spans.len() >= 5);
    }

    #[test]
    fn test_colorize_xml_declaration() {
        let line = colorize_xml_line("<?xml version=\"1.0\"?>");
        assert!(!line.spans.is_empty());
        // XML declarations are styled specially
        assert_eq!(line.spans.len(), 1);
    }

    #[test]
    fn test_colorize_xml_comment() {
        let line = colorize_xml_line("<!-- comment -->");
        assert!(!line.spans.is_empty());
        // Comments should be styled as a single span
        assert_eq!(line.spans.len(), 1);
    }

    #[test]
    fn test_colorize_tag_with_text() {
        let line = colorize_xml_line("<tag>text content</tag>");
        assert!(!line.spans.is_empty());
        // Should have opening tag, text, closing tag
        assert!(line.spans.len() >= 5);
    }

    #[test]
    fn test_colorize_multiple_attributes() {
        let line = colorize_xml_line("<tag attr1=\"val1\" attr2=\"val2\">");
        assert!(!line.spans.is_empty());
        // Multiple attributes create multiple spans
        assert!(line.spans.len() >= 7);
    }

    #[test]
    fn test_colorize_self_closing_tag() {
        let line = colorize_xml_line("<tag />");
        assert!(!line.spans.is_empty());
    }

    #[test]
    fn test_colorize_nested_quotes() {
        let line = colorize_xml_line("<tag attr=\"value with 'quotes'\">");
        assert!(!line.spans.is_empty());
        // Should handle nested quotes correctly
        assert!(line.spans.len() >= 4);
    }

    #[test]
    fn test_colorize_empty_string() {
        let line = colorize_xml_line("");
        assert_eq!(line.spans.len(), 1);
    }

    #[test]
    fn test_colorize_whitespace_only() {
        let line = colorize_xml_line("   ");
        assert_eq!(line.spans.len(), 1);
    }

    #[test]
    fn test_colorize_tag_with_single_quotes() {
        let line = colorize_xml_line("<tag attr='value'>");
        assert!(!line.spans.is_empty());
        assert!(line.spans.len() >= 4);
    }

    #[test]
    fn test_colorize_incomplete_tag() {
        let line = colorize_xml_line("<tag");
        assert!(!line.spans.is_empty());
    }

    #[test]
    fn test_colorize_text_with_multiple_tags() {
        let line = colorize_xml_line("<a>text</a><b>more</b>");
        assert!(!line.spans.is_empty());
        // Multiple tags should create many spans
        assert!(line.spans.len() >= 8);
    }
}
