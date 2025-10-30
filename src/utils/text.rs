//! Text processing utilities
//!
//! Provides functions for text formatting, colorization, and cleaning:
//! - ANSI escape sequence stripping
//! - Log line colorization
//! - XML syntax highlighting
//! - Package name extraction from logs

use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

/// Regex pattern for detecting Java package names
/// Matches packages starting with common TLDs (com, org, net, io, fr, etc.) 
/// or well-known Java namespaces (java, javax, sun, spring, etc.)
/// followed by at least one more segment: word.word(.word)*
static PACKAGE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\b(?:com|org|net|io|fr|de|uk|nl|eu|gov|edu|mil|int|co|me|info|biz|mobi|name|pro|aero|asia|cat|coop|jobs|museum|tel|travel|xxx|ac|ad|ae|af|ag|ai|al|am|ao|aq|ar|as|at|au|aw|ax|az|ba|bb|bd|be|bf|bg|bh|bi|bj|bm|bn|bo|br|bs|bt|bv|bw|by|bz|ca|cc|cd|cf|cg|ch|ci|ck|cl|cm|cn|cr|cu|cv|cw|cx|cy|cz|dj|dk|dm|do|dz|ec|ee|eg|er|es|et|fi|fj|fk|fm|fo|ga|gb|gd|ge|gf|gg|gh|gi|gl|gm|gn|gp|gq|gr|gs|gt|gu|gw|gy|hk|hm|hn|hr|ht|hu|id|ie|il|im|in|iq|ir|is|it|je|jm|jo|jp|ke|kg|kh|ki|km|kn|kp|kr|kw|ky|kz|la|lb|lc|li|lk|lr|ls|lt|lu|lv|ly|ma|mc|md|mg|mh|mk|ml|mm|mn|mo|mp|mq|mr|ms|mt|mu|mv|mw|mx|my|mz|na|nc|ne|nf|ng|ni|no|np|nr|nu|nz|om|pa|pe|pf|pg|ph|pk|pl|pm|pn|pr|ps|pt|pw|py|qa|re|ro|rs|ru|rw|sa|sb|sc|sd|se|sg|sh|si|sj|sk|sl|sm|sn|so|sr|st|su|sv|sy|sz|tc|td|tf|tg|th|tj|tk|tl|tm|tn|to|tp|tr|tt|tv|tw|tz|ua|ug|uk|us|uy|uz|va|vc|ve|vg|vi|vn|vu|wf|ws|ye|yt|za|zm|zw|java|javax|jakarta|sun|oracle|ibm|spring|springframework|apache|hibernate|jboss|wildfly|tomcat|jetty|eclipse|maven|gradle|junit|mockito|slf4j|logback|log4j|guava|gson|jackson|akka|scala|kotlin|groovy|clojure)(?:\.[a-zA-Z_][a-zA-Z0-9_]*)+\b"
    ).expect("Invalid package regex pattern")
});

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

/// Extract package name from a log line using regex pattern matching
/// This approach is more robust and independent of log format
/// Returns (start_pos, end_pos, package_name) if found
fn extract_package_from_log_line<'a>(text: &'a str, _log_format: &str) -> Option<(usize, usize, &'a str)> {
    // Use regex to find Java package pattern in the line
    // This is much more robust than trying to parse the log format
    let captures = PACKAGE_PATTERN.find(text)?;
    
    let start = captures.start();
    let end = captures.end();
    let package_name = captures.as_str();
    
    // Additional validation: reasonable length
    if package_name.len() > 100 {
        return None;
    }
    
    Some((start, end, package_name))
}

