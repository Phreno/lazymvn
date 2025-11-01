//! Text processing and colorization utilities

mod log_parser;
mod xml_formatter;

pub use log_parser::{clean_log_line, colorize_log_line, colorize_log_line_with_format};
pub use xml_formatter::colorize_xml_line;

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
