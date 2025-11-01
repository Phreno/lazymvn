//! Log pattern detection regex
//!
//! This module contains all regex patterns used for detecting and analyzing log content:
//! - Java package names (with different levels of precision)
//! - Java exceptions
//! - Stack trace lines
//!
//! These patterns are compiled once using LazyLock for performance and can be used
//! for both colorization and statistical analysis of log files.

use regex::Regex;
use std::sync::LazyLock;

/// Regex pattern for detecting Java package names with known prefixes
/// Matches packages starting with common TLDs or well-known Java namespaces
///
/// This is the most precise pattern and should be tried first.
/// Examples: com.example.Service, org.springframework.boot.Application, fr.company.Module
pub static PACKAGE_PATTERN_WITH_PREFIX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\b(?:com|org|net|io|fr|de|uk|nl|eu|gov|edu|mil|int|co|me|info|biz|mobi|name|pro|aero|asia|cat|coop|jobs|museum|tel|travel|xxx|ac|ad|ae|af|ag|ai|al|am|ao|aq|ar|as|at|au|aw|ax|az|ba|bb|bd|be|bf|bg|bh|bi|bj|bm|bn|bo|br|bs|bt|bv|bw|by|bz|ca|cc|cd|cf|cg|ch|ci|ck|cl|cm|cn|cr|cu|cv|cw|cx|cy|cz|dj|dk|dm|do|dz|ec|ee|eg|er|es|et|fi|fj|fk|fm|fo|ga|gb|gd|ge|gf|gg|gh|gi|gl|gm|gn|gp|gq|gr|gs|gt|gu|gw|gy|hk|hm|hn|hr|ht|hu|id|ie|il|im|in|iq|ir|is|it|je|jm|jo|jp|ke|kg|kh|ki|km|kn|kp|kr|kw|ky|kz|la|lb|lc|li|lk|lr|ls|lt|lu|lv|ly|ma|mc|md|mg|mh|mk|ml|mm|mn|mo|mp|mq|mr|ms|mt|mu|mv|mw|mx|my|mz|na|nc|ne|nf|ng|ni|no|np|nr|nu|nz|om|pa|pe|pf|pg|ph|pk|pl|pm|pn|pr|ps|pt|pw|py|qa|re|ro|rs|ru|rw|sa|sb|sc|sd|se|sg|sh|si|sj|sk|sl|sm|sn|so|sr|st|su|sv|sy|sz|tc|td|tf|tg|th|tj|tk|tl|tm|tn|to|tp|tr|tt|tv|tw|tz|ua|ug|uk|us|uy|uz|va|vc|ve|vg|vi|vn|vu|wf|ws|ye|yt|za|zm|zw|java|javax|jakarta|sun|oracle|ibm|spring|springframework|apache|hibernate|jboss|wildfly|tomcat|jetty|eclipse|maven|gradle|junit|mockito|slf4j|logback|log4j|guava|gson|jackson|akka|scala|kotlin|groovy|clojure)(?:\.[a-zA-Z_][a-zA-Z0-9_]*)+\b"
    ).expect("Invalid package regex pattern")
});

/// Regex pattern for detecting Java package names without requiring specific prefix
/// More permissive: matches any lowercase word followed by at least 2 more segments
/// Pattern: word.word.word (minimum 3 segments to avoid false positives)
///
/// Examples: service.impl.userservice (all lowercase with 3+ segments)
pub static PACKAGE_PATTERN_GENERIC: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\b[a-z][a-z0-9_]*(?:\.[a-z][a-z0-9_]*){2,}\b"
    ).expect("Invalid generic package regex pattern")
});

/// Permissive pattern for truncated packages (minimum 2 segments)
/// Use only when in log context to avoid false positives
/// Each segment must be at least 3 characters to avoid ambiguous matches like "my.Class"
/// Starts with lowercase word, followed by at least one more segment (can be capitalized class name)
///
/// Examples: service.UserService, impl.MyClass (used for truncated logger names like %c{1})
pub static PACKAGE_PATTERN_PERMISSIVE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b[a-z][a-z0-9_]{2,}\.(?:[a-z][a-z0-9_]{2,}|[A-Z][a-zA-Z0-9_]{2,})(?:\.(?:[a-z][a-z0-9_]{2,}|[A-Z][a-zA-Z0-9_]{2,}))*\b")
        .expect("Invalid permissive package regex pattern")
});

/// Regex pattern for detecting Java exceptions
/// Matches class names ending with "Exception" (e.g., NullPointerException, IOException)
/// Requires at least one character before "Exception"
///
/// Examples: NullPointerException, IOException, IllegalArgumentException
pub static EXCEPTION_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b[A-Z][a-zA-Z0-9]*Exception\b").expect("Invalid exception regex pattern")
});

