# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Multi-Tab Project Support** (#TBD):
  - Open up to 10 Maven projects simultaneously in separate tabs
  - Visual tab bar at top showing all open projects with active tab highlighting
  - Tab indicators showing current position (e.g., "1/3")
  - Independent state per tab: each maintains its own module selection, profiles, flags, and output
  - Quick tab navigation with `Ctrl+Left`/`Ctrl+Right` keybindings
  - Create new tabs with `Ctrl+T` (opens recent projects popup)
  - Close tabs with `Ctrl+W` (prevents closing last tab)
  - Process isolation: each tab can run its own Maven process independently
  - Auto-cleanup: automatically saves preferences and kills processes when closing tabs
  - Tab bar only displays when multiple tabs are open (clean single-tab UI)
  - Footer redesigned with 3 lines: Views, Tabs/Navigation, Actions
  - Perfect for microservices development, frontend+backend workflows, or multi-project testing
- **Logging Configuration** (#TBD):
  - Control log verbosity via `lazymvn.toml` configuration
  - Package-level log level overrides using JVM arguments
  - Automatically injects `-Dlog4j.logger.{package}={level}` and `-Dlogging.level.{package}={level}`
  - No source code modifications required
  - Per-developer preferences (doesn't modify project files)
  - Works with Log4j, Logback, SLF4J, and Spring Boot logging
  - See `LOGGING_CONFIG.md` for detailed documentation
  - Example configuration in `lazymvn.toml.example`
- **Live Configuration Reload** (#TBD):
  - Press `Ctrl+E` to edit configuration file in system editor
  - Configuration changes are automatically applied when editor closes
  - No application restart needed for configuration updates
  - Detects changes to launch_mode, watch settings, notifications, maven_settings, and more
  - Automatically recreates file watcher if watch configuration changes
  - Provides immediate feedback on detected configuration changes
  - Improves developer workflow by eliminating restart cycle
- **Asynchronous Profile Loading** (#TBD):
  - Profile discovery now happens asynchronously in a background thread
  - UI remains responsive during profile loading (no blocking)
  - Animated spinner (⠋⠙⠹⠸⠼⠴⠦⠧) displays while profiles are being discovered
  - 30-second timeout for profile loading with clear error messaging
  - Loading status displayed in Profiles pane title and content area
  - Improved startup experience - application starts faster and feels more dynamic
  - Profile loading state management with `ProfileLoadingStatus` enum (Loading, Loaded, Error)

### Fixed
- **WAR Module `exec:java` Support** (#TBD):
  - Fixed `NoClassDefFoundError: javax/servlet/Filter` when running WAR modules with `exec:java`
  - Automatically adds `-Dexec.classpathScope=compile` for WAR packaging to include provided dependencies
  - Servlet API and other `provided` scope dependencies now available at runtime
  - Also adds `-Dexec.cleanupDaemonThreads=false` for better shutdown behavior
  - Works automatically without POM modifications - detects packaging type and adjusts classpath scope
- **Process Cleanup on Exit** (#TBD):
  - Fixed orphaned Maven/Java processes when quitting lazymvn
  - Application now properly kills running Maven processes on exit (both 'q' key and Ctrl+C)
  - Prevents zombie Java processes that continue running after lazymvn closes
  - Graceful shutdown: sends SIGTERM first, then SIGKILL if process doesn't terminate
  - Works on both Unix (kill command with process groups) and Windows (taskkill /T)

### Changed
- **Startup Performance**:
  - Profiles are now loaded in parallel with UI initialization
  - Loading screen progresses faster through initialization steps
  - Profile discovery step completes immediately, actual loading happens in background

### Technical
- Added `ProfileLoadingStatus` enum to track loading state
- Added `profile_loading_spinner()` method for animated loading indicator
- Added `poll_profiles_updates()` to process async profile results
- Added `start_loading_profiles()` to initiate async loading with timeout
- Added 5 unit tests for profile state management and spinner animation
- Added 2 integration tests for timeout behavior and spinner frames
- Profile loading now uses mpsc channels for thread communication
- Added `reload_config()` method to `TuiState` for live configuration reload
- Config reload detects changes and logs modifications to key settings
- File watcher automatically recreated when watch configuration changes
- Added `PartialEq` trait to config structs for change detection
- Added `cleanup()` method to `TuiState` for graceful shutdown
- Integrated `ctrlc` crate for signal handling (Ctrl+C, SIGTERM)
- Process cleanup now kills entire process group to catch child processes
- File watcher automatically recreated when watch configuration changes
- Added `PartialEq` trait to config structs for change detection
- Timeout mechanism prevents indefinite hangs if Maven is unresponsive
- Enhanced `build_launch_command()` to accept packaging type and adjust classpath scope accordingly
- Added 2 new tests for WAR and JAR packaging classpath behavior

## [0.3.6] - 2025-10-20

### Fixed
- **Spring Boot Launcher** (#39):
  - Fixed "No plugin found for prefix 'spring-boot'" error
  - Automatically detects if `spring-boot-maven-plugin` is configured in POM
  - Smart command selection: uses `spring-boot:run` when plugin available, falls back to `exec:java -Dexec.mainClass` otherwise
  - Fixed duplicate profiles and flags in Spring Boot commands (issue where parameters were added twice)
  - Works with any Maven project (Spring Boot or plain Java)
  
- **Profile XML Preview for maven_settings.xml**:
  - Profile XML preview now correctly uses `maven_settings.xml` when configured
  - Previously only checked `settings.xml`, ignoring `maven_settings.xml`
  - Now consistent with profile detection and Maven command execution
  - Respects `lazymvn.toml` configuration for custom settings file paths

### Technical
- Added `has_spring_boot_plugin()` function to detect plugin presence
- Simplified `run_spring_boot_starter()` to avoid duplicate parameter building
- Updated `get_profile_xml()` to use `config.maven_settings` for consistency
- Added 2 new tests: `test_has_spring_boot_plugin`, `test_get_profile_xml_with_maven_settings_xml`
- 86 tests passing (all green)
- Zero clippy warnings

## [0.3.5] - 2025-10-20

### Fixed
- Spring Boot starter command building improvements
- Plugin detection and fallback mechanism

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

