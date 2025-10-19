# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.4] - 2025-10-19

### Added
- **Profile XML Display** (#38-related):
  - Press `2` to focus on Profiles view and see the full XML of the selected profile
  - Displays profile source (pom.xml, settings.xml, or ~/.m2/settings.xml)
  - Auto-formatted XML with proper 4-space indentation using `xmltree` library
  - Professional syntax highlighting with color-coded elements:
    - Light Blue for opening tags
    - Light Red for closing tags
    - Light Yellow for attribute names
    - Light Green for attribute values
    - Dark Gray for structural elements (brackets, comments)
  - Automatically switches between profile XML and Maven logs when changing focus
  - Preserves Maven log buffer when viewing profiles
  - Scrollable and read-only display
  
- **Extended Profile Detection**:
  - Now detects profiles from `settings.xml` in addition to POM files
  - Searches in order: `<project>/settings.xml`, `~/.m2/settings.xml`, project POMs, module POMs
  - Maven's `help:all-profiles` only shows POM profiles - now supplemented with direct settings.xml parsing
  - All profiles from all sources merged and deduplicated
  
- **Maven Command Display** (#38):
  - Shows the exact Maven command being executed at the top of output
  - Displayed in cyan with bold styling: `$ mvn -P profiles -pl module flags goals`
  - Includes all parameters: settings, profiles, module, flags, and goals
  - Also logged to debug log with timestamp
  
### Fixed
- Profile persistence now working correctly - states saved and restored between sessions
- Mouse click tests adjusted for Projects pane
- Settings.xml profiles now properly detected and usable in Maven commands
- All clippy warnings resolved with idiomatic Rust patterns (let-chains, for loops)
- XML indentation preserved in profile display (fixed Wrap trimming issue)

### Technical
- Added `xmltree = "0.11"` dependency for professional XML parsing and formatting
- New functions: `prettify_xml()`, `colorize_xml_line()`, `extract_profiles_from_settings_xml()`
- Enhanced `get_profiles()` to read from both POM files and settings.xml
- Improved color scheme for better readability on dark terminals
- 84 tests passing (9 new tests added)
- Zero clippy warnings with `-D warnings` flag

## [0.3.0] - 2025-10-19

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
- **Per-Module Preferences**:
  - Automatically saves active profiles and enabled flags for each module
  - Preferences restored when switching between modules
  - Stored in `~/.config/lazymvn/preferences/<project-hash>.json`
  - Works seamlessly with multi-module projects
  - No manual configuration needed - just toggle and go!
- **Truthful Maven Profile Activation**:
  - Three-state profile system: Active (✓), Inactive ( ), Explicitly Disabled (✗)
  - Detects auto-activated profiles (via file existence, OS, JDK version, etc.)
  - Toggle profiles with Space: Inactive → Active → Explicitly Disabled → Inactive
  - Commands respect all three states: `-P profile` (active), omit (inactive), `-P !profile` (disabled)
  - Proper handling of `mvn help:active-profiles` to detect activation
  - See PROFILE_ACTIVATION.md for detailed documentation

### Changed
- **Kill process keybinding**: Changed from `x` to `Escape` for better UX consistency
  - More intuitive and follows common conventions (Escape to stop/cancel)
  - Shown in footer navigation bar as "Esc Kill"
- Rounded borders for popup windows (recent projects, starters)
- Improved UI consistency across all components

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

