# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Spring Boot Starter Support** (#33):
  - Press `s` to run Spring Boot applications
  - Intelligent detection of `*Application.java`, `*Main.java`, and `@SpringBootApplication` classes
  - Fuzzy search selector for choosing main class
  - Cached starters per project in `~/.config/lazymvn/starters/<hash>.json`
  - Support for multiple starters (API, Admin, Batch, etc.)
  - Starter manager with `Ctrl+Shift+S` to view, run, and manage cached starters
  - Mark starters as default
  - Remembers last used starter
  - Executes via `mvn spring-boot:run -Dspring-boot.run.mainClass=<FQCN>`
  - Full integration with profiles and build flags
- **Recent Projects Navigation** (#32): 
  - Track up to 20 recently opened Maven projects
  - Press `Ctrl+R` to open a popup listing all recent projects
  - Navigate with arrow keys and press Enter to switch projects
  - Projects are stored in `~/.config/lazymvn/recent.json` (Linux/macOS) or `%APPDATA%\lazymvn\recent.json` (Windows)
  - Invalid paths are automatically cleaned from the list
  - Switch between projects without restarting LazyMVN
  - **Smart fallback**: When no POM is found in current directory, automatically loads the most recent project
  - Clear error messages when no project is available

## [0.2.0] - 2025-10-17

### Added
- **Mouse support**: Click on panes to focus them, click on items to select them (#15)
- **Adaptive layout**: Automatically adjusts to terminal size (#26)
  - Single-column mode for narrow terminals (< 80 columns)
  - Focus-driven panel expansion for short terminals (< 30 rows)
  - Smooth transitions when terminal is resized
- **Enhanced navigation**: 
  - Left/right arrows now cycle through all panes (1→2→3→4→0)
  - Key `0` to focus Output pane directly
  - Focus indication with rounded borders and visual styling
- **Improved UI**:
  - Rounded borders for all panes for a modern look
  - Better padding alignment between Profiles and Flags panes
  - Adjusted layout ratio (30/70 split) for more output space
  - Removed redundant profile/flag buttons from footer

### Fixed
- **Windows keyboard events**: Fixed duplicate key events on Windows (#22)
  - Only processes KeyEventKind::Press events, ignoring Release and Repeat
  - Resolves double-toggle issues and navigation skipping
- Flags pane now properly pre-selects first item for consistent alignment
- Focus state properly maintained across all panes

### Changed
- Navigation now uses unified Focus enum for all panes (Projects, Modules, Profiles, Flags, Output)
- View switching (keys 1-4) now also sets focus to the selected pane
- Layout intelligently adapts based on terminal dimensions and focused pane

## [0.1.9] - 2025-10-16

### Added
- Asynchronous command execution with real-time streaming output
- Animated spinner (⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏) showing command is running
- Elapsed time display during command execution
- UI remains responsive during long-running Maven builds
- Navigation key debouncing (100ms) to prevent oversensitive scrolling
- Profile detection from all modules in multi-module projects (not just root)
- Automatic profile deduplication and alphabetical sorting
- Kill running Maven process with `x` key (#24)
- Process ID tracking for running commands
- Cross-platform process termination (SIGTERM on Unix, taskkill on Windows)

### Changed
- Command execution is now non-blocking - UI updates while commands run
- Output streams line-by-line as it arrives instead of waiting for completion
- Profile discovery now runs without `-N` flag to include child module profiles

### Fixed
- Application freezing during long-running Maven commands (#17)
- Arrow key oversensitivity causing multiple selections on Windows (#12)
- Profiles in child module POMs not being detected in multi-module projects (#16)
- Windows support: Maven command now uses `mvn.cmd` instead of `mvn`

## [0.1.0] - Initial Development

### Added
- Support for single-module Maven projects (projects without `<modules>` section)
- Display "(root project)" for single-module projects in UI
- Automatic detection and normalization of empty module lists
- Smart POM change detection using content hashing
- Module caching in `~/.config/lazymvn/cache.json`
- Search functionality in output pane (`/` to search, `n`/`N` to navigate)
- Maven profiles view and toggle functionality
- Build flags view and toggle functionality
- Support for Maven build flags:
  - `--also-make` - Build module dependencies
  - `--also-make-dependents` - Build dependent modules  
  - `-DskipTests` - Skip test execution
  - `--update-snapshots` - Force snapshot updates
  - `--offline` - Work offline
  - `--fail-fast` - Stop at first failure
- Page Up/Down support for output scrolling
- Auto-detection of Maven settings from project or `~/.m2/`
- Configuration file support via `lazymvn.toml`
- Color-coded log output (INFO, WARN, ERROR levels)
- Demo projects in `demo/` folder:
  - `multi-module/` - Multi-module Maven project example
  - `single-module/` - Single-module Maven project example
- Comprehensive test suite
- Debug logging with `--debug` flag
- Basic TUI interface using ratatui and crossterm
- Maven module discovery from POM files
- Common Maven commands: build, compile, test, package, install, dependency:tree
- Module selection and navigation
- Real-time Maven output display
- Keyboard shortcuts for common operations
- Maven wrapper (`mvnw`) support
- Cross-platform support (Linux, macOS, Windows)

### Changed
- Reorganized demo projects into structured `demo/` folder
- Renamed `demo-project/` to `demo/multi-module/`
- Improved UI with multiple views: Projects, Modules, Profiles, Flags
- Enhanced footer with context-aware command display
- Updated documentation with accurate feature descriptions

### Removed
- Unused `tokio` dependency
- Outdated installation instructions

### Fixed
- Commands now execute correctly on single-module projects (without `-pl` flag)
- Cache invalidation when POM file changes
- Module selection state initialization for empty projects
- `--project` / `-p` command-line argument now correctly changes to specified directory

