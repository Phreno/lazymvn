# maven-log-analyzer

[![Crates.io](https://img.shields.io/crates/v/maven-log-analyzer.svg)](https://crates.io/crates/maven-log-analyzer)
[![Documentation](https://docs.rs/maven-log-analyzer/badge.svg)](https://docs.rs/maven-log-analyzer)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Rust library for analyzing Maven build logs - extract packages, exceptions, and build metrics.

## Features

- **Package Detection**: Extract Java package names from log lines with high precision
  - Supports full packages (com.example.Service)
  - Handles truncated logger names (%c{1}, %c{2})
  - Three-tier matching strategy for maximum accuracy
  
- **Exception Detection**: Identify Java exceptions in logs
  
- **Stack Trace Parsing**: Parse and extract information from Java stack traces

- **Log Normalization**: Clean ANSI escape sequences and normalize log content

- **Statistical Analysis**: Extract unique packages for build statistics

- **Zero UI Dependencies**: Pure regex-based analysis, no dependencies on terminal or UI libraries

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
maven-log-analyzer = "0.1"
```

### Extract packages from log lines

```rust
use maven_log_analyzer::analysis;

let log_line = "[INFO] com.example.service.UserService - Processing request";
let log_format = "[%p] %c - %m%n";

if let Some((_start, _end, package)) = analysis::extract_package_from_log_line(log_line, log_format) {
    println!("Found package: {}", package);
    // Output: Found package: com.example.service.UserService
}
```

### Extract all unique packages from logs

```rust
use maven_log_analyzer::analysis;

let logs = vec![
    "[INFO] com.example.Service - Message 1".to_string(),
    "[DEBUG] org.springframework.Application - Message 2".to_string(),
    "[ERROR] com.example.Service - Message 3".to_string(),
];

let packages = analysis::extract_unique_packages(&logs, Some("[%p] %c - %m%n"));
// Returns: ["com.example.Service", "org.springframework.Application"]
```

### Clean ANSI sequences from logs

```rust
use maven_log_analyzer::parser;

let raw = "\u{1b}[32m[INFO]\u{1b}[0m Message";
let cleaned = parser::clean_log_line(raw);
// Returns: Some("[INFO] Message")
```

### Detect exceptions and stack traces

```rust
use maven_log_analyzer::patterns::{EXCEPTION_PATTERN, STACKTRACE_PATTERN};

// Check for exceptions
if EXCEPTION_PATTERN.is_match("NullPointerException occurred") {
    println!("Exception found!");
}

// Parse stack trace
let stack_line = "    at com.example.MyClass.myMethod(MyClass.java:42)";
if let Some(captures) = STACKTRACE_PATTERN.captures(stack_line) {
    let class = captures.get(1).map(|m| m.as_str()).unwrap();
    let method = captures.get(2).map(|m| m.as_str()).unwrap();
    let location = captures.get(3).map(|m| m.as_str()).unwrap();
    println!("Class: {}, Method: {}, Location: {}", class, method, location);
}
```

## Package Detection Strategy

The library uses a three-tier matching strategy for maximum accuracy:

1. **Prefix-based** (most precise): Matches packages starting with known TLDs or Java namespaces
   - Examples: `com.example.Service`, `org.springframework.Application`
   
2. **Generic** (3+ segments): Matches any lowercase package with 3+ segments
   - Examples: `service.impl.userservice`, `repository.data.handler`
   
3. **Permissive** (log context): For truncated logger names when log level is detected
   - Examples: `service.UserService`, `impl.MyClass`

## False Positive Filtering

The library automatically filters out common false positives:
- File extensions (`.xml`, `.json`, `.properties`, etc.)
- URL patterns (`http.`, `https.`, `www.`)
- Ambiguous short patterns (`my.Class`, `file.path`)

## Supported Log Formats

The library works with any log format but provides best results when log format is specified:
- Log4j patterns: `[%p] %c - %m%n`
- Logback patterns: `%d{HH:mm:ss.SSS} [%level] %logger{36} - %msg%n`
- Custom formats: Any format containing Java package names

## Testing

The library includes comprehensive tests for all functionality:

```bash
cd crates/maven-log-analyzer
cargo test
```

## Future Enhancements

- Build metrics extraction (duration, test counts, errors)
- Package activity analysis (frequency, error rates)
- Exception statistics aggregation
- Dependency graph generation from logs
- Timeline analysis

## License

MIT

## Contributing

Contributions are welcome! This library is part of the [lazymvn](https://github.com/Phreno/lazymvn) project.
