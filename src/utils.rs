use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

pub fn clean_log_line(raw: &str) -> Option<String> {
    let mut result = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if let Some('[') = chars.peek() {
                chars.next();
                // Consume until we reach end of ANSI sequence
                while let Some(next) = chars.next() {
                    if ('@'..='~').contains(&next) {
                        break;
                    }
                }
                continue;
            }
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

    if let Some(info_pos) = text.find("[INFO]") {
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

#[cfg(test)]
mod tests {
    use super::{clean_log_line, colorize_log_line};

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
