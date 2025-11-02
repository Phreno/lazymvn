# LazyMVN Architecture

## 🏗️ High-Level Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     LazyMVN CLI                         │
│                  (Terminal UI/TUI)                      │
└────────────────┬────────────────────────────────────────┘
                 │
        ┌────────┴────────┐
        │                 │
        v                 v
┌───────────────┐  ┌──────────────┐
│  Configuration│  │   History    │
│    Manager    │  │   Manager    │
└───────┬───────┘  └──────┬───────┘
        │                 │
        v                 v
┌─────────────────────────────────┐
│      Maven Executor Core        │
│   (Process Management)          │
└─────────┬──────────────┬────────┘
          │              │
          v              v
    ┌─────────┐    ┌──────────┐
    │  Cache  │    │   Logs   │
    │ System  │    │ Analysis │
    └─────────┘    └──────────┘
```

## 📦 Module Structure

### Core Modules

#### 1. **UI Layer** (`src/ui/`)
- **TUI Framework**: Ratatui-based terminal interface
- **Input Handling**: Keyboard shortcuts and navigation
- **Rendering**: Real-time log display and status updates
- **Tabs Management**: Multiple Maven sessions

**Key Files**:
- `src/ui/mod.rs` - Main UI orchestration
- `src/ui/tabs.rs` - Tab management
- `src/ui/input.rs` - Input handling
- `src/ui/render.rs` - Rendering logic

#### 2. **Configuration** (`src/config/`)
- **TOML Parsing**: `lazymvn.toml` configuration
- **Profile Management**: Maven profiles
- **Custom Goals**: User-defined goals and flags
- **Live Reload**: Auto-reload on config changes

**Key Files**:
- `src/config/mod.rs` - Configuration loading
- `src/config/live_reload.rs` - File watching
- `src/config/profiles.rs` - Profile handling

#### 3. **Maven Integration** (`src/maven/`)
- **Process Execution**: Maven command execution
- **POM Parsing**: Using `mvn-pom` library
- **Module Detection**: Multi-module project support
- **Profile Activation**: Dynamic profile switching

**Key Files**:
- `src/maven/executor.rs` - Maven process execution
- `src/maven/parser.rs` - POM parsing integration
- `src/maven/profiles.rs` - Profile management

#### 4. **Caching System** (`src/cache/`)
- **Goal Cache**: Maven goals and targets
- **Profile Cache**: Available profiles
- **Module Cache**: Project structure
- **Invalidation**: Smart cache expiration

**Key Files**:
- `src/cache/mod.rs` - Cache orchestration
- `src/cache/storage.rs` - Cache persistence
- `src/cache/invalidation.rs` - Cache refresh logic

#### 5. **History Management** (`src/history/`)
- **Command History**: Previously run Maven commands
- **Session Tracking**: Multi-project history
- **Deduplication**: Smart history cleanup
- **Context Switching**: Per-project history

**Key Files**:
- `src/history/mod.rs` - History core
- `src/history/store.rs` - Persistence
- `src/history/context.rs` - Context switching

#### 6. **Log Management** (`src/logs/`)
- **Analysis**: Using `log-analyze` library
- **Colorization**: Using `log-colorize` library
- **Rotation**: Automatic log rotation
- **Filtering**: Log level filtering

**Key Files**:
- `src/logs/analyzer.rs` - Log analysis integration
- `src/logs/formatter.rs` - Log formatting
- `src/logs/rotation.rs` - Log rotation

### Library Crates

#### 1. **java-agent** (`crates/java-agent/`)
```
java-agent/
├── src/
│   ├── lib.rs          # Agent API
│   ├── builder.rs      # JAR builder
│   └── config.rs       # Agent configuration
├── java/               # Java source code
│   └── LazyAgent.java
├── Cargo.toml
└── build.rs            # Build script
```

**Purpose**: Compile and manage Java debugging agent

#### 2. **log-analyze** (`crates/log-analyze/`)
```
log-analyze/
├── src/
│   ├── lib.rs          # Public API
│   ├── analyzer.rs     # Log analysis engine
│   ├── patterns.rs     # Error patterns
│   └── entry.rs        # Log entry types
├── tests/
│   └── integration.rs
└── Cargo.toml
```

**Purpose**: Parse and analyze Maven build logs

#### 3. **log-colorize** (`crates/log-colorize/`)
```
log-colorize/
├── src/
│   ├── lib.rs          # Public API
│   ├── colors.rs       # Color schemes
│   ├── formatter.rs    # ANSI formatting
│   └── themes.rs       # Predefined themes
├── tests/
│   └── color_test.rs
└── Cargo.toml
```

**Purpose**: Apply ANSI colors to log output

#### 4. **mvn-pom** (`crates/mvn-pom/`)
```
mvn-pom/
├── src/
│   ├── lib.rs          # Public API
│   ├── parser.rs       # XML parsing
│   ├── model.rs        # POM data model
│   ├── dependency.rs   # Dependency handling
│   └── profile.rs      # Profile handling
├── tests/
│   ├── fixtures/       # Test POM files
│   └── integration.rs
└── Cargo.toml
```

**Purpose**: Parse and manipulate Maven POM files

## 🔄 Data Flow

### 1. Startup Flow

```
User runs `lazymvn`
    ↓
