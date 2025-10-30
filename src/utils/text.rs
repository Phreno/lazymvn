//! Text processing utilities
//!
//! Provides functions for text formatting, colorization, and cleaning:
//! - ANSI escape sequence stripping
//! - Log line colorization
//! - XML syntax highlighting

use crate::utils::log_analysis::extract_package_from_log_line;
use crate::utils::log_patterns::{EXCEPTION_PATTERN, STACKTRACE_PATTERN};
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
    
    // Check if this is a stack trace line (starts with "at ")
    if let Some(captures) = STACKTRACE_PATTERN.captures(text) {
        colorize_stacktrace_line(text, captures, &mut spans);
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
                colorize_with_exceptions(&text[remaining_start..pkg_start], spans);
            }
            
            // Add colored package name
            spans.push(Span::styled(
                pkg_name.to_string(),
                Style::default().fg(Color::Cyan),
            ));
            
            // Add remaining text after package (with exception highlighting)
            if pkg_end < text.len() {
                colorize_with_exceptions(&text[pkg_end..], spans);
            }
        } else {
            // Package position is invalid, just add remaining text with exception highlighting
            if remaining_start < text.len() {
                colorize_with_exceptions(&text[remaining_start..], spans);
            }
        }
    } else {
        // No package info, just add remaining text with exception highlighting
        if remaining_start < text.len() {
            colorize_with_exceptions(&text[remaining_start..], spans);
        }
    }
}

/// Helper function to colorize text with exception highlighting
/// Searches for Java exception names and highlights them in red
fn colorize_with_exceptions(text: &str, spans: &mut Vec<Span<'static>>) {
    let mut last_end = 0;
    
    // Find all exceptions in the text
    for exception_match in EXCEPTION_PATTERN.find_iter(text) {
        let start = exception_match.start();
        let end = exception_match.end();
        
        // Add text before exception (if any)
        if start > last_end {
            spans.push(Span::raw(text[last_end..start].to_string()));
        }
        
        // Add colored exception name
        spans.push(Span::styled(
            exception_match.as_str().to_string(),
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(ratatui::style::Modifier::BOLD),
        ));
        
        last_end = end;
    }
    
    // Add remaining text (if any)
    if last_end < text.len() {
        spans.push(Span::raw(text[last_end..].to_string()));
    }
}

