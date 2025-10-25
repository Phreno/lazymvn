# Contributing to LazyMVN

Thank you for considering contributing to LazyMVN! This document provides guidelines and instructions for contributing.

## Code of Conduct

- Be respectful and constructive
- Focus on the technical merits of ideas
- Welcome newcomers and help them learn
- Keep discussions professional and on-topic

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Git
- Maven 3.x (for testing with real projects)

### Development Setup

1. Fork and clone the repository:
```bash
git clone https://github.com/YOUR_USERNAME/lazymvn.git
cd lazymvn
```

2. Build the project:
```bash
cargo build
```

3. Run tests:
```bash
cargo test
```

4. Test with demo projects:
```bash
cargo run -- --project demo/multi-module --debug
cargo run -- --project demo/single-module --debug
```

## Development Workflow

### Branch Naming

- `feature/` - New features (e.g., `feature/fuzzy-search`)
- `fix/` - Bug fixes (e.g., `fix/cache-invalidation`)
- `docs/` - Documentation updates
- `refactor/` - Code refactoring

### Commit Messages

Follow conventional commit format:

```
<type>: <short description>

<detailed description>

<footer>
```

Types:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `test:` - Test additions or modifications
- `refactor:` - Code refactoring
- `style:` - Code style changes (formatting)
- `chore:` - Build process or tooling changes

Example:
```
feat: add fuzzy search for module selection

Implement fuzzy matching using fuzzy-matcher crate to allow
quick module filtering by typing partial names.

Closes #42
```

### Code Style

We follow standard Rust conventions:

1. **Format your code:**
```bash
cargo fmt
```

2. **Run the linter:**
```bash
cargo clippy -- -D warnings
```

3. **Follow the existing style:**
- 4-space indentation
- Maximum line length: 100 characters
- Use `snake_case` for functions and variables
- Use `PascalCase` for types and traits
- Use `SCREAMING_SNAKE_CASE` for constants

### Testing

1. **Run all tests:**
```bash
cargo test
```

2. **Run specific test:**
```bash
cargo test test_name
```

3. **Run tests with output:**
```bash
cargo test -- --nocapture
```

4. **Add tests for new features:**
   - Unit tests in the same file as implementation
   - Integration tests in `tests/` directory
   - Use the demo projects for manual testing

### Documentation

1. **Update README.md** when adding user-facing features
2. **Update CHANGELOG.md** following Keep a Changelog format
3. **Update AGENTS.md** when changing architecture or adding modules
4. **Add doc comments** for public APIs:
```rust
/// Parses Maven modules from POM content
///
/// # Arguments
/// * `content` - The POM file content as a string
///
/// # Returns
/// A vector of module names found in the `<modules>` section
pub fn parse_modules_from_str(content: &str) -> Vec<String> {
    // ...
}
```
5. **Document new modules** in module-level doc comments:
```rust
//! # Core Configuration
//!
//! This module handles loading and saving lazymvn configuration files.
//! Configuration is stored in `~/.config/lazymvn/projects/<hash>/config.toml`.
```

## Pull Request Process

### Before Submitting

1. ✅ Code builds without errors: `cargo build`
2. ✅ All tests pass: `cargo test`
3. ✅ Code is formatted: `cargo fmt`
4. ✅ No clippy warnings: `cargo clippy -- -D warnings`
5. ✅ Architecture validation: `./scripts/test-refactoring.sh`
6. ✅ Documentation is updated
7. ✅ CHANGELOG.md is updated

### Submitting

1. Push your branch to your fork
2. Create a Pull Request against `main` branch
3. Fill in the PR template with:
   - Clear description of changes
   - Link to related issues
   - Testing performed
   - Screenshots (for UI changes)

### Review Process

1. Maintainers will review your PR
2. Address any feedback or requested changes
3. Once approved, maintainers will merge your PR

## Project Structure

Understanding the modular codebase:

```
src/
├── main.rs              # Entry point, CLI argument parsing
├── lib.rs               # Public API (for library usage)
├── tui.rs               # TUI coordination and rendering
├── maven_tests.rs       # Maven integration tests
│
├── core/                # Core functionality
│   ├── mod.rs
│   ├── config.rs        # Configuration file management (lazymvn.toml)
│   └── project.rs       # POM parsing, module discovery, caching
│
├── maven/               # Maven command execution and detection
│   ├── mod.rs
│   ├── command.rs       # Command building and execution
│   ├── detection.rs     # Spring Boot/exec:java detection
│   ├── process.rs       # Process management
│   ├── profiles.rs      # Profile loading
│   ├── log4j.rs         # Log4j configuration override
│   └── spring.rs        # Spring Boot properties override
│
├── features/            # Optional features
│   ├── mod.rs
│   ├── favorites.rs     # Favorites management
│   ├── history.rs       # Command history
│   └── starters.rs      # Spring Boot starter management
│
├── utils/               # Shared utilities
│   ├── mod.rs
│   ├── text.rs          # Text processing (colorization, ANSI stripping)
│   ├── logger.rs        # Logging system
│   ├── watcher.rs       # File watching for live reload
│   ├── loading.rs       # Loading screen animations
│   └── git.rs           # Git repository operations
│
└── ui/                  # UI components
    ├── mod.rs
    ├── keybindings/     # Key event handling
    │   ├── mod.rs
    │   └── types.rs
    ├── state/           # Application state management
    │   ├── mod.rs
    │   └── project_tab.rs  # Per-tab state
    ├── panes/           # UI pane rendering
    │   └── mod.rs
    ├── search.rs        # Search functionality
    └── theme.rs         # Colors and styles
```

### Module Architecture