/// Regex pattern for detecting Java stack trace lines
/// Matches lines like: at com.example.MyClass.myMethod(MyClass.java:42)
///
/// Captures:
/// - Group 1: Full class path (e.g., com.example.MyClass)
/// - Group 2: Method name (e.g., myMethod, <init>)
/// - Group 3: Source location (e.g., MyClass.java:42)
pub static STACKTRACE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*at\s+([a-zA-Z0-9_.$]+)\.([a-zA-Z0-9_<>]+)\(([^)]+)\)\s*$")
        .expect("Invalid stacktrace regex pattern")
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_pattern_with_prefix() {
        let test_cases = vec![
            ("com.example.Service", true),
            ("org.springframework.boot.Application", true),
            ("fr.company.Module", true),
            ("java.util.ArrayList", true),
            ("javax.servlet.HttpServlet", true),
            ("jakarta.persistence.Entity", true),
            ("uk.co.company.Service", true),
            ("test.package.Class", false), // No known prefix
            ("MyClass", false),
        ];

        for (input, should_match) in test_cases {
            let matches = PACKAGE_PATTERN_WITH_PREFIX.is_match(input);
            assert_eq!(
                matches, should_match,
                "Pattern mismatch for '{}': expected {}, got {}",
                input, should_match, matches
            );
        }
    }

    #[test]
    fn test_package_pattern_generic() {
        let test_cases = vec![
            ("service.impl.userservice", true), // All lowercase with 3+ segments
            ("repository.data.repository", true),
            ("test.package.something", true),
            ("a.b.c", true), // 3 segments minimum
            ("service.impl.UserService", false), // Contains uppercase - won't match (use permissive instead)
            ("my.Class", false), // Only 2 segments
            ("MyClass", false),
        ];

        for (input, should_match) in test_cases {
            let matches = PACKAGE_PATTERN_GENERIC.is_match(input);
            assert_eq!(
                matches, should_match,
                "Generic pattern mismatch for '{}': expected {}, got {}",
                input, should_match, matches
            );
        }
    }

    #[test]
    fn test_package_pattern_permissive() {
        let test_cases = vec![
            ("service.UserService", true),
            ("impl.MyClass", true),
            ("repository.data.Repository", true),
            ("my.Class", false), // Segments too short (< 3 chars)
            ("a.b", false),
            ("MyClass", false),
        ];

        for (input, should_match) in test_cases {
            let matches = PACKAGE_PATTERN_PERMISSIVE.is_match(input);
            assert_eq!(
                matches, should_match,
                "Permissive pattern mismatch for '{}': expected {}, got {}",
                input, should_match, matches
            );
        }
    }

    #[test]
    fn test_exception_pattern() {
        let test_cases = vec![
            ("NullPointerException", true),
            ("IOException", true),
            ("IllegalArgumentException", true),
            ("RuntimeException", true),
            ("SQLException", true),
            ("MyException", true),
            ("Exception", false), // Pattern [A-Z][a-zA-Z0-9]* requires at least one char before "Exception"
            ("MyClass", false),
            ("exception", false), // Must start with uppercase
        ];

        for (input, should_match) in test_cases {
            let matches = EXCEPTION_PATTERN.is_match(input);
            assert_eq!(
                matches, should_match,
                "Exception pattern mismatch for '{}': expected {}, got {}",
                input, should_match, matches
            );
        }
    }

    #[test]
    fn test_stacktrace_pattern() {
        let test_cases = vec![
            ("    at com.example.MyClass.myMethod(MyClass.java:42)", true),
            ("at org.springframework.boot.SpringApplication.<init>(SpringApplication.java:123)", true),
            ("  at com.example.OuterClass$InnerClass.method(OuterClass.java:99)", true),
            ("\tat java.util.ArrayList.add(ArrayList.java:123)", true),
            ("[INFO] Starting application at port 8080", false), // Not a stack trace
            ("at incomplete line", false),
        ];

        for (input, should_match) in test_cases {
            let matches = STACKTRACE_PATTERN.is_match(input);
            assert_eq!(
                matches, should_match,
                "Stacktrace pattern mismatch for '{}': expected {}, got {}",
                input, should_match, matches
            );
        }
    }

    #[test]
    fn test_stacktrace_captures() {
        let input = "    at com.example.MyClass.myMethod(MyClass.java:42)";
        let captures = STACKTRACE_PATTERN.captures(input);
        
        assert!(captures.is_some());
        let caps = captures.unwrap();
        
        assert_eq!(caps.get(1).map(|m| m.as_str()), Some("com.example.MyClass"));
        assert_eq!(caps.get(2).map(|m| m.as_str()), Some("myMethod"));
        assert_eq!(caps.get(3).map(|m| m.as_str()), Some("MyClass.java:42"));
    }
}