/// Helper function to colorize a Java stack trace line
/// Format: "at com.example.MyClass.myMethod(MyClass.java:42)"
/// Captures: (1) = full class path, (2) = method name, (3) = source location
fn colorize_stacktrace_line(text: &str, captures: regex::Captures, spans: &mut Vec<Span<'static>>) {
    let full_match = captures.get(0).unwrap();
    let class_path = captures.get(1).map(|m| m.as_str()).unwrap_or("");
    let method_name = captures.get(2).map(|m| m.as_str()).unwrap_or("");
    let source_location = captures.get(3).map(|m| m.as_str()).unwrap_or("");
    
    // Calculate positions
    let leading_whitespace = &text[..full_match.start()];
    
    // Add leading whitespace (indentation)
    if !leading_whitespace.is_empty() {
        spans.push(Span::raw(leading_whitespace.to_string()));
    }
    
    // Add "at" keyword in dark gray
    spans.push(Span::styled(
        "at ".to_string(),
        Style::default().fg(Color::DarkGray),
    ));
    
    // Add class path in cyan (like packages)
    spans.push(Span::styled(
        class_path.to_string(),
        Style::default().fg(Color::Cyan),
    ));
    
    // Add dot separator
    spans.push(Span::styled(
        ".".to_string(),
        Style::default().fg(Color::DarkGray),
    ));
    
    // Add method name in light yellow
    spans.push(Span::styled(
        method_name.to_string(),
        Style::default().fg(Color::LightYellow),
    ));
    
    // Add opening parenthesis
    spans.push(Span::styled(
        "(".to_string(),
        Style::default().fg(Color::DarkGray),
    ));
    
    // Add source location in gray
    spans.push(Span::styled(
        source_location.to_string(),
        Style::default().fg(Color::Gray),
    ));
    
    // Add closing parenthesis
    spans.push(Span::styled(
        ")".to_string(),
        Style::default().fg(Color::DarkGray),
    ));
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
    fn test_colorize_exceptions() {
        // Test that exception names are highlighted
        let test_cases = vec![
            "[ERROR] com.example.Service - NullPointerException occurred",
            "[WARN] java.io.FileReader - IOException: file not found",
            "[ERROR] Failed with IllegalArgumentException and RuntimeException",
            "[DEBUG] Caught SQLException while processing",
        ];
        
        let log_format = "[%p] %c - %m%n";
        
        for log_line in test_cases {
            let line = colorize_log_line_with_format(log_line, Some(log_format));
            
            // Should have multiple spans (level + exception(s) + other text)
            assert!(line.spans.len() >= 3, "Line should have multiple spans for: {}", log_line);
            
            // Check that at least one span is styled (for the exception)
            let has_styled_exception = line.spans.iter().any(|span| {
                span.content.contains("Exception") && !span.style.fg.is_none()
            });
            
            assert!(has_styled_exception, "Should highlight exception in: {}", log_line);
        }
    }

    #[test]
    fn test_colorize_multiple_exceptions() {
        // Test line with multiple exceptions
        let log_line = "[ERROR] Caught IOException then RuntimeException";
        let line = colorize_log_line_with_format(log_line, Some("[%p] %m%n"));
        
        // Count how many exception spans we have
        let exception_spans: Vec<_> = line.spans.iter()
            .filter(|span| span.content.contains("Exception"))
            .collect();
        
        assert_eq!(exception_spans.len(), 2, "Should find both exceptions");
    }

    #[test]
    fn test_exception_with_package_colorization() {
        // Test that both package and exception are colored
        let log_line = "[ERROR] com.example.MyService - NullPointerException in method";
        let line = colorize_log_line_with_format(log_line, Some("[%p] %c - %m%n"));
        
        // Should have: level, package (cyan), exception (red), and other text
        let has_package = line.spans.iter().any(|span| {
            span.content.contains("com.example.MyService")
        });
        
        let has_exception = line.spans.iter().any(|span| {
            span.content.contains("NullPointerException")
        });
        
        assert!(has_package, "Should highlight package");
        assert!(has_exception, "Should highlight exception");
    }

    #[test]
    fn test_colorize_stacktrace() {
        // Test basic stack trace line
        let stacktrace = "    at com.example.MyClass.myMethod(MyClass.java:42)";
        let line = colorize_log_line_with_format(stacktrace, None);
        
        // Should have multiple spans for different parts
        assert!(line.spans.len() >= 5, "Should have multiple spans for stacktrace parts");
        
        // Check that key parts are present
        let has_at = line.spans.iter().any(|span| span.content == "at ");
        let has_class = line.spans.iter().any(|span| span.content.contains("com.example.MyClass"));
        let has_method = line.spans.iter().any(|span| span.content.contains("myMethod"));
        let has_source = line.spans.iter().any(|span| span.content.contains("MyClass.java:42"));
        
        assert!(has_at, "Should contain 'at' keyword");
        assert!(has_class, "Should contain class path");
        assert!(has_method, "Should contain method name");
        assert!(has_source, "Should contain source location");
    }

    #[test]
    fn test_colorize_stacktrace_with_generics() {
        // Test stack trace with generic method names (e.g., <init>, <clinit>)
        let stacktrace = "    at org.springframework.boot.SpringApplication.<init>(SpringApplication.java:123)";
        let line = colorize_log_line_with_format(stacktrace, None);
        
        assert!(line.spans.len() >= 5, "Should handle generic method names");
        
        let has_init = line.spans.iter().any(|span| span.content.contains("<init>"));
        assert!(has_init, "Should detect <init> method");
    }

    #[test]
    fn test_colorize_stacktrace_with_inner_class() {
        // Test stack trace with inner classes (using $)
        let stacktrace = "    at com.example.OuterClass$InnerClass.method(OuterClass.java:99)";
        let line = colorize_log_line_with_format(stacktrace, None);
        
        assert!(line.spans.len() >= 5, "Should handle inner class notation");
        
        let has_inner = line.spans.iter().any(|span| span.content.contains("OuterClass$InnerClass"));
        assert!(has_inner, "Should detect inner class");
    }

    #[test]
    fn test_normal_line_vs_stacktrace() {
        // Ensure normal lines starting with "at" but not stack traces aren't misidentified
        let normal_line = "[INFO] Starting application at port 8080";
        let line = colorize_log_line_with_format(normal_line, Some("[%p] %m%n"));
        
        // Should be colored as log level, not as stacktrace
        let has_info = line.spans.iter().any(|span| span.content == "[INFO]");
        assert!(has_info, "Should recognize as normal log line, not stacktrace");
    }
}
