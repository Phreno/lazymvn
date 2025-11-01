//! Log parser utilities
//!
//! This module provides utilities for parsing and normalizing log content:
//! - ANSI escape sequence stripping
//! - Log line normalization

/// Clean ANSI escape sequences from a log line
/// Returns None if the resulting line is empty
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_log_line_no_ansi() {
        let input = "[INFO] Simple log line";
        let result = clean_log_line(input);
        assert_eq!(result, Some("[INFO] Simple log line".to_string()));
    }

    #[test]
    fn test_clean_log_line_with_ansi() {
        let input = "\u{1b}[32m[INFO]\u{1b}[0m Simple log line";
        let result = clean_log_line(input);
        assert_eq!(result, Some("[INFO] Simple log line".to_string()));
    }

    #[test]
    fn test_clean_log_line_empty() {
        let input = "";
        let result = clean_log_line(input);
        assert_eq!(result, None);
    }

    #[test]
    fn test_clean_log_line_only_whitespace() {
        let input = "   \t\n  ";
        let result = clean_log_line(input);
        assert_eq!(result, None);
    }

    #[test]
    fn test_clean_log_line_carriage_return() {
        let input = "Line 1\r\nLine 2\r\n";
        let result = clean_log_line(input);
        assert_eq!(result, Some("Line 1\nLine 2".to_string()));
    }
}
