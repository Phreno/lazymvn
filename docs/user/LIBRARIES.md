# LazyMVN Libraries

LazyMVN is organized into reusable libraries that can be used independently or together.

## 📚 Available Libraries

### 1. **java-agent** (`crates/java-agent`)

Java agent for debugging and instrumentation.

**Status**: ✅ Stable  
**Version**: Part of LazyMVN workspace  
**Location**: `crates/java-agent/`

#### Features
- Java agent JAR compilation
- Bytecode instrumentation support
- Debugging capabilities
- Integration with LazyMVN debugger

#### Usage

```toml
[dependencies]
java-agent = { path = "../java-agent" }
```

```rust
use java_agent::build_agent;

// Build the Java agent
build_agent()?;
```

**See Also**: `docs/internal/AGENTS.md`

---

### 2. **log-analyze** (`crates/log-analyze`)

Intelligent log analysis and pattern detection.

**Status**: ✅ Stable  
**Version**: Part of LazyMVN workspace  
**Location**: `crates/log-analyze/`

#### Features
- Error pattern detection
- Stack trace parsing
- Build failure analysis
- Log level filtering
- Pattern matching and highlighting

#### Usage

```toml
[dependencies]
log-analyze = { path = "../log-analyze" }
```

```rust
use log_analyze::{analyze_log, LogEntry, ErrorPattern};

// Analyze log content
let results = analyze_log(&log_content)?;

// Check for errors
if results.has_errors() {
    for error in results.errors() {
        println!("Error: {}", error.message);
    }
}
```

**Key Types**:
- `LogEntry`: Represents a single log line
- `LogAnalyzer`: Main analysis engine
- `ErrorPattern`: Pattern matching for errors
- `AnalysisResult`: Analysis output

**See Also**: `docs/user/LOG_FORMATTING.md`

---

### 3. **log-colorize** (`crates/log-colorize`)

Terminal color formatting for logs.

**Status**: ✅ Stable  
**Version**: Part of LazyMVN workspace  
**Location**: `crates/log-colorize/`

#### Features
- ANSI color support
- Log level color coding
- Syntax highlighting
- Customizable themes
- Performance optimized

#### Usage

```toml
[dependencies]
log-colorize = { path = "../log-colorize" }
```

```rust
use log_colorize::{colorize_log, ColorScheme};

// Colorize log output
let colored = colorize_log(&log_line, ColorScheme::Default);
println!("{}", colored);

// Custom color scheme
let scheme = ColorScheme::custom()
    .error("#FF0000")
    .warning("#FFA500")
    .info("#00FF00")
    .build();
```

**Key Features**:
- ERROR: Red
- WARNING: Yellow
- INFO: Green
- DEBUG: Cyan
- TRACE: Gray

**See Also**: `docs/user/LOG_FORMATTING.md`

---

### 4. **mvn-pom** (`crates/mvn-pom`)

Maven POM.xml parsing and manipulation.

**Status**: ✅ Stable  
**Version**: Part of LazyMVN workspace  
**Location**: `crates/mvn-pom/`

#### Features
- Fast POM parsing
- XML manipulation
- Dependency resolution
- Profile extraction
- Module detection
- Property expansion

#### Usage

```toml
[dependencies]
mvn-pom = { path = "../mvn-pom" }
```

```rust
use mvn_pom::{Pom, parse_pom};

// Parse POM file
let pom = parse_pom("pom.xml")?;

// Get project information
println!("Artifact: {}:{}", pom.group_id, pom.artifact_id);
println!("Version: {}", pom.version);

// List modules
for module in pom.modules() {
    println!("Module: {}", module);
}

// Get profiles
for profile in pom.profiles() {
    println!("Profile: {}", profile.id);
}
```

**Key Types**:
- `Pom`: Represents a parsed POM file
- `Dependency`: Maven dependency
- `Profile`: Maven profile
- `Module`: Maven module

**See Also**: 
- `docs/user/PROFILE_ACTIVATION.md`
- `docs/internal/POM_PARSING.md`

---

## 🏗️ Architecture

```
lazymvn (main binary)
├── java-agent        (Java instrumentation)
├── log-analyze       (Log analysis engine)
├── log-colorize      (Terminal formatting)
└── mvn-pom           (POM parsing)
```

### Dependency Graph

```
lazymvn
  ├─> java-agent
  ├─> log-analyze ─> log-colorize
  └─> mvn-pom
```

## 🔧 Development

### Building All Libraries

```bash
# Build all workspace members
cargo build --workspace

# Build specific library
cargo build -p java-agent
cargo build -p log-analyze
cargo build -p log-colorize
cargo build -p mvn-pom

# Run tests for all libraries
cargo test --workspace

# Run tests for specific library
cargo test -p mvn-pom
```

### Testing

Each library has its own test suite:

```bash
# Test individual library
cd crates/mvn-pom
cargo test

# Run integration tests
cargo test --test '*'

# Run with coverage
cargo tarpaulin --workspace
```

## 📦 Future: Publishing to crates.io

These libraries are designed to be reusable and will be published to crates.io in the future:

**Planned**:
- ✅ `java-agent` - Java agent utilities
- ✅ `log-analyze` - Log analysis toolkit
- ✅ `log-colorize` - Terminal color formatting
- ✅ `mvn-pom` - Maven POM parser

**Timeline**: After stabilization and API review (see `docs/ROADMAP_INDEX.md`)

## 🤝 Contributing

When working with libraries:

1. **Keep them independent**: Libraries should not depend on the main binary
2. **Document public APIs**: All public items need documentation
3. **Write tests**: Aim for >80% coverage
4. **Follow conventions**: Use consistent naming and patterns
5. **Version carefully**: Consider semantic versioning

See `CONTRIBUTING.md` for details.

## 📚 Additional Resources

- **User Guides**: `docs/user/`
- **Internal Docs**: `docs/internal/`
- **Architecture**: `docs/internal/ARCHITECTURE.md`
- **Roadmap**: `docs/ROADMAP_INDEX.md`
