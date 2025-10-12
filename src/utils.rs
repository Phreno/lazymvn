pub fn clean_log_line(raw: &str) -> String {
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

    result.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::clean_log_line;

    #[test]
    fn strips_carriage_returns() {
        let cleaned = clean_log_line("line\r");
        assert_eq!(cleaned, "line");
    }

    #[test]
    fn strips_ansi_sequences() {
        let cleaned = clean_log_line("\u{1b}[32mSUCCESS\u{1b}[0m build");
        assert_eq!(cleaned, "SUCCESS build");
    }

    #[test]
    fn trims_trailing_space() {
        let cleaned = clean_log_line("line   ");
        assert_eq!(cleaned, "line");
    }
}