Load Configuration (lazymvn.toml)
    ↓
Parse POM.xml (mvn-pom)
    ↓
Load Cache (goals, profiles, modules)
    ↓
Load History (previous commands)
    ↓
Initialize UI (TUI)
    ↓
Display Menu
```

### 2. Command Execution Flow

```
User selects goal
    ↓
Validate cache freshness
    ↓
Build Maven command
    ↓
Spawn Maven process
    ↓
Stream output → UI
    ↓
Analyze logs (log-analyze)
    ↓
Colorize output (log-colorize)
    ↓
Display in real-time
    ↓
Save to history
    ↓
Update cache if needed
```

### 3. Cache Invalidation Flow

```
POM.xml changes detected
    ↓
Invalidate affected caches
    ↓
Re-parse POM (mvn-pom)
    ↓
Rebuild cache entries
    ↓
Notify user (optional)
```

## 🗃️ Data Storage

### Configuration Files
```
~/.config/lazymvn/
└── lazymvn.toml          # Main configuration
```

### Data Directory
```
~/.local/share/lazymvn/
├── cache/
│   ├── goals.json        # Cached Maven goals
│   ├── profiles.json     # Cached profiles
│   └── modules.json      # Cached modules
├── history/
│   └── commands.json     # Command history
└── logs/
    ├── debug.log         # Debug logs
    └── sessions/         # Session logs
        └── 20251102_*.log
```

## 🔧 Key Technologies

### UI & Terminal
- **Ratatui**: Terminal UI framework
- **Crossterm**: Cross-platform terminal manipulation
- **ANSI Colors**: Terminal colorization

### Data Formats
- **TOML**: Configuration files (via `toml`)
- **JSON**: Cache and history (via `serde_json`)
- **XML**: POM parsing (via `quick-xml`)

### File System
- **Notify**: File system watching for live reload
- **Dirs**: XDG directory standard compliance

### Process Management
- **Tokio**: Async runtime for Maven processes
- **Process spawning**: `std::process::Command`

## 🔌 Extension Points

### 1. Custom Goals
Users can define custom goals in `lazymvn.toml`:

```toml
[[custom_goals]]
name = "Full Build"
command = "clean install -DskipTests"
description = "Clean and install without tests"
```

### 2. Profiles
Custom profile activation logic can be added via configuration:

```toml
[[profiles]]
name = "dev"
auto_activate = true
properties = { "env" = "development" }
```

### 3. Log Analyzers
Custom log analysis patterns can be defined:

```toml
[[log_patterns]]
pattern = "BUILD FAILURE"
action = "highlight"
color = "red"
```

## 🚀 Performance Optimizations

### 1. Caching Strategy
- **Goal cache**: Avoid repeated Maven introspection
- **POM cache**: Skip parsing if file unchanged
- **Module cache**: Remember project structure

### 2. Async Processing
- **Non-blocking UI**: Maven runs in background thread
- **Streaming logs**: Real-time output without buffering
- **Parallel cache updates**: Update multiple caches concurrently

### 3. Memory Management
- **Log rotation**: Automatic cleanup of old logs
- **History limits**: Cap history entries
- **Cache expiration**: Remove stale cache entries

## 🔐 Security Considerations

1. **Command Injection**: All Maven commands are validated
2. **File Permissions**: Proper permissions for cache/logs
3. **Environment Variables**: Sanitized before Maven execution
4. **Agent Security**: Java agent runs with limited permissions

## 📊 Monitoring & Debugging

### Debug Mode
Enable debug logging:
```bash
export RUST_LOG=debug
lazymvn
```

### Log Locations
- **Debug logs**: `~/.local/share/lazymvn/logs/debug.log`
- **Session logs**: `~/.local/share/lazymvn/logs/sessions/`

### Performance Profiling
```bash
# Build with profiling
cargo build --release --features profiling

# Run with timing
LAZYMVN_PROFILE=1 lazymvn
```

## 🔮 Future Architecture Improvements

See `docs/ROADMAP_INDEX.md` for planned enhancements:

1. **Plugin System**: Extensible plugin architecture
2. **Remote Maven**: Support for remote Maven repositories
3. **Analytics**: Build time tracking and metrics
4. **Notifications**: Desktop notifications for build completion
5. **Multi-Maven**: Support multiple Maven versions

## 📚 Related Documentation

- **Libraries**: `docs/user/LIBRARIES.md`
- **Configuration**: `docs/user/README.md`
- **Development**: `CONTRIBUTING.md`
- **Roadmap**: `docs/ROADMAP_INDEX.md`
