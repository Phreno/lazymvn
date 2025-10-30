//! Log analysis and extraction utilities
//!
//! This module provides functions for extracting and analyzing log content:
//! - Package name extraction from log lines
//! - False positive filtering
//! - Unique package collection for statistics
//!
//! These functions use the regex patterns defined in `log_patterns` module.

use crate::utils::log_patterns::*;
use std::collections::HashSet;

/// Extract package name from a log line using regex pattern matching
/// Uses a three-pass approach for robustness:
/// 1. First pass: Try to match packages with known prefixes (com, org, fr, etc.) - most precise
/// 2. Second pass: Try generic 3+ segment packages (service.impl.Class)
/// 3. Third pass: If in log context, try permissive 2+ segment pattern (service.Class)
/// This handles complete, truncated, and heavily truncated package names
/// Returns (start_pos, end_pos, package_name) if found
pub fn extract_package_from_log_line<'a>(text: &'a str, _log_format: &str) -> Option<(usize, usize, &'a str)> {
    // First pass: Try with known prefix (most precise, preferred)
    if let Some(captures) = PACKAGE_PATTERN_WITH_PREFIX.find(text) {
        let start = captures.start();
        let end = captures.end();
        let package_name = captures.as_str();
        
        if package_name.len() <= 100 && !is_false_positive(package_name) {
            return Some((start, end, package_name));
        }
    }
    
    // Second pass: Try generic pattern (3+ segments without prefix requirement)
    if let Some(captures) = PACKAGE_PATTERN_GENERIC.find(text) {
        let start = captures.start();
        let end = captures.end();
        let package_name = captures.as_str();
        
        if package_name.len() <= 100 && !is_false_positive(package_name) {
            return Some((start, end, package_name));
        }
    }
    
    // Check if this looks like a log line (has log level marker)
    let has_log_level = text.contains("[DEBUG]") 
        || text.contains("[INFO]") 
        || text.contains("[WARN")
        || text.contains("[ERROR]")
        || text.contains("[ERR]")
        || text.contains("DEBUG")
        || text.contains("INFO")
        || text.contains("WARN")
        || text.contains("ERROR");
    
    // Third pass: If in log context, try more permissive pattern (2+ segments)
    // This catches heavily truncated logger names like "service.UserService"
    if has_log_level {
        if let Some(captures) = PACKAGE_PATTERN_PERMISSIVE.find(text) {
            let start = captures.start();
            let end = captures.end();
            let package_name = captures.as_str();
            
            if package_name.len() <= 100 && !is_false_positive(package_name) {
                return Some((start, end, package_name));
            }
        }
    }
    
    None
}

/// Check if a potential package name is actually a false positive
pub fn is_false_positive(package_name: &str) -> bool {
    let lowercase = package_name.to_lowercase();
    
    // Ambiguous TLDs that are unlikely to be Java packages
    // "my" is Malaysia TLD but commonly used in generic code (my.Class, my.Property, etc.)
    if lowercase.starts_with("my.") {
        // Allow known patterns like my.company.*, but reject short ones like my.Class
        let parts: Vec<&str> = lowercase.split('.').collect();
        if parts.len() <= 2 {
            return true;
        }
    }
    
    // File extensions
    if lowercase.ends_with(".xml")
        || lowercase.ends_with(".json")
        || lowercase.ends_with(".properties")
        || lowercase.ends_with(".yml")
        || lowercase.ends_with(".yaml")
        || lowercase.ends_with(".txt")
        || lowercase.ends_with(".log") {
        return true;
    }
    
    // URL-like patterns
    if lowercase.starts_with("http.") 
        || lowercase.starts_with("https.")
        || lowercase.starts_with("www.") {
        return true;
    }
    
    // Common non-package patterns
    if lowercase.starts_with("file.")
        || lowercase.starts_with("path.")
        || lowercase == "my.property"
        || lowercase == "some.value" {
        return true;
    }
    
    false
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
        // Try to extract package from this line
        if let Some((_start, _end, package_name)) = extract_package_from_log_line(line, format) {
            // Only add if it looks like a valid package (contains at least one dot or is a simple word)
            if !package_name.is_empty() && package_name.len() <= 100 {
                packages.insert(package_name.to_string());
            }
        }
    }
    
    // Convert to sorted vector for consistent ordering
    let mut result: Vec<String> = packages.into_iter().collect();
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
            ("[INFO] com.example.service.UserService - User created", Some("com.example.service.UserService")),
            ("[ERROR] org.springframework.boot.SpringApplication - Failed to start", Some("org.springframework.boot.SpringApplication")),
            // Custom packages with known TLDs
            ("[WARN] fr.foo.bar.uid - Warning message", Some("fr.foo.bar.uid")),
            // Test case with consecutive brackets (common format)
            ("[INFO][fr.foo.bar] Message", Some("fr.foo.bar")),
            ("[DEBUG][com.example.MyClass] Debug", Some("com.example.MyClass")),
            // Java standard packages
            ("[INFO] java.util.ArrayList - Message", Some("java.util.ArrayList")),
            ("[DEBUG] javax.servlet.http.HttpServlet - Message", Some("javax.servlet.http.HttpServlet")),
            // UK domain
            ("[INFO] uk.co.company.Service - UK company service", Some("uk.co.company.Service")),
            // Spring framework
            ("[INFO] org.springframework.context.annotation.AnnotationConfigApplicationContext - Message", Some("org.springframework.context.annotation.AnnotationConfigApplicationContext")),
            // Test packages (common in unit tests, 3+ segments should be detected even without recognized TLD)
            ("[WARN] test.package.Something - Warning message", Some("test.package.Something")),
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
            ("10:00:00.123 DEBUG fr.foo.Service Processing", Some("fr.foo.Service")),
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

    #[test]
    fn test_extract_truncated_packages() {
        // Test extraction of packages that have been truncated (e.g., with %c{1} or %c{2})
        // These don't start with known prefixes but should still be detected
        let test_cases = vec![
            // Truncated to last 2 segments
            ("[INFO] service.UserService - Message", Some("service.UserService")),
            // Truncated to last 3 segments  
            ("[DEBUG] impl.service.MyService - Debug", Some("impl.service.MyService")),
            // Common truncation patterns
            ("[INFO] controller.api.RestController - Request", Some("controller.api.RestController")),
            ("[ERROR] repository.data.UserRepository - Error", Some("repository.data.UserRepository")),
            // Still requires minimum 3 segments to avoid false positives
            ("[INFO] MyClass - Message", None),
            ("[DEBUG] my.Class - Debug", None),
        ];
        
        for (log_line, expected_package) in test_cases {
            let result = extract_package_from_log_line(log_line, "[%p] %c - %m%n");
            
            match expected_package {
                Some(expected) => {
                    assert!(result.is_some(), "Should extract truncated package from: {}", log_line);
                    let (_start, _end, pkg) = result.unwrap();
                    assert_eq!(pkg, expected, "Package mismatch for line: {}", log_line);
                }
                None => {
                    assert!(result.is_none(), "Should NOT extract from: {} (too few segments)", log_line);
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
        let lines = vec![
            "[INFO] com.example.Service - Message".to_string(),
        ];

        let packages = extract_unique_packages(&lines, None);
        assert!(packages.is_empty(), "Should return empty vec when no format provided");
    }
}
