# LazyMVN

**LazyMVN** is a **cross-platform terminal UI (TUI)** for interacting with **Maven** projects efficiently without leaving the terminal.
Inspired by *LazyGit*, it provides a clean, keyboard-driven interface to build, test, and manage Maven projects via a **single Rust binary** with no external dependencies.

## Warning

I built this project mainly for myself, as I spend most of my time working in the terminal and wanted a tool tailored to my workflow.
I wrote it in Rust mostly for fun — and because my skills are limited, I rely heavily on AI assistance and improvising as I learned.

It’s a personal experiment more than a polished product, but I’m quite happy with how it turned out.
That said, there is no warranty or guarantee — use it entirely at your own risk.

## Acknowledgment

LazyMVN draws strong inspiration from LazyGit by Jesse Duffield.
I want to credit both the project and its author for the idea and for shaping how I think about terminal-based interfaces.

## Features

### LazyGit-Style Interface
- **Dedicated numbered view blocks**: Projects [0], Modules [2], Profiles [3], Flags [4], Output [0]
- **Quick navigation**: Switch between views instantly with number keys or arrow keys (←/→)
- **Mouse support**: Click on any pane to focus it, click on items to select them
- **Simultaneous display**: All views visible at once for better context
- **Clean separation**: Output pane on the right, selection blocks on the left
- **Adaptive layout**: Automatically adjusts to terminal size
  - **Output-priority design**: Ensures logs are always readable with at least 150 chars
  - Single-column mode when output would be too narrow (< 150 chars in two-column)
  - Two-column mode for wide terminals (>= 190 columns)
  - Focus-driven expansion for short terminals (< 30 rows)
  - Perfect for split-screen development

### Project Support
- **Single-module projects**: Automatically detected, displayed as "(root project)"
- **Multi-module projects**: Lists all modules from the `<modules>` section
- **Smart caching**: Remembers project structure and tracks POM changes
- **Recent projects**: Track up to 20 recently opened Maven projects
- **Quick switching**: Switch between projects with `Ctrl+R` without restarting
- **Smart fallback**: If no POM is found, automatically loads the most recent project

### Multi-Tab Support
- **Multiple projects simultaneously**: Open up to 10 projects in separate tabs
- **Visual tab bar**: Shows all open projects with active tab highlighting
- **Tab indicators**: Display current tab position (e.g., "1/3")
- **Independent state**: Each tab maintains its own module selection, profiles, flags, and output
- **Quick navigation**: Switch between tabs with `Ctrl+Left`/`Ctrl+Right`
- **Tab management**: Create new tabs with `Ctrl+T`, close tabs with `Ctrl+W`
- **Process isolation**: Each tab can run its own Maven process independently
- **Auto-cleanup**: Automatically saves preferences and kills processes when closing tabs

### Maven Operations
- Execute common Maven commands: `clean`, `compile`, `test`, `package`, `install`, `dependency:tree`
- **Run Spring Boot applications** with `s` key
- Module-scoped builds using `-pl` flag (multi-module projects)
- Build combinations: `clean install` with one keystroke
- Kill running processes with `Escape` key

### Spring Boot Support
- **Intelligent starter detection**: Scans for `*Application.java`, `*Main.java`, and `@SpringBootApplication`
- **Fuzzy search selection**: Type to filter potential main classes
- **Cached starters**: Remember your main classes per project
- **Multiple starters**: Support for API, Admin, Batch, etc.
- **Manager interface**: `Ctrl+Shift+S` to view, run, and manage cached starters
- **Smart launch strategy**: Auto-detects `spring-boot:run` vs `exec:java` before launching (no failures!)
  - Analyzes effective POM for Spring Boot plugin and packaging
  - Falls back to `exec:java` if needed
  - Configurable modes: `auto` (default), `force-run`, `force-exec`
  - See [docs/user/SPRING_BOOT_LAUNCHER.md](docs/user/SPRING_BOOT_LAUNCHER.md) for details

### Profiles & Flags
- Toggle Maven profiles interactively
- Enable/disable build flags:
  - `--also-make` - Build module dependencies
  - `--also-make-dependents` - Build dependent modules
  - `-DskipTests` - Skip test execution
  - `--update-snapshots` - Force snapshot updates
  - `--offline` - Work offline
  - `--fail-fast` - Stop at first failure
