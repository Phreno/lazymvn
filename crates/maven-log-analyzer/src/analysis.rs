//! Log analysis and extraction utilities
//!
//! This module provides functions for extracting and analyzing log content:
//! - Package name extraction from log lines
//! - False positive filtering
//! - Unique package collection for statistics
//!
//! These functions use the regex patterns defined in `patterns` module.

use crate::patterns::*;
use std::collections::HashSet;

/// Extract package name from a log line using regex pattern matching
/// Uses a three-pass approach for robustness:
/// 1. First pass: Try to match packages with known prefixes (com, org, fr, etc.) - most precise
/// 2. Second pass: Try generic 3+ segment packages (service.impl.Class)
/// 3. Third pass: If in log context, try permissive 2+ segment pattern (service.Class)
///
/// This handles complete, truncated, and heavily truncated package names
/// Returns (start_pos, end_pos, package_name) if found
pub fn extract_package_from_log_line<'a>(
    text: &'a str,
    _log_format: &str,
) -> Option<(usize, usize, &'a str)> {
    try_extract_with_prefix(text)
        .or_else(|| try_extract_generic(text))
        .or_else(|| try_extract_permissive(text))
}

/// Try to extract package with known prefix (most precise)
fn try_extract_with_prefix(text: &str) -> Option<(usize, usize, &str)> {
    PACKAGE_PATTERN_WITH_PREFIX
        .find(text)
        .and_then(|captures| validate_package_match(&captures))
}

/// Try to extract generic 3+ segment package
fn try_extract_generic(text: &str) -> Option<(usize, usize, &str)> {
    PACKAGE_PATTERN_GENERIC
        .find(text)
        .and_then(|captures| validate_package_match(&captures))
}

/// Try to extract permissive 2+ segment package if in log context
fn try_extract_permissive(text: &str) -> Option<(usize, usize, &str)> {
    if has_log_level(text) {
        PACKAGE_PATTERN_PERMISSIVE
            .find(text)
            .and_then(|captures| validate_package_match(&captures))
    } else {
        None
    }
}

/// Validate a regex match as a valid package
fn validate_package_match<'a>(captures: &regex::Match<'a>) -> Option<(usize, usize, &'a str)> {
    let package_name = captures.as_str();
    if is_valid_package_length(package_name) && !is_false_positive(package_name) {
        Some((captures.start(), captures.end(), package_name))
    } else {
        None
    }
}

/// Check if package name has valid length
fn is_valid_package_length(package_name: &str) -> bool {
    !package_name.is_empty() && package_name.len() <= 100
}

/// Check if text contains a log level marker
fn has_log_level(text: &str) -> bool {
    ["[DEBUG]", "[INFO]", "[WARN]", "[ERROR]", "[ERR]", "DEBUG", "INFO", "WARN", "ERROR"]
        .iter()
        .any(|level| text.contains(level))
}

/// Check if a potential package name is actually a false positive
pub fn is_false_positive(package_name: &str) -> bool {
    let lowercase = package_name.to_lowercase();
    
    is_ambiguous_tld(&lowercase)
        || has_file_extensions(&lowercase)
        || has_url_like_patterns(&lowercase)
        || has_common_non_package_patterns(&lowercase)
}

/// Check if package starts with ambiguous TLD pattern
fn is_ambiguous_tld(lowercase: &str) -> bool {
    if lowercase.starts_with("my.") {
        // Allow known patterns like my.company.*, but reject short ones like my.Class
        lowercase.split('.').count() <= 2
    } else {
        false
    }
}

/// Check if string matches common non-package patterns
fn has_common_non_package_patterns(text: &str) -> bool {
    text.starts_with("file.")
        || text.starts_with("path.")
        || text == "my.property"
        || text == "some.value"
}

/// Check if string matches URL-like patterns
fn has_url_like_patterns(text: &str) -> bool {
    ["http.", "https.", "www."]
        .iter()
        .any(|prefix| text.starts_with(prefix))
}

/// Check if string ends with common file extensions
fn has_file_extensions(text: &str) -> bool {
    [".xml", ".json", ".properties", ".yml", ".yaml", ".txt", ".log"]
        .iter()
        .any(|ext| text.ends_with(ext))
}

/// Extract unique package names from command output lines
/// Returns a sorted list of unique package names found in the logs
pub fn extract_unique_packages(lines: &[String], log_format: Option<&str>) -> Vec<String> {
    log_format
        .map(|format| collect_unique_packages(lines, format))
        .unwrap_or_default()
}

/// Collect unique packages from lines
fn collect_unique_packages(lines: &[String], format: &str) -> Vec<String> {
    let packages: HashSet<String> = lines
        .iter()
        .filter_map(|line| extract_package_from_log_line(line, format))
        .map(|(_, _, package_name)| package_name)
        .filter(|pkg| is_valid_package_length(pkg))
        .map(String::from)
        .collect();

    to_sorted_vec(packages)
}

