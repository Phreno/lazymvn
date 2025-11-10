//! Log parser utilities
//!
//! This module provides utilities for parsing and normalizing log content:
//! - ANSI escape sequence stripping
//! - Log line normalization

/// Clean ANSI escape sequences from a log line
/// Returns None if the resulting line is empty
pub fn clean_log_line(raw: &str) -> Option<String> {
    strip_ansi_and_carriage_returns(raw)
        .and_then(to_non_empty_trimmed)
}

/// Strip ANSI escape sequences and carriage returns
fn strip_ansi_and_carriage_returns(raw: &str) -> Option<String> {
    let result = process_chars(raw);
    Some(result)
}

/// Process characters, removing ANSI sequences and carriage returns
fn process_chars(raw: &str) -> String {
    let mut result = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();

    while let Some(ch) = chars.next() {
        if is_ansi_escape_start(ch, chars.peek()) {
            consume_ansi_sequence(&mut chars);
        } else if !is_carriage_return(ch) {
            result.push(ch);
        }
    }

    result
}

/// Check if character is start of ANSI escape sequence
fn is_ansi_escape_start(ch: char, next: Option<&char>) -> bool {
    ch == '\u{1b}' && next == Some(&'[')
}

/// Check if character is carriage return
fn is_carriage_return(ch: char) -> bool {
    ch == '\r'
}

/// Consume ANSI escape sequence
fn consume_ansi_sequence<I>(chars: &mut std::iter::Peekable<I>)
where
    I: Iterator<Item = char>,
{
    chars.next(); // Skip the '['
    
    // Consume until we reach end of ANSI sequence
    for next in chars.by_ref() {
        if is_ansi_sequence_terminator(next) {
            break;
        }
    }
}

/// Check if character terminates ANSI sequence
fn is_ansi_sequence_terminator(ch: char) -> bool {
    ('@'..='~').contains(&ch)
}

/// Convert to trimmed string if non-empty
fn to_non_empty_trimmed(s: String) -> Option<String> {
    let trimmed = s.trim_end();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
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

    #[test]
    fn test_is_carriage_return() {
        assert!(is_carriage_return('\r'));
        assert!(!is_carriage_return('\n'));
        assert!(!is_carriage_return('a'));
    }

    #[test]
    fn test_is_ansi_escape_start() {
        assert!(is_ansi_escape_start('\u{1b}', Some(&'[')));
        assert!(!is_ansi_escape_start('\u{1b}', Some(&'a')));
        assert!(!is_ansi_escape_start('a', Some(&'[')));
        assert!(!is_ansi_escape_start('\u{1b}', None));
    }

    #[test]
    fn test_is_ansi_sequence_terminator() {
        assert!(is_ansi_sequence_terminator('m'));
        assert!(is_ansi_sequence_terminator('H'));
        assert!(is_ansi_sequence_terminator('J'));
        assert!(!is_ansi_sequence_terminator('1'));
        assert!(!is_ansi_sequence_terminator(';'));
    }

    #[test]
    fn test_to_non_empty_trimmed() {
        assert_eq!(to_non_empty_trimmed("hello  ".to_string()), Some("hello".to_string()));
        assert_eq!(to_non_empty_trimmed("  ".to_string()), None);
        assert_eq!(to_non_empty_trimmed("".to_string()), None);
    }
}
