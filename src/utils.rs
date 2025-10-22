use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

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
    use super::{clean_log_line, colorize_log_line, colorize_xml_line};

    #[test]
    fn strips_carriage_returns() {
        let cleaned = clean_log_line("line\r").unwrap();
        assert_eq!(cleaned, "line");
    }

    #[test]
    fn strips_ansi_sequences() {
        let cleaned = clean_log_line("\u{1b}[32mSUCCESS\u{1b}[0m build").unwrap();
        assert_eq!(cleaned, "SUCCESS build");
    }

    #[test]
    fn trims_trailing_space() {
        let cleaned = clean_log_line("line   ").unwrap();
        assert_eq!(cleaned, "line");
    }

    #[test]
    fn test_colorize_xml_opening_tag() {
        let line = colorize_xml_line("        <profile>");
        assert!(!line.spans.is_empty(), "Should have spans");
        // Should have multiple colored spans for tag parts
        assert!(
            line.spans.len() >= 3,
            "Should have at least 3 spans (bracket, name, bracket)"
        );
    }

    #[test]
    fn test_colorize_xml_closing_tag() {
        let line = colorize_xml_line("        </profile>");
        assert!(!line.spans.is_empty(), "Should have spans");
        // Closing tags should be styled differently
        assert!(line.spans.len() >= 3, "Should have spans for closing tag");
    }

    #[test]
    fn test_colorize_xml_with_attributes() {
        let line = colorize_xml_line("        <id>dev</id>");
        assert!(!line.spans.is_empty(), "Should have spans");
        // Should colorize tag names and content
    }

    #[test]
    fn test_colorize_xml_declaration() {
        let line = colorize_xml_line("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
        assert!(!line.spans.is_empty(), "Should have spans");
        // XML declarations should be styled in magenta
    }

    #[test]
    fn test_colorize_xml_comment() {
        let line = colorize_xml_line("        <!-- This is a comment -->");
        assert!(!line.spans.is_empty(), "Should have spans");
        // Comments should be styled in gray
    }

    #[test]
    fn colorize_log_line_highlights_info() {
        let line = colorize_log_line("[INFO] Building project...");
        assert_eq!(line.spans.len(), 2);
        assert_eq!(line.spans[0].content, "[INFO]");
        assert_eq!(line.spans[1].content, " Building project...");
        assert_eq!(line.spans[0].style.fg, Some(ratatui::style::Color::Green));
    }

    #[test]
    fn colorize_log_line_highlights_warning() {
        let line = colorize_log_line("Some text [WARNING] Warning message");
        assert_eq!(line.spans.len(), 3);
        assert_eq!(line.spans[0].content, "Some text ");
        assert_eq!(line.spans[1].content, "[WARNING]");
        assert_eq!(line.spans[2].content, " Warning message");
        assert_eq!(line.spans[1].style.fg, Some(ratatui::style::Color::Yellow));
    }

    #[test]
    fn colorize_log_line_highlights_error() {
        let line = colorize_log_line("[ERROR] Build failed");
        assert_eq!(line.spans.len(), 2);
        assert_eq!(line.spans[0].content, "[ERROR]");
        assert_eq!(line.spans[1].content, " Build failed");
        assert_eq!(line.spans[0].style.fg, Some(ratatui::style::Color::Red));
    }

    #[test]
    fn colorize_log_line_handles_plain_text() {
        let line = colorize_log_line("Plain text without keywords");
        assert_eq!(line.spans.len(), 1);
        assert_eq!(line.spans[0].content, "Plain text without keywords");
        assert_eq!(line.spans[0].style.fg, None);
    }
}

/// Get the current Git branch name for a project
/// Returns None if not a Git repository or if branch cannot be determined
pub fn get_git_branch(project_root: &std::path::Path) -> Option<String> {
    use std::process::Command;

    let output = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .arg("branch")
        .arg("--show-current")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let branch = String::from_utf8(output.stdout).ok()?;
    let branch = branch.trim();

    if branch.is_empty() {
        None
    } else {
        Some(branch.to_string())
    }
}