- **Per-module preferences**: Active profiles and flags are automatically saved per module
  - Remembered across sessions
  - Automatically restored when switching modules

### Output & Navigation
- Real-time Maven output display with color-coded log levels
- Search through output with regex support (`/` to search, `n`/`N` to navigate)
- Scroll through output with arrows, Page Up/Down, or mouse wheel
- Automatic output scrolling to latest content
- Responsive UI during command execution with progress indicator

### Configuration
- Auto-detect Maven settings from project or `~/.m2/settings.xml`
- Optional project-specific configuration via `lazymvn.toml`
- Support for custom Maven wrapper scripts (`mvnw`)
- **Persistent state**: 
  - Module-specific profiles and flags in `~/.config/lazymvn/preferences/`
  - Recent projects in `~/.config/lazymvn/recent.json`
  - Spring Boot starters in `~/.config/lazymvn/starters/`
- Global configuration in `~/.config/lazymvn/` (Linux/macOS) or `%APPDATA%\lazymvn\` (Windows)
- Recent projects list stored in `recent.json` (automatically maintained)
- **Automatic log rotation**: Debug and error logs are rotated when they exceed 5 MB, keeping up to 5 backups per file (max ~60 MB total). Old rotated logs are cleaned up after 30 days. See [Log Rotation](docs/user/LOG_ROTATION.md) for details.

### Technical Stack

- **Language**: Rust (Edition 2024)
- **Architecture**: Modular design with 22+ specialized modules for maintainability and code quality
- **Terminal UI**: [ratatui](https://github.com/ratatui-org/ratatui) for rendering
- **XML Parsing**: [quick-xml](https://github.com/tafia/quick-xml) for POM processing
- **Terminal Backend**: [crossterm](https://github.com/crossterm-rs/crossterm) for cross-platform terminal control
- **Config**: [toml](https://github.com/toml-rs/toml) for configuration file parsing
- **Performance**: Caching system to avoid repeated POM parsing


## Key Bindings

### Navigation
| Key | Action |
|-----|--------|
| `←` / `→` | Cycle focus between all panes (Projects → Modules → Profiles → Flags → Output) |
| `↑` / `↓` | Move selection in current list pane / Scroll output |
| `Page Up` / `Page Down` | Scroll output by pages |
| `Home` / `End` | Jump to start/end of output |
| `Ctrl+R` | Show recent projects and switch to a different project |
| `Ctrl+E` | Edit configuration file (lazymvn.toml) - **changes are applied immediately after editor closes** |
| `Ctrl+K` | Refresh caches (profiles and starters) - **forces reload from Maven and rescans dependencies** |
| `Ctrl+G` | Show custom goals popup - **execute configured Maven plugin goals** |
| **Mouse** | Click on pane to focus it, click on item to select it |

### Tab Management
| Key | Action |
|-----|--------|
| `Ctrl+T` | Create new tab (opens recent projects popup) |
| `Ctrl+W` | Close current tab (cannot close last tab) |
| `Ctrl+←` | Switch to previous tab |
| `Ctrl+→` | Switch to next tab |

### Views
| Key | Action |
|-----|--------|
| `0` | Focus Output pane |
| `1` | Focus Projects pane |
| `2` | Focus Modules pane |
| `3` | Focus Profiles pane |
| `4` | Focus Flags pane |

### Maven Commands
| Key | Action | Maven Command |
|-----|--------|---------------|
| `b` | Build | `clean install` |
| `c` | Compile | `compile` |
| `C` | Clean | `clean` |
| `k` | Package | `package` |
| `t` | Test | `test` |
| `i` | Install | `install` |
| `s` | **Start** (Spring Boot) | `spring-boot:run` |
| `d` | Dependencies | `dependency:tree` |
| `Esc` | Kill running process | - |

### Spring Boot
| Key | Action |
|-----|--------|
| `s` | Run preferred/cached starter (or show selector) |
| `Ctrl+Shift+S` | Open starter manager |

### Workflow Management
| Key | Action |
|-----|--------|
| `Ctrl+F` | Show favorites (saved command configurations) |
| `Ctrl+S` | Save current configuration as a favorite |
| `Ctrl+H` | Show command history |

### Selection & Search
| Key | Action |
|-----|--------|
| `Space` or `Enter` | Toggle selection (profiles/flags) |
| `/` | Start search in output |
| `n` | Next search match |
| `N` | Previous search match |
| `y` | **Yank** (copy) output to clipboard |
| `Y` (Shift+Y) | **Yank Debug Report** - Copy comprehensive debug info (version, logs, config, all tabs output) |
| `Esc` | Exit search mode |

### General
| Key | Action |
|-----|--------|
| `?` | Show help popup with all keybindings |
| `q` | Quit lazymvn |


### Requirements

- Rust 1.70+ (for building from source)
- Maven 3.x or Maven wrapper (`mvnw`) in your project

## Usage

### Basic Usage

Navigate to any Maven project directory and run:

```bash
lazymvn
```

**Command-line Options:**

```bash
lazymvn [OPTIONS]

