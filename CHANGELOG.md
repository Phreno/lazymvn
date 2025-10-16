# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.9] - 2025-10-16

### Added
- Asynchronous command execution with real-time streaming output
- Animated spinner (⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏) showing command is running
- Elapsed time display during command execution
- UI remains responsive during long-running Maven builds
- Navigation key debouncing (100ms) to prevent oversensitive scrolling
- Profile detection from all modules in multi-module projects (not just root)
- Automatic profile deduplication and alphabetical sorting

### Changed
- Command execution is now non-blocking - UI updates while commands run
- Output streams line-by-line as it arrives instead of waiting for completion
- Profile discovery now runs without `-N` flag to include child module profiles

### Fixed
- Application freezing during long-running Maven commands (#17)
- Arrow key oversensitivity causing multiple selections on Windows (#12)
- Profiles in child module POMs not being detected in multi-module projects (#16)

## [Unreleased]

### Added
- Support for single-module Maven projects (projects without `<modules>` section)
- Display "(root project)" for single-module projects in UI
- Automatic detection and normalization of empty module lists
- Smart POM change detection using content hashing
- Module caching in `~/.config/lazymvn/cache.json`
- Search functionality in output pane (`/` to search, `n`/`N` to navigate)
- Maven profiles view and toggle functionality (`p` key)
- Build flags view and toggle functionality (`f` key)
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
- Comprehensive test suite with 30 tests
- Debug logging with `--debug` flag

### Changed
- Reorganized demo projects into structured `demo/` folder
- Renamed `demo-project/` to `demo/multi-module/`
- Improved UI with three views: Modules, Profiles, Flags
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

## [0.1.0] - Initial Development

### Added
- Basic TUI interface using ratatui and crossterm
- Maven module discovery from POM files
- Common Maven commands: build, compile, test, package, install, dependency:tree
- Module selection and navigation
- Real-time Maven output display
- Keyboard shortcuts for common operations
- Maven wrapper (`mvnw`) support
- Cross-platform support (Linux, macOS, Windows)
