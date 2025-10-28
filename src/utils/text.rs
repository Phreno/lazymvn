//! Text processing utilities
//!
//! Provides functions for text formatting, colorization, and cleaning:
//! - ANSI escape sequence stripping
//! - Log line colorization
//! - XML syntax highlighting

use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

/// Clean a log line by removing ANSI escape sequences and carriage returns
/// Returns None if the line is empty after cleaning
pub fn clean_log_line(raw: &str) -> Option<String> {
    let mut result = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}'
            && let Some('[') = chars.peek()
        {
            chars.next();
            // Consume until we reach end of ANSI sequence
            for next in chars.by_ref() {
                if ('@'..='~').contains(&next) {
                    break;
                }
            }
            continue;
        }

        if ch != '\r' {
            result.push(ch);
        }
    }

    let trimmed = result.trim_end();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

/// Create a line with keyword-based coloring (simple approach)
/// Highlights [INFO], [WARNING], [ERROR], [DEBUG] keywords and command lines
pub fn colorize_log_line(text: &str) -> Line<'static> {
    let mut spans = Vec::new();

    // Check if this is a command line (starts with $)
    if text.starts_with("$ ") {
        spans.push(Span::styled(
            text.to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(ratatui::style::Modifier::BOLD),
        ));
    } else if let Some(debug_pos) = text.find("[DEBUG]") {
        // Split around [DEBUG]
        if debug_pos > 0 {
            spans.push(Span::raw(text[..debug_pos].to_string()));
        }
        spans.push(Span::styled(
            "[DEBUG]".to_string(),
            Style::default().fg(Color::Magenta),
        ));
        let remaining = &text[debug_pos + 7..];
        if !remaining.is_empty() {
            spans.push(Span::raw(remaining.to_string()));
        }
    } else if let Some(info_pos) = text.find("[INFO]") {
        // Split around [INFO]
        if info_pos > 0 {
            spans.push(Span::raw(text[..info_pos].to_string()));
        }
        spans.push(Span::styled(
            "[INFO]".to_string(),
            Style::default().fg(Color::Green),
        ));
        let remaining = &text[info_pos + 6..];
        if !remaining.is_empty() {
            spans.push(Span::raw(remaining.to_string()));
        }
    } else if let Some(warn_pos) = text.find("[WARNING]") {
        // Split around [WARNING]
        if warn_pos > 0 {
            spans.push(Span::raw(text[..warn_pos].to_string()));
        }
        spans.push(Span::styled(
            "[WARNING]".to_string(),
            Style::default().fg(Color::Yellow),
        ));
        let remaining = &text[warn_pos + 9..];
        if !remaining.is_empty() {
            spans.push(Span::raw(remaining.to_string()));
        }
    } else if let Some(error_pos) = text.find("[ERROR]").or_else(|| text.find("[ERR]")) {
        // Split around [ERROR] or [ERR]
        let (keyword, len) = if text[error_pos..].starts_with("[ERROR]") {
            ("[ERROR]", 7)
        } else {
            ("[ERR]", 5)
        };

        if error_pos > 0 {
            spans.push(Span::raw(text[..error_pos].to_string()));
        }
        spans.push(Span::styled(
            keyword.to_string(),
            Style::default().fg(Color::Red),
        ));
        let remaining = &text[error_pos + len..];
        if !remaining.is_empty() {
            spans.push(Span::raw(remaining.to_string()));
        }
    } else {
        // No special keywords, return as-is
        spans.push(Span::raw(text.to_string()));
    }

    Line::from(spans)
}

/// Colorize XML syntax for better readability
/// Highlights tags, attributes, and values with different colors
pub fn colorize_xml_line(text: &str) -> Line<'static> {
    let mut spans = Vec::new();
    let mut chars = text.chars().peekable();
    let mut current = String::new();

    while let Some(ch) = chars.next() {
        if ch == '<' {
            // Flush any accumulated text as normal
            if !current.is_empty() {
                spans.push(Span::raw(current.clone()));
                current.clear();
            }

            // Start of tag
            current.push(ch);

            // Check if it's a closing tag or comment
            let is_closing = chars.peek() == Some(&'/');
            let is_comment = chars.peek() == Some(&'!');

            // Consume the tag
            let mut in_quotes = false;
            let mut quote_char = ' ';

            while let Some(&next_ch) = chars.peek() {
                chars.next();
                current.push(next_ch);

                if next_ch == '"' || next_ch == '\'' {
                    if in_quotes && quote_char == next_ch {
                        in_quotes = false;
                    } else if !in_quotes {
                        in_quotes = true;
                        quote_char = next_ch;
                    }
                }

                if next_ch == '>' && !in_quotes {
                    break;
                }
            }

            // Colorize the tag
            if is_comment {
                // Comments in dark gray (more subtle)
                spans.push(Span::styled(
                    current.clone(),
                    Style::default().fg(Color::DarkGray),
                ));
            } else {
                // Parse tag name and attributes
                colorize_xml_tag(&current, &mut spans, is_closing);
            }
            current.clear();
        } else {
            current.push(ch);
        }
    }

    // Flush remaining text
    if !current.is_empty() {
        let trimmed = current.trim();
        if !trimmed.is_empty() {
            // Text content in white
            spans.push(Span::raw(current));
        } else {
            // Preserve whitespace
            spans.push(Span::raw(current));
        }
    }

    if spans.is_empty() {
        spans.push(Span::raw(text.to_string()));
    }

    Line::from(spans)
}

