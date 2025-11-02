# LazyMvn Library Extraction Plan 📦

## Analysis Date: 2025-11-01

Based on analysis of the codebase, I've identified **3 high-value libraries** that can be extracted:

---

## 🎯 Library 1: **maven-log-analyzer** (Highest Priority)

### Description
A comprehensive Maven log analysis library for parsing, analyzing, and generating statistics from Maven build logs. This directly addresses your goal of doing statistics over logs.

### Components (Total: ~650 lines)

#### Core Files:
1. **log_patterns.rs** (196 lines)
   - Regex patterns for Java packages, exceptions, stack traces
   - 8 public static patterns (LazyLock for performance)
   - Comprehensive TLD and framework detection

2. **log_analysis.rs** (~320 lines of logic)
   - Package name extraction (3-pass algorithm)
   - False positive filtering
   - Unique package collection
   - Statistics generation

3. **log_parser.rs** (247 lines)
   - ANSI sequence cleaning
   - Log line normalization
   - Basic parsing utilities

### Public API:
```rust
// Pattern matching
pub static PACKAGE_PATTERN_WITH_PREFIX: Regex;
pub static PACKAGE_PATTERN_GENERIC: Regex;
pub static PACKAGE_PATTERN_PERMISSIVE: Regex;
pub static EXCEPTION_PATTERN: Regex;
pub static STACKTRACE_PATTERN: Regex;

// Analysis functions
pub fn extract_package_from_log_line(text: &str, log_format: &str) -> Option<(usize, usize, &str)>;
pub fn is_false_positive(package: &str) -> bool;
pub fn collect_unique_packages(lines: &[String], log_format: &str) -> HashSet<String>;

// Parsing utilities
pub fn clean_log_line(raw: &str) -> Option<String>;

// Future additions for statistics:
pub struct LogStatistics {
    pub total_lines: usize,
    pub unique_packages: HashSet<String>,
    pub exceptions: Vec<String>,
    pub stack_traces: Vec<String>,
    pub log_levels: HashMap<String, usize>, // INFO, DEBUG, ERROR counts
    pub build_duration: Option<Duration>,
    pub test_results: TestStatistics,
}

pub fn analyze_log(content: &str) -> LogStatistics;
pub fn analyze_log_file(path: &Path) -> Result<LogStatistics>;
```

### Use Cases:
- ✅ Extract all Java packages used in a build
- ✅ Identify exceptions and their frequency
- ✅ Track which packages are most active (log volume)
- ✅ Generate build reports
- ✅ Monitor build health over time
- ✅ Detect anomalies in build patterns
- ✅ Compare builds to identify changes

### Dependencies:
```toml
[dependencies]
regex = "1.10"
```

### Value Proposition:
- **Reusable** across any Java/Maven project
- **Statistical analysis ready** - foundation for metrics
- **Well-tested** regex patterns
- **Performance optimized** with LazyLock
- **No UI dependencies** - pure logic

---

## 🎨 Library 2: **maven-log-colorizer** (High Priority)

### Description
TUI colorization library for Maven logs, XML, and Java stack traces. Provides beautiful, semantic highlighting for terminal UIs.

### Components (Total: ~440 lines)

#### Core Files:
1. **log_colorizer.rs** (from log_parser.rs - 244 lines)
   - Keyword-based colorization
   - Package name highlighting
   - Command line detection
   - Log level coloring

2. **xml_formatter.rs** (190 lines)
   - XML syntax highlighting
   - Tag, attribute, value coloring
   - Declaration handling

3. **tests** (included in mod.rs)
   - Comprehensive test suite

### Public API:
```rust
use ratatui::text::Line;

// Log colorization
pub fn colorize_log_line(text: &str) -> Line<'static>;
pub fn colorize_log_line_with_format(text: &str, log_format: Option<&str>) -> Line<'static>;

// XML colorization
pub fn colorize_xml_line(text: &str) -> Line<'static>;

// Future additions:
pub fn colorize_java_code(code: &str) -> Line<'static>;
pub fn colorize_json(json: &str) -> Line<'static>;
pub fn colorize_yaml(yaml: &str) -> Line<'static>;
```

### Use Cases:
- ✅ Beautiful TUI log viewers
- ✅ XML/POM file viewers
- ✅ Stack trace highlighting
- ✅ Build output dashboards
- ✅ Any Ratatui-based Java/Maven TUI

### Dependencies:
```toml
[dependencies]
ratatui = "0.28"
regex = "1.10"
```

### Value Proposition:
- **Ratatui-specific** but reusable
- **Semantic coloring** - not just pretty, but meaningful
- **Extensible** - easy to add new formats
- **Companion to analyzer** - perfect pair

---

## 🔧 Library 3: **maven-wrapper-detector** (Medium Priority)

### Description
Lightweight library for detecting and executing Maven (mvn) or Maven Wrapper (mvnw) with smart fallback logic.

### Components (Total: ~150 lines)

Extracted from `maven/command.rs`:

```rust
pub fn detect_maven_command(project_root: &Path) -> String;
pub fn has_maven_wrapper(project_root: &Path) -> bool;
pub fn get_maven_executable(project_root: &Path) -> PathBuf;
```

### Use Cases:
- ✅ Any tool that needs to run Maven
- ✅ Build scripts
- ✅ CI/CD pipelines
- ✅ IDE integrations

### Dependencies:
```toml
[dependencies]
# stdlib only
```

### Value Proposition:
- **Zero dependencies**
- **Cross-platform** (Windows/Unix)
- **Smart detection** - wrapper preferred
- **Simple API**

---