Options:
  -d, --debug              Enable debug logging to ~/.local/share/lazymvn/logs/debug.log
  -p, --project <PROJECT>  Path to the Maven project
      --force-run          Force spring-boot:run for launching (overrides auto)
      --force-exec         Force exec:java for launching (overrides auto)
  -h, --help               Print help
```

**Examples:**

```bash
# Launch with auto-detection (default)
lazymvn

# Force spring-boot:run strategy
lazymvn --force-run

# Force exec:java strategy  
lazymvn --force-exec

# Open a specific project
lazymvn --project /path/to/maven/project

# Debug mode with force-run
lazymvn --debug --force-run
```

**Smart Project Detection:**
- If a `pom.xml` is found in the current directory or parent directories, LazyMVN loads that project
- If no POM is found, LazyMVN automatically loads your most recently used Maven project
- If no recent projects exist, you'll see a helpful error message with instructions

LazyMVN automatically detects your project structure:

**Single-module projects:**
```
┌Modules──────────────┐┌Output:─────────────┐
│>> (root project)    ││ Run a command...   │
└─────────────────────┘└────────────────────┘
```

**Multi-module projects:**
```
┌Modules──────────────┐┌Output: library─────┐
│>> library           ││ Run a command...   │
│   app               ││                    │
└─────────────────────┘└────────────────────┘
```

### Multi-Project Workflow

LazyMVN supports working with multiple Maven projects simultaneously using tabs:

**Opening Multiple Projects:**

1. Launch LazyMVN with your first project
2. Press `Ctrl+T` to create a new tab
3. Select a project from the recent projects list (or it opens automatically)
4. Repeat to open up to 10 projects

**Visual Tab Bar:**

When you have multiple tabs open, a tab bar appears at the top:

```
 [1] my-api │ 2 admin-service │ 3 batch-jobs  (1/3) 
┌─────────────────────────────────────────────────┐
│  Projects, Modules, Profiles, Flags, Output...  │
└─────────────────────────────────────────────────┘
```

- `[1]` indicates the active tab (with brackets)
- `2`, `3` show inactive tabs
- `(1/3)` shows your position (tab 1 of 3)
- `│` separates tabs
- Each tab shows the project directory name

**Tab Features:**

- **Independent state**: Each tab has its own selected module, active profiles, enabled flags, and command output
- **Concurrent processes**: Run Maven commands in multiple tabs simultaneously (e.g., run API in tab 1, tests in tab 2)
- **Quick switching**: Use `Ctrl+Left` / `Ctrl+Right` to navigate between tabs
- **Auto-save**: Closing a tab saves preferences and kills any running Maven process
- **Protection**: Cannot close the last remaining tab (prevents accidental exit)

**Typical Workflows:**

1. **Microservices development**: Open API, Admin, and Worker services in separate tabs
2. **Frontend + Backend**: Run backend in one tab, frontend build watch in another
3. **Multi-project testing**: Run tests in parallel across different projects
4. **Comparison**: Compare dependency trees or build outputs side-by-side

### Command-line Options

```
lazymvn [OPTIONS]

Options:
  -d, --debug              Enable debug logging to ~/.local/share/lazymvn/logs/debug.log
  -p, --project <PATH>     Path to Maven project (defaults to current directory)
  -h, --help               Print help information
