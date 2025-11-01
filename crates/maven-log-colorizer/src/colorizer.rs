use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use maven_log_analyzer::analysis::extract_package_from_log_line;
use maven_log_analyzer::patterns::{EXCEPTION_PATTERN, STACKTRACE_PATTERN};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colorize_plain_text() {
        let line = colorize_log_line("Plain text");
        assert_eq!(line.spans.len(), 1);
    }

    #[test]
    fn test_colorize_info_level() {
        let line = colorize_log_line("This is [INFO] message");
        assert!(line.spans.len() > 1);
        // Should have at least: text before, INFO keyword, text after
        assert!(line.spans.iter().any(|s| s.content.contains("INFO")));
    }

    #[test]
    fn test_colorize_with_package() {
        let log_line = "[INFO] com.example.MyClass - Processing started";
        let log_format = "[%p] %c - %m%n";
        let line = colorize_log_line_with_format(log_line, Some(log_format));
        
        // Should have multiple spans including package name
        assert!(line.spans.len() > 2);
        assert!(line.spans.iter().any(|s| s.content.contains("com.example.MyClass")));
    }

    #[test]
    fn test_colorize_exception() {
        let log_line = "[ERROR] Failed with NullPointerException";
        let line = colorize_log_line(log_line);
        
        // Should highlight both ERROR and NullPointerException
        assert!(line.spans.len() > 2);
        assert!(line.spans.iter().any(|s| s.content.contains("NullPointerException")));
    }

    #[test]
    fn test_colorize_stack_trace() {
        let log_line = "    at com.example.MyClass.myMethod(MyClass.java:42)";
        let line = colorize_log_line(log_line);
        
        // Should have multiple colored segments
        assert!(line.spans.len() > 5);
        assert!(line.spans.iter().any(|s| s.content.contains("com.example.MyClass")));
        assert!(line.spans.iter().any(|s| s.content.contains("myMethod")));
    }

    #[test]
    fn test_colorize_command_line() {
        let log_line = "$ mvn clean install";
        let line = colorize_log_line(log_line);
        
        assert_eq!(line.spans.len(), 1);
        assert_eq!(line.spans[0].content, "$ mvn clean install");
    }
}