/// Create a line with keyword-based coloring (simple approach)
/// Highlights [INFO], [WARNING], [ERROR], [DEBUG] keywords, command lines, and package names
/// 
/// If log_format is provided, will attempt to extract and colorize package names based on the pattern
pub fn colorize_log_line_with_format(text: &str, log_format: Option<&str>) -> Line<'static> {
    let mut spans = Vec::new();

    // Check if this is a command line (starts with $)
    if text.starts_with("$ ") {
        spans.push(Span::styled(
            text.to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(ratatui::style::Modifier::BOLD),
        ));
        return Line::from(spans);
    }
    
    // Try to extract package name if log_format is provided
    let package_info = log_format.and_then(|fmt| extract_package_from_log_line(text, fmt));

    // Find and colorize log level
    if let Some(debug_pos) = text.find("[DEBUG]") {
        colorize_with_level_and_package(text, debug_pos, "[DEBUG]", 7, Color::Magenta, package_info, &mut spans);
    } else if let Some(info_pos) = text.find("[INFO]") {
        colorize_with_level_and_package(text, info_pos, "[INFO]", 6, Color::Green, package_info, &mut spans);
    } else if let Some(warn_pos) = text.find("[WARNING]").or_else(|| text.find("[WARN]")) {
        let (keyword, len) = if text[warn_pos..].starts_with("[WARNING]") {
            ("[WARNING]", 9)
        } else {
            ("[WARN]", 6)
        };
        colorize_with_level_and_package(text, warn_pos, keyword, len, Color::Yellow, package_info, &mut spans);
    } else if let Some(error_pos) = text.find("[ERROR]").or_else(|| text.find("[ERR]")) {
        let (keyword, len) = if text[error_pos..].starts_with("[ERROR]") {
            ("[ERROR]", 7)
        } else {
            ("[ERR]", 5)
        };
        colorize_with_level_and_package(text, error_pos, keyword, len, Color::Red, package_info, &mut spans);
    } else {
        // No special keywords, return as-is
        spans.push(Span::raw(text.to_string()));
    }

    Line::from(spans)
}

/// Helper function to colorize a log line with level and optional package name
fn colorize_with_level_and_package(
    text: &str,
    level_pos: usize,
    level_keyword: &str,
    level_len: usize,
    level_color: Color,
    package_info: Option<(usize, usize, &str)>,
    spans: &mut Vec<Span<'static>>,
) {
    // Add text before level
    if level_pos > 0 {
        spans.push(Span::raw(text[..level_pos].to_string()));
    }
    
    // Add colored level
    spans.push(Span::styled(
        level_keyword.to_string(),
        Style::default().fg(level_color),
    ));
    
    let remaining_start = level_pos + level_len;
    
    // If we have package info, split the remaining text around it
    if let Some((pkg_start, pkg_end, pkg_name)) = package_info {
        if pkg_start >= remaining_start && pkg_start < text.len() {
            // Add text between level and package
            if pkg_start > remaining_start {
                spans.push(Span::raw(text[remaining_start..pkg_start].to_string()));
            }
            
            // Add colored package name
            spans.push(Span::styled(
                pkg_name.to_string(),
                Style::default().fg(Color::Cyan),
            ));
            
            // Add remaining text after package
            if pkg_end < text.len() {
                spans.push(Span::raw(text[pkg_end..].to_string()));
            }
        } else {
            // Package position is invalid, just add remaining text
            if remaining_start < text.len() {
                spans.push(Span::raw(text[remaining_start..].to_string()));
            }
        }
    } else {
        // No package info, just add remaining text
        if remaining_start < text.len() {
            spans.push(Span::raw(text[remaining_start..].to_string()));
        }
    }
}