```

### Demo Projects

Two demonstration projects are included in the `demo/` folder for testing and learning:

**Multi-module project** (`demo/multi-module/`):
- Parent POM with two modules: `library` and `app`
- Tests module selection and scoped builds
- Includes Maven profiles and settings examples

**Single-module project** (`demo/single-module/`):
- Simple calculator application with JUnit tests
- No modules section in POM
- Tests single-module project handling

Try them out:
```bash
cd demo/multi-module
lazymvn

cd ../single-module
lazymvn
```

### Maven Settings

LazyMVN automatically searches for Maven settings in this order:
1. `lazymvn.toml` configuration (if specified)
2. `settings.xml` in project directory
3. `maven_settings.xml` in project directory
4. `~/.m2/settings.xml` in user home directory

### Configuration File

Create a `lazymvn.toml` in your project root for custom settings:

```toml
# Optional: Custom Maven settings file path
maven_settings = "./custom-settings.xml"

# Optional: Custom Maven flags (appear in Flags panel)
[maven]
custom_flags = [
  { name = "Enable feature X", flag = "-Dfeature.x=true" },
  { name = "Development mode", flag = "-Dspring.profiles.active=dev", enabled = true },
  { name = "Skip integration tests", flag = "-DskipITs=true" },
]
```

See [examples/](examples/) for more configuration options.

**Live Configuration Reload:**

Press `Ctrl+E` to edit the configuration file in your system editor (`$EDITOR`, `$VISUAL`, or platform default). When you save and close the editor, **changes are automatically applied** without restarting lazymvn. This includes:
- Maven settings path
- Launch mode (auto/force-run/force-exec)
- Watch configuration (enable/disable file watching)
- Notification settings
- Output buffer settings
- Logging configuration
- Custom Maven flags

The application will log detected changes and recreate the file watcher if watch settings were modified.

### Debug Logging

When troubleshooting issues, enable detailed debug logging:

```bash
lazymvn --debug
```

This creates a `debug.log` file in `~/.local/share/lazymvn/logs/` with timestamped entries at INFO, DEBUG, and ERROR levels. The log file doesn't interfere with the TUI and can be monitored in a separate terminal:

```bash
tail -f ~/.local/share/lazymvn/logs/debug.log
```

**Debug logs include:**
- Module discovery and parsing
- POM hash calculations and cache operations
- Maven command construction
- Profile and flag selections
- Search operations
- Key event handling

## How It Works

### Project Discovery
1. Searches for `pom.xml` starting from current directory, walking up the tree
2. Parses the POM to extract `<modules>` section
3. If no modules found, treats project as single-module (root project)
4. Caches results with POM hash to detect changes

### Module Caching
Cache is stored in `~/.config/lazymvn/cache.json` and includes:
- Project root path
- List of modules (or `["."]` for single-module projects)
- POM content hash for change detection

When you return to a project:
- Cache is validated against current POM hash
- If POM changed, modules are re-parsed
- If unchanged, cached modules are used instantly

### Profile & Starter Caching
To improve startup performance, LazyMVN also caches:

**Profiles Cache** (`~/.config/lazymvn/profiles/<project-hash>.json`):
- Maven profiles detected via `mvn help:all-profiles`
- Loaded instantly on subsequent launches
- Press `Ctrl+K` to refresh if profiles change

**Starters Cache** (`~/.config/lazymvn/starters/<project-hash>.json`):
- Spring Boot main classes discovered by scanning dependencies
- Auto-scanned on first load if cache is empty
- Allows instant starter selection without rescanning
- Press `Ctrl+K` to refresh if dependencies change

> **Tip**: Use `Ctrl+K` at any time to force a refresh of both caches. This is useful after adding new Maven profiles or Spring Boot dependencies.

### Command Execution
For **multi-module projects**:
```bash
mvn -pl <selected-module> [profiles] [flags] <command>
```

For **single-module projects**:
```bash
mvn [profiles] [flags] <command>
```

Commands are executed via:
- `./mvnw` if Maven wrapper exists in project root
- `mvn` system command otherwise

## Project Structure

```
lazymvn/
├── src/
│   ├── main.rs           # Entry point and TUI setup
│   ├── config.rs         # Configuration loading
│   ├── project.rs        # Module discovery and caching
│   ├── maven.rs          # Maven command execution
│   ├── tui.rs            # Main TUI coordination
│   ├── utils.rs          # Log parsing utilities
│   └── ui/
│       ├── keybindings.rs  # Key event handling
│       ├── state.rs        # Application state
│       ├── panes.rs        # UI rendering
│       ├── search.rs       # Search functionality
│       └── theme.rs        # Color schemes
├── demo/
│   ├── multi-module/     # Demo multi-module project
│   └── single-module/    # Demo single-module project
└── Cargo.toml            # Dependencies and metadata
```

## Project Resources

### Documentation
- **[docs/](docs/)** - Comprehensive documentation hub
  - **[User Documentation](docs/user/README.md)**: Feature guides and tutorials
  - **[Internal Documentation](docs/internal/README.md)**: Implementation details and architecture
  - **[Roadmap](docs/ROADMAP_EXECUTIVE_SUMMARY.md)**: Project vision and planning
  - **[Quick Wins](docs/QUICK_WINS.md)**: High-impact improvements for contributors

### Configuration Examples
- **[examples/](examples/)** - Configuration file examples
  - Complete examples for various use cases (Spring Boot, watch mode, logging, etc.)
  - See [examples/README.md](examples/README.md) for all available examples

### Test Scripts
- **[scripts/](scripts/)** - Test and validation scripts
  - Environment setup validation, feature tests, and integration tests
  - See [scripts/README.md](scripts/README.md) for usage instructions

### Development Guidelines
- **[docs/internal/AGENTS.md](docs/internal/AGENTS.md)** - Coding guidelines and project structure for contributors and AI agents
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Contribution process and code of conduct
- **[docs/internal/VERSIONING.md](docs/internal/VERSIONING.md)** - Versioning strategy and release process


### Development Setup

```bash
git clone https://github.com/Phreno/lazymvn.git
cd lazymvn
cargo build
cargo test
```

### Testing

```bash
# Run all tests
cargo test