/// Convert HashSet to sorted Vec
fn to_sorted_vec(set: HashSet<String>) -> Vec<String> {
    let mut result: Vec<String> = set.into_iter().collect();
    result.sort();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_package_from_various_formats() {
        let test_cases = vec![
            // Standard format with full package names
            (
                "[INFO] com.example.service.UserService - User created",
                Some("com.example.service.UserService"),
            ),
            (
                "[ERROR] org.springframework.boot.SpringApplication - Failed to start",
                Some("org.springframework.boot.SpringApplication"),
            ),
            // Custom packages with known TLDs
            (
                "[WARN] fr.foo.bar.uid - Warning message",
                Some("fr.foo.bar.uid"),
            ),
            // Test case with consecutive brackets (common format)
            ("[INFO][fr.foo.bar] Message", Some("fr.foo.bar")),
            (
                "[DEBUG][com.example.MyClass] Debug",
                Some("com.example.MyClass"),
            ),
            // Java standard packages
            (
                "[INFO] java.util.ArrayList - Message",
                Some("java.util.ArrayList"),
            ),
            (
                "[DEBUG] javax.servlet.http.HttpServlet - Message",
                Some("javax.servlet.http.HttpServlet"),
            ),
            // UK domain
            (
                "[INFO] uk.co.company.Service - UK company service",
                Some("uk.co.company.Service"),
            ),
            // Spring framework
            (
                "[INFO] org.springframework.context.annotation.AnnotationConfigApplicationContext - Message",
                Some("org.springframework.context.annotation.AnnotationConfigApplicationContext"),
            ),
            // Test packages (common in unit tests, 3+ segments should be detected even without recognized TLD)
            (
                "[WARN] test.package.Something - Warning message",
                Some("test.package.Something"),
            ),
            // Simple class name without package (should NOT match)
            ("[DEBUG] MyClass - Debug info", None),
        ];

        let log_format = "[%p] %c - %m%n";

        for (log_line, expected_package) in test_cases {
            let result = extract_package_from_log_line(log_line, log_format);

            match expected_package {
                Some(expected) => {
                    assert!(
                        result.is_some(),
                        "Should extract package from: {}",
                        log_line
                    );
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
                    assert!(
                        result.is_none(),
                        "Should NOT extract package from: {}",
                        log_line
                    );
                }
            }
        }
    }

    #[test]
    fn test_extract_package_format_independent() {
        // Test that extraction works regardless of log format
        let test_cases = vec![
            // No brackets
            (
                "INFO com.example.Service message",
                Some("com.example.Service"),
            ),
            // Different separators
            (
                "2024-10-30 10:00:00 [INFO] org.apache.kafka.Consumer - Started",
                Some("org.apache.kafka.Consumer"),
            ),
            // Timestamp first
            (
                "10:00:00.123 DEBUG fr.foo.Service Processing",
                Some("fr.foo.Service"),
            ),
            // Package in middle of line
            (
                "Some text com.google.common.collect.ImmutableList more text",
                Some("com.google.common.collect.ImmutableList"),
            ),
            // Jakarta EE
            (
                "INFO jakarta.persistence.EntityManager transaction",
                Some("jakarta.persistence.EntityManager"),
            ),
        ];

        for (log_line, expected_package) in test_cases {
            let result = extract_package_from_log_line(log_line, "[%p] %c - %m%n");

            if let Some(expected) = expected_package {
                assert!(
                    result.is_some(),
                    "Should extract package from: {}",
                    log_line
                );
                let (_start, _end, pkg) = result.unwrap();
                assert_eq!(pkg, expected, "Package mismatch for line: {}", log_line);
            }
        }
    }

    #[test]
    fn test_extract_truncated_packages() {
        // Test extraction of packages that have been truncated (e.g., with %c{1} or %c{2})
        // These don't start with known prefixes but should still be detected
        let test_cases = vec![
            // Truncated to last 2 segments
            (
                "[INFO] service.UserService - Message",
                Some("service.UserService"),
            ),
            // Truncated to last 3 segments
            (
                "[DEBUG] impl.service.MyService - Debug",
                Some("impl.service.MyService"),
            ),
            // Common truncation patterns
            (
                "[INFO] controller.api.RestController - Request",
                Some("controller.api.RestController"),
            ),
            (
                "[ERROR] repository.data.UserRepository - Error",
                Some("repository.data.UserRepository"),
            ),
            // Still requires minimum 3 segments to avoid false positives
            ("[INFO] MyClass - Message", None),
            ("[DEBUG] my.Class - Debug", None),
        ];

        for (log_line, expected_package) in test_cases {
            let result = extract_package_from_log_line(log_line, "[%p] %c - %m%n");

            match expected_package {
                Some(expected) => {
                    assert!(
                        result.is_some(),
                        "Should extract truncated package from: {}",
                        log_line
                    );
                    let (_start, _end, pkg) = result.unwrap();
                    assert_eq!(pkg, expected, "Package mismatch for line: {}", log_line);
                }
                None => {
                    assert!(
                        result.is_none(),
                        "Should NOT extract from: {} (too few segments)",
                        log_line
                    );
                }
            }
        }
    }

    #[test]
    fn test_is_false_positive() {
        let test_cases = vec![
            // File extensions
            ("config.xml", true),
            ("data.json", true),
            ("application.properties", true),
            ("settings.yml", true),
            ("output.log", true),
            // URL patterns
            ("http.client", true),
            ("https.server", true),
            ("www.example", true),
            // Ambiguous patterns
            ("my.Class", true),
            ("file.path", true),
            ("path.to", true),
            // Valid packages
            ("com.example.Service", false),
            ("org.springframework.Application", false),
            ("my.company.Module", false), // 3+ segments is ok
        ];

        for (input, expected) in test_cases {
            let result = is_false_positive(input);
            assert_eq!(
                result, expected,
                "False positive check failed for '{}': expected {}, got {}",
                input, expected, result
            );
        }
    }

    #[test]
    fn test_extract_unique_packages() {
        let lines = vec![
            "[INFO] com.example.Service - Message 1".to_string(),
            "[DEBUG] org.springframework.Application - Message 2".to_string(),
            "[ERROR] com.example.Service - Message 3".to_string(), // Duplicate
            "[WARN] fr.company.Module - Message 4".to_string(),
            "Plain text without package".to_string(),
        ];

        let log_format = Some("[%p] %c - %m%n");
        let packages = extract_unique_packages(&lines, log_format);

        assert_eq!(packages.len(), 3);
        assert!(packages.contains(&"com.example.Service".to_string()));
        assert!(packages.contains(&"org.springframework.Application".to_string()));
        assert!(packages.contains(&"fr.company.Module".to_string()));

        // Check that result is sorted
        let mut sorted = packages.clone();
        sorted.sort();
        assert_eq!(packages, sorted);
    }

    #[test]
    fn test_extract_unique_packages_no_format() {
        let lines = vec!["[INFO] com.example.Service - Message".to_string()];

        let packages = extract_unique_packages(&lines, None);
        assert!(
            packages.is_empty(),
            "Should return empty vec when no format provided"
        );
    }

    #[test]
    fn test_has_log_level() {
        assert!(has_log_level("[DEBUG] message"));
        assert!(has_log_level("[INFO] message"));
        assert!(has_log_level("[WARN] message"));
        assert!(has_log_level("[ERROR] message"));
        assert!(has_log_level("[ERR] message"));
        assert!(has_log_level("DEBUG message"));
        assert!(has_log_level("INFO message"));
        assert!(!has_log_level("plain text"));
        assert!(!has_log_level("TRACE message"));
    }

    #[test]
    fn test_has_file_extensions() {
        assert!(has_file_extensions("config.xml"));
        assert!(has_file_extensions("data.json"));
        assert!(has_file_extensions("app.properties"));
        assert!(has_file_extensions("config.yml"));
        assert!(has_file_extensions("config.yaml"));
        assert!(has_file_extensions("output.log"));
        assert!(!has_file_extensions("com.example.Service"));
    }

    #[test]
    fn test_has_url_like_patterns() {
        assert!(has_url_like_patterns("http.client"));
        assert!(has_url_like_patterns("https.server"));
        assert!(has_url_like_patterns("www.example"));
        assert!(!has_url_like_patterns("com.example.Service"));
    }

    #[test]
    fn test_has_common_non_package_patterns() {
        assert!(has_common_non_package_patterns("file.path"));
        assert!(has_common_non_package_patterns("path.to"));
        assert!(has_common_non_package_patterns("my.property"));
        assert!(has_common_non_package_patterns("some.value"));
        assert!(!has_common_non_package_patterns("com.example.Service"));
    }

    #[test]
    fn test_is_ambiguous_tld() {
        assert!(is_ambiguous_tld("my.Class"));
        assert!(is_ambiguous_tld("my.property"));
        assert!(!is_ambiguous_tld("my.company.Service"));
        assert!(!is_ambiguous_tld("com.example.Service"));
    }

    #[test]
    fn test_is_valid_package_length() {
        assert!(is_valid_package_length("com.example"));
        assert!(is_valid_package_length(&"a".repeat(100)));
        assert!(!is_valid_package_length(""));
        assert!(!is_valid_package_length(&"a".repeat(101)));
    }
}