LazyMVN uses internal modules with clear separation of concerns:

**Core (`src/core/`)** - Configuration and project management
- `config.rs` - Load/save configuration files
- `project.rs` - Parse Maven POMs, discover modules

**Maven (`src/maven/`)** - Maven integration
- `command.rs` - Build and execute Maven commands
- `detection.rs` - Auto-detect Spring Boot/exec:java
- `profiles.rs` - Load available Maven profiles

**Features (`src/features/`)** - Optional enhancements
- `favorites.rs` - Save favorite command configurations
- `history.rs` - Track command execution history
- `starters.rs` - Manage Spring Boot starters

**Utils (`src/utils/`)** - Shared utilities
- `text.rs` - Text processing and colorization
- `logger.rs` - Logging system configuration
- `watcher.rs` - File watching for live reload

**UI (`src/ui/`)** - Terminal user interface
- `state/` - Application state (TuiState, tab state)
- `keybindings/` - Keyboard event handling
- `panes/` - UI rendering and layouts

### Import Patterns

When working with the codebase, use these import patterns:

```rust
// Configuration
use crate::core::config::{Config, LaunchMode};

// Project management
use crate::core::project;

// Maven integration
use crate::maven;
use crate::maven::detection;
use crate::maven::profiles;

// Features
use crate::features::favorites::Favorites;
use crate::features::history::CommandHistory;
use crate::features::starters::StartersCache;

// Utilities
use crate::utils::text::{clean_log_line, colorize_log_line};
use crate::utils::logger;
use crate::utils::watcher::FileWatcher;

// UI components
use crate::ui::state::TuiState;
use crate::ui::theme;
```

### Where to Add New Code

**Adding configuration options:**
- Edit `src/core/config.rs`
- Update `config_template.toml` (embedded template)
- Add tests in `config.rs`

**Adding Maven functionality:**
- Command execution → `src/maven/command.rs`
- Auto-detection logic → `src/maven/detection.rs`
- Profile management → `src/maven/profiles.rs`

**Adding user features:**
- Favorites-related → `src/features/favorites.rs`
- History-related → `src/features/history.rs`
- Spring Boot starters → `src/features/starters.rs`

**Adding utilities:**
- Text processing → `src/utils/text.rs`
- File operations → Consider `src/utils/` (or create new file)

**Adding UI elements:**
- State management → `src/ui/state/mod.rs`
- Key bindings → `src/ui/keybindings/mod.rs`
- Visual rendering → `src/ui/panes/mod.rs`

### Migrating Existing Code

If you have an existing PR or branch that needs updating for the new module structure:

**1. Update imports** - Replace old paths with new module paths:

```bash
# Configuration
sed -i 's/use crate::config::/use crate::core::config::/g' src/**/*.rs

# Project management
sed -i 's/use crate::project::/use crate::core::project::/g' src/**/*.rs

# Favorites
sed -i 's/use crate::favorites::/use crate::features::favorites::/g' src/**/*.rs

# History
sed -i 's/use crate::history::/use crate::features::history::/g' src/**/*.rs

# Starters
sed -i 's/use crate::starters::/use crate::features::starters::/g' src/**/*.rs

# Logger
sed -i 's/use crate::logger::/use crate::utils::logger::/g' src/**/*.rs

# Watcher
sed -i 's/use crate::watcher::/use crate::utils::watcher::/g' src/**/*.rs
```

**2. Verify imports** - Check for double `core::` or other issues:

```bash
# Find potential issues
grep -r "core::core::" src/
grep -r "features::features::" src/
grep -r "utils::utils::" src/
```

**3. Test compilation:**

```bash
cargo build
cargo test
./scripts/test-refactoring.sh
```

**Before/After Examples:**

```rust
// OLD imports
use crate::config::{Config, LaunchMode};
use crate::project;
use crate::favorites::Favorites;
use crate::logger;

// NEW imports
use crate::core::config::{Config, LaunchMode};
use crate::core::project;
use crate::features::favorites::Favorites;
use crate::utils::logger;
```

See [REFACTORING_SUMMARY.md](REFACTORING_SUMMARY.md) for complete migration details.

### Key Concepts

**Module Discovery Flow:**
1. Find `pom.xml` (walk up directory tree)
2. Parse POM XML to extract `<modules>`
3. Normalize empty lists to `["."]` for single-module projects
4. Cache results with POM hash

**Command Execution Flow:**
1. User presses key (e.g., `b` for build)
2. `keybindings::handle_key_event` processes event
3. `state::run_selected_module_command` constructs command
4. `maven::execute_maven_command` runs Maven
5. Output streamed to `state.command_output`
6. UI re-renders with new output

**State Management:**
- `TuiState` holds all application state
- Views: Modules, Profiles, Flags
- Focus: left pane or output pane
- Selection state for each list
- Output buffer and scroll position

## Feature Ideas

Looking for contribution ideas? Here are some feature requests:

### Small Tasks (Good First Issues)
- [ ] Add more build flags
- [ ] Improve error messages
- [ ] Add keyboard shortcut help screen (press `?`)
- [ ] Add syntax highlighting for output
- [ ] Make colors configurable

### Medium Tasks
- [ ] Fuzzy search for module selection
- [ ] Command history
- [ ] Bookmarks for frequently used modules
- [ ] Export output to file
- [ ] Split view for comparing outputs

### Large Tasks
- [ ] Plugin system
- [ ] Remote Maven repository browser
- [ ] Dependency visualization
- [ ] Multi-project workspace support
- [ ] Integration with CI/CD systems

## Questions?

- Open an issue for bugs or feature requests
- Tag issues with `question` for general questions
- Check existing issues before creating new ones

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