/// Create a line with keyword-based coloring (simple approach)
/// Highlights [INFO], [WARNING], [ERROR], [DEBUG] keywords and command lines
/// 
/// This is a convenience wrapper around colorize_log_line_with_format for backward compatibility
pub fn colorize_log_line(text: &str) -> Line<'static> {
    colorize_log_line_with_format(text, None)
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

    #[test]
    fn test_colorize_log_line_with_package() {
        // Test with log format containing %p and %c
        let log_format = "[%p] %c - %m%n";
        let log_line = "[INFO] com.example.MyClass - Starting application";
        let line = colorize_log_line_with_format(log_line, Some(log_format));
        
        // Should have at least 4 spans: before level, level, package, and message
        assert!(line.spans.len() >= 3);
        
        // Check that one span contains the package name
        let has_package = line.spans.iter().any(|span| span.content.contains("com.example.MyClass"));
        assert!(has_package, "Package name should be present in spans");
    }

    #[test]
    fn test_colorize_log_line_with_short_logger() {
        // Test with shortened logger format %c{1}
        // Note: with the new regex-based extraction, single-word class names
        // without a package prefix are not detected (which is correct behavior)
        let log_format = "[%p] %c{1} - %m%n";
        let log_line = "[DEBUG] com.example.MyClass - Debug message";
        let line = colorize_log_line_with_format(log_line, Some(log_format));
        
        // Should have at least: level + package + message
        assert!(line.spans.len() >= 3);
        let has_package = line.spans.iter().any(|span| span.content.contains("com.example.MyClass"));
        assert!(has_package, "Package name should be present in spans");
    }

    #[test]
    fn test_colorize_log_line_without_format() {
        // Test backward compatibility - no format provided
        let log_line = "[INFO] Some message";
        let line = colorize_log_line_with_format(log_line, None);
        
        // Should still colorize the level
        assert!(line.spans.len() >= 2);
    }

    #[test]
    fn test_extract_package_from_various_formats() {
        let test_cases = vec![
            // Standard format with full package names
            ("[INFO] com.example.service.UserService - User created", Some("com.example.service.UserService")),
            ("[ERROR] org.springframework.boot.SpringApplication - Failed to start", Some("org.springframework.boot.SpringApplication")),
            // Custom packages with known TLDs
            ("[WARN] test.package.Something - Warning message", None), // 'test' is not a recognized TLD
            ("[WARN] fr.laposte.disf.uid - Warning message", Some("fr.laposte.disf.uid")),
            // Test case with consecutive brackets (common format)
            ("[INFO][fr.foo.bar] Message", Some("fr.foo.bar")),
            ("[DEBUG][com.example.MyClass] Debug", Some("com.example.MyClass")),
            // Java standard packages
            ("[INFO] java.util.ArrayList - Message", Some("java.util.ArrayList")),
            ("[DEBUG] javax.servlet.http.HttpServlet - Message", Some("javax.servlet.http.HttpServlet")),
            // Spring framework
            ("[INFO] org.springframework.context.annotation.AnnotationConfigApplicationContext - Message", Some("org.springframework.context.annotation.AnnotationConfigApplicationContext")),
            // Simple class name without package (should NOT match)
            ("[DEBUG] MyClass - Debug info", None),
        ];
        
        let log_format = "[%p] %c - %m%n";
        
        for (log_line, expected_package) in test_cases {
            let result = extract_package_from_log_line(log_line, log_format);
            
            match expected_package {
                Some(expected) => {
                    assert!(result.is_some(), "Should extract package from: {}", log_line);
                    let (_start, _end, pkg) = result.unwrap();
                    assert_eq!(pkg, expected, "Package mismatch for line: {}", log_line);
                    
                    // Verify that the package name doesn't start with '[' (common mistake)
                    assert!(
                        !pkg.starts_with('['),
                        "Package name should not start with '[': {}",
                        pkg
                    );
                }
                None => {
                    assert!(result.is_none(), "Should NOT extract package from: {}", log_line);
                }
            }
        }
    }

    #[test]
    fn test_extract_package_format_independent() {
        // Test that extraction works regardless of log format
        let test_cases = vec![
            // No brackets
            ("INFO com.example.Service message", Some("com.example.Service")),
            // Different separators
            ("2024-10-30 10:00:00 [INFO] org.apache.kafka.Consumer - Started", Some("org.apache.kafka.Consumer")),
            // Timestamp first
            ("10:00:00.123 DEBUG fr.laposte.Service Processing", Some("fr.laposte.Service")),
            // Package in middle of line
            ("Some text com.google.common.collect.ImmutableList more text", Some("com.google.common.collect.ImmutableList")),
            // Jakarta EE
            ("INFO jakarta.persistence.EntityManager transaction", Some("jakarta.persistence.EntityManager")),
        ];
        
        for (log_line, expected_package) in test_cases {
            let result = extract_package_from_log_line(log_line, "[%p] %c - %m%n");
            
            if let Some(expected) = expected_package {
                assert!(result.is_some(), "Should extract package from: {}", log_line);
                let (_start, _end, pkg) = result.unwrap();
                assert_eq!(pkg, expected, "Package mismatch for line: {}", log_line);
            }
        }
    }
}

/// Extract unique package names from command output lines
/// Returns a sorted list of unique package names found in the logs
pub fn extract_unique_packages(lines: &[String], log_format: Option<&str>) -> Vec<String> {
    if log_format.is_none() {
        return Vec::new();
    }
    
    let format = log_format.unwrap();
    let mut packages = HashSet::new();
    
    for line in lines {
        // Clean the line first
        if let Some(cleaned) = clean_log_line(line) {
            // Try to extract package from this line
            if let Some((_start, _end, package_name)) = extract_package_from_log_line(&cleaned, format) {
                // Only add if it looks like a valid package (contains at least one dot or is a simple word)
                if !package_name.is_empty() && package_name.len() <= 100 {
                    packages.insert(package_name.to_string());
                }
            }
        }
    }
    
    // Convert to sorted vector for consistent ordering
    let mut result: Vec<String> = packages.into_iter().collect();
    result.sort();
    result
}