/// Helper to colorize XML tag components
fn colorize_xml_tag(tag: &str, spans: &mut Vec<Span<'static>>, is_closing: bool) {
    // Tag format: <tagname attr="value">
    let content = tag.trim_start_matches('<').trim_end_matches('>');

    if content.starts_with('?') {
        // XML declaration: <?xml ... ?> in light purple
        spans.push(Span::styled(
            tag.to_string(),
            Style::default().fg(Color::LightMagenta),
        ));
        return;
    }

    let mut parts = content.split_whitespace();

    // Tag name
    if let Some(tag_name) = parts.next() {
        // Opening bracket in dark gray
        spans.push(Span::styled(
            "<".to_string(),
            Style::default().fg(Color::DarkGray),
        ));

        // Tag name: light blue for opening, light red for closing
        let tag_color = if is_closing {
            Color::LightRed
        } else {
            Color::LightBlue
        };
        spans.push(Span::styled(
            tag_name.to_string(),
            Style::default().fg(tag_color),
        ));

        // Attributes
        let remainder: Vec<&str> = parts.collect();
        if !remainder.is_empty() {
            let attrs = remainder.join(" ");
            colorize_xml_attributes(&attrs, spans);
        }

        // Closing bracket in dark gray
        spans.push(Span::styled(
            ">".to_string(),
            Style::default().fg(Color::DarkGray),
        ));
    } else {
        // Fallback
        spans.push(Span::raw(tag.to_string()));
    }
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
    fn strips_ansi_sequences() {
        let input = "\u{1b}[31mRed\u{1b}[0m text";
        let result = clean_log_line(input);
        assert_eq!(result, Some("Red text".to_string()));
    }

    #[test]
    fn strips_carriage_returns() {
        let input = "Hello\rWorld";
        let result = clean_log_line(input);
        assert_eq!(result, Some("HelloWorld".to_string()));
    }

    #[test]
    fn trims_trailing_space() {
        let input = "Test line   ";
        let result = clean_log_line(input);
        assert_eq!(result, Some("Test line".to_string()));
    }

    #[test]
    fn colorize_log_line_handles_plain_text() {
        let line = colorize_log_line("Plain text");
        assert_eq!(line.spans.len(), 1);
    }

    #[test]
    fn colorize_log_line_highlights_info() {
        let line = colorize_log_line("This is [INFO] message");
        assert!(line.spans.len() >= 2);
    }

    #[test]
    fn colorize_log_line_highlights_debug() {
        let line = colorize_log_line("This is [DEBUG] message");
        assert!(line.spans.len() >= 2);
        // Check that [DEBUG] is colored in magenta
        assert!(line.spans.iter().any(|s| s.content == "[DEBUG]"));
    }

    #[test]
    fn colorize_log_line_highlights_warning() {
        let line = colorize_log_line("This is [WARNING] message");
        assert!(line.spans.len() >= 2);
    }

    #[test]
    fn colorize_log_line_highlights_error() {
        let line = colorize_log_line("This is [ERROR] message");
        assert!(line.spans.len() >= 2);
    }

    #[test]
    fn test_colorize_xml_declaration() {
        let line = colorize_xml_line("<?xml version=\"1.0\"?>");
        assert!(!line.spans.is_empty());
    }

    #[test]
    fn test_colorize_xml_opening_tag() {
        let line = colorize_xml_line("<project>");
        assert!(line.spans.len() >= 3); // <, project, >
    }

    #[test]
    fn test_colorize_xml_closing_tag() {
        let line = colorize_xml_line("</project>");
        assert!(line.spans.len() >= 3);
    }

    #[test]
    fn test_colorize_xml_with_attributes() {
        let line = colorize_xml_line("<project xmlns=\"http://maven.apache.org\">");
        assert!(line.spans.len() >= 4); // <, project, attrs, >
    }

    #[test]
    fn test_colorize_xml_comment() {
        let line = colorize_xml_line("<!-- This is a comment -->");
        assert_eq!(line.spans.len(), 1);
    }
}
