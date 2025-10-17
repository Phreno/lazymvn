# LazyMVN

[![Open in GitHub Codespaces](https://github.com/codespaces/badge.svg)](https://codespaces.new/Phreno/lazymvn)
[![Open in Gitpod](https://gitpod.io/button/open-in-gitpod.svg)](https://gitpod.io/#https://github.com/Phreno/lazymvn)

**LazyMVN** is a **cross-platform terminal UI (TUI)** for interacting with **Maven** projects efficiently without leaving the terminal.
Inspired by *LazyGit*, it provides a clean, keyboard-driven interface to build, test, and manage Maven projects via a **single Rust binary** with no external dependencies.

## Features

### LazyGit-Style Interface
- **Dedicated numbered view blocks**: Projects [0], Modules [2], Profiles [3], Flags [4], Output [0]
- **Quick navigation**: Switch between views instantly with number keys or arrow keys (←/→)
- **Mouse support**: Click on any pane to focus it, click on items to select them
- **Simultaneous display**: All views visible at once for better context
- **Clean separation**: Output pane on the right, selection blocks on the left
- **Adaptive layout**: Automatically adjusts to terminal size
  - Single-column mode for narrow terminals (< 80 columns)
  - Focus-driven expansion for short terminals (< 30 rows)
  - Perfect for split-screen development

### Project Support
- **Single-module projects**: Automatically detected, displayed as "(root project)"
- **Multi-module projects**: Lists all modules from the `<modules>` section
- **Smart caching**: Remembers project structure and tracks POM changes

### Maven Operations
- Execute common Maven commands: `clean`, `compile`, `test`, `package`, `install`, `dependency:tree`
- Module-scoped builds using `-pl` flag (multi-module projects)
- Build combinations: `clean install` with one keystroke
- Kill running processes with `x` key

### Profiles & Flags
- Toggle Maven profiles interactively
- Enable/disable build flags:
  - `--also-make` - Build module dependencies
  - `--also-make-dependents` - Build dependent modules
  - `-DskipTests` - Skip test execution
  - `--update-snapshots` - Force snapshot updates
  - `--offline` - Work offline
  - `--fail-fast` - Stop at first failure

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

## Technical Stack

- **Language:** Rust (edition 2024)
- **CLI:** `clap` (argument parsing)
- **TUI:** `ratatui` + `crossterm` (terminal rendering)
- **Search:** `fuzzy-matcher` + `regex` (fuzzy search and pattern matching)
- **XML:** `quick-xml` (POM parsing)
- **Config:** `toml` + `serde` (configuration)

## Development Environment

### GitHub Codespaces / DevContainer

LazyMVN includes a fully configured development environment for instant setup:

```bash
# Launch in GitHub Codespaces
1. Click "Code" → "Create codespace on main"
2. Wait for automatic setup (Rust + Java + tools)
3. Start developing immediately!
```

**Included tools:**
- ✅ Rust (latest stable) + cargo tools
- ✅ Java 21 + Maven
- ✅ VS Code extensions (rust-analyzer, Java pack)
- ✅ Git Flow + useful aliases
- ✅ Pre-configured settings and optimizations

See [`.devcontainer/README.md`](.devcontainer/README.md) for details.

### Manual Setup

For local development without containers:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Java + Maven
sudo apt install openjdk-21-jdk maven  # Ubuntu/Debian
brew install openjdk@21 maven         # macOS

# Clone and build
git clone https://github.com/Phreno/lazymvn.git
cd lazymvn
cargo build
```

## Key Bindings

### Navigation
| Key | Action |
|-----|--------|
| `←` / `→` | Cycle focus between all panes (Projects → Modules → Profiles → Flags → Output) |
| `↑` / `↓` | Move selection in current list pane / Scroll output |
| `Page Up` / `Page Down` | Scroll output by pages |
| `Home` / `End` | Jump to start/end of output |
| **Mouse** | Click on pane to focus it, click on item to select it |

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
| `d` | Dependencies | `dependency:tree` |
| `x` | Kill running process | - |

### Selection & Search
| Key | Action |
|-----|--------|
| `Space` or `Enter` | Toggle selection (profiles/flags) |
| `/` | Start search in output |
| `n` | Next search match |
| `N` | Previous search match |
| `Esc` | Exit search mode |

### General
| Key | Action |
|-----|--------|
| `q` | Quit lazymvn |

## Installation

### From Source

```bash
git clone https://github.com/Phreno/lazymvn.git
cd lazymvn
cargo build --release
sudo cp target/release/lazymvn /usr/local/bin/
```

### From Release (Coming Soon)

```bash
curl -sSL https://github.com/Phreno/lazymvn/releases/latest/download/lazymvn -o /usr/local/bin/lazymvn
chmod +x /usr/local/bin/lazymvn
```

### Requirements

- Rust 1.70+ (for building from source)
- Maven 3.x or Maven wrapper (`mvnw`) in your project

## Usage

### Basic Usage

Navigate to any Maven project directory and run:

```bash
lazymvn
```

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

### Command-line Options

```
lazymvn [OPTIONS]

Options:
  -d, --debug              Enable debug logging to lazymvn-debug.log
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
```

### Debug Logging

When troubleshooting issues, enable detailed debug logging:

```bash
lazymvn --debug
```

This creates a `lazymvn-debug.log` file in the current directory with timestamped entries at INFO, DEBUG, and ERROR levels. The log file doesn't interfere with the TUI and can be monitored in a separate terminal:

```bash
tail -f lazymvn-debug.log
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

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

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