# Test with debug output
cargo test -- --nocapture

# Run with demo project
cargo run -- --project demo/multi-module --debug
```

### Code Style

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings
```

## License

MIT License

## 📚 Documentation

Comprehensive documentation is available in the `docs/` directory:

### For Users
- **[Getting Started](docs/user/README.md)** - User guide and features
- **[Architecture](docs/user/ARCHITECTURE.md)** - How LazyMVN works
- **[Libraries](docs/user/LIBRARIES.md)** - Reusable components
- **[Troubleshooting](docs/user/TROUBLESHOOTING.md)** - Common issues and solutions

### For Developers
- **[Contributing](CONTRIBUTING.md)** - How to contribute
- **[Internal Docs](docs/internal/README.md)** - Technical details
- **[Roadmap](docs/ROADMAP_INDEX.md)** - Future plans

### Quick Links
- **Configuration**: [Custom Flags](docs/user/CUSTOM_FLAGS.md), [Profiles](docs/user/PROFILE_ACTIVATION.md)
- **Logging**: [Log Config](docs/user/LOGGING_CONFIG.md), [Log Formatting](docs/user/LOG_FORMATTING.md)
- **Spring Boot**: [Launcher](docs/user/SPRING_BOOT_LAUNCHER.md), [Properties](docs/user/SPRING_PROPERTIES_OVERRIDE.md)

## Logging Configuration

LazyMVN allows you to control log verbosity without modifying your source code. Add a `[logging]` section to your `lazymvn.toml`:

```toml
[logging]
packages = [
    { name = "org.springframework", level = "WARN" },
    { name = "org.hibernate", level = "ERROR" },
    { name = "com.mycompany", level = "DEBUG" },
]
```

**Benefits:**
- 🎯 No source code changes required
- 🔧 Per-developer preferences
- 🔄 Instantly reversible
- 📦 Works across all modules
- ✨ Compatible with Log4j, Logback, SLF4J, and Spring Boot

See [docs/user/LOGGING_CONFIG.md](docs/user/LOGGING_CONFIG.md) for detailed documentation and examples.