## 📊 Recommended Extraction Order

### Phase 1: Maven Log Analyzer (Week 1)
**Priority: HIGHEST** ⭐⭐⭐⭐⭐

1. Create new crate `maven-log-analyzer`
2. Extract and refactor:
   - `utils/log_patterns.rs`
   - `utils/log_analysis.rs`
   - `utils/text/log_parser.rs` (parsing only)
3. Add statistics module:
   - `LogStatistics` struct
   - `analyze_log()` function
   - Aggregation utilities
4. Write comprehensive examples
5. Publish to crates.io

**Estimated effort**: 8-12 hours

### Phase 2: Maven Log Colorizer (Week 1-2)
**Priority: HIGH** ⭐⭐⭐⭐

1. Create new crate `maven-log-colorizer`
2. Extract:
   - `utils/text/log_parser.rs` (colorization)
   - `utils/text/xml_formatter.rs`
3. Add more colorizers (optional):
   - JSON
   - YAML
   - Java source code
4. Write TUI examples with ratatui
5. Publish to crates.io

**Estimated effort**: 4-6 hours

### Phase 3: Maven Wrapper Detector (Week 2)
**Priority: MEDIUM** ⭐⭐⭐

1. Create new crate `maven-wrapper-detector`
2. Extract wrapper detection logic from `maven/command.rs`
3. Add tests for various project structures
4. Publish to crates.io

**Estimated effort**: 2-3 hours

---

## 🏗️ Implementation Strategy

### Workspace Structure

```
lazymvn/
├── Cargo.toml                    # Workspace root
├── lazymvn/                      # Main application
│   ├── Cargo.toml
│   └── src/
├── crates/
│   ├── maven-log-analyzer/       # Library 1
│   │   ├── Cargo.toml
│   │   ├── README.md
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── patterns.rs
│   │   │   ├── analysis.rs
│   │   │   ├── statistics.rs
│   │   │   └── parser.rs
│   │   └── examples/
│   │       └── analyze_build_log.rs
│   │
│   ├── maven-log-colorizer/      # Library 2
│   │   ├── Cargo.toml
│   │   ├── README.md
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── log.rs
│   │   │   └── xml.rs
│   │   └── examples/
│   │       └── colorize_log.rs
│   │
│   └── maven-wrapper-detector/   # Library 3
│       ├── Cargo.toml
│       ├── README.md
│       ├── src/
│       │   └── lib.rs
│       └── examples/
│           └── detect_maven.rs
```

### Root Cargo.toml
```toml
[workspace]
members = [
    "lazymvn",
    "crates/maven-log-analyzer",
    "crates/maven-log-colorizer",
    "crates/maven-wrapper-detector",
]
resolver = "2"

[workspace.dependencies]
regex = "1.10"
ratatui = "0.28"
```

---

## 💡 Additional Statistics Features for maven-log-analyzer

Once extracted, the analyzer can be extended with:

### 1. Build Metrics
```rust
pub struct BuildMetrics {
    pub duration: Duration,
    pub modules_built: usize,
    pub tests_run: usize,
    pub tests_failed: usize,
    pub tests_skipped: usize,
    pub compilation_errors: usize,
    pub warnings: usize,
}
```

### 2. Package Activity Analysis
```rust
pub struct PackageActivity {
    pub package_name: String,
    pub log_lines: usize,
    pub log_levels: HashMap<LogLevel, usize>,
    pub exceptions: Vec<String>,
}

pub fn analyze_package_activity(logs: &[String]) -> Vec<PackageActivity>;
```

### 3. Time Series Analysis
```rust
pub struct TimeSeriesPoint {
    pub timestamp: DateTime<Utc>,
    pub log_level: LogLevel,
    pub package: Option<String>,
}

pub fn extract_time_series(logs: &[String]) -> Vec<TimeSeriesPoint>;
```

### 4. Exception Frequency
```rust
pub struct ExceptionStats {
    pub exception_type: String,
    pub count: usize,
    pub packages: HashSet<String>,
    pub sample_stack_traces: Vec<String>,
}

pub fn analyze_exceptions(logs: &[String]) -> Vec<ExceptionStats>;
```

### 5. Dependency Analysis
```rust
pub fn extract_dependencies(logs: &[String]) -> Vec<Dependency>;
pub fn build_dependency_graph(logs: &[String]) -> DependencyGraph;
```

---

## 🎯 Success Criteria

### maven-log-analyzer
- [ ] Can parse Maven build logs
- [ ] Extracts packages accurately (>95% precision)
- [ ] Generates comprehensive statistics
- [ ] Performance: <100ms for 10K line logs
- [ ] Zero dependencies on UI libraries
- [ ] Published on crates.io
- [ ] README with examples
- [ ] CI/CD setup

### maven-log-colorizer
- [ ] Beautiful semantic highlighting
- [ ] Works with ratatui TUI apps
- [ ] Supports XML, logs, stack traces
- [ ] Performance: <1ms per line
- [ ] Published on crates.io
- [ ] Screenshot examples
- [ ] Demo application

### maven-wrapper-detector
- [ ] Detects mvnw on Windows/Unix
- [ ] Fallback to system mvn
- [ ] Zero dependencies
- [ ] Cross-platform tests
- [ ] Published on crates.io

---

## 📝 Next Steps

1. **Approve this plan** ✅
2. **Start with maven-log-analyzer** (highest value for your statistics goal)
3. **Create workspace structure**
4. **Extract and test code**
5. **Add statistics features**
6. **Publish libraries**

Would you like me to proceed with Phase 1 (maven-log-analyzer)?

