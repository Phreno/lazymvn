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

1.  **Update user documentation** in `docs/user/` when adding or changing user-facing features.
2.  **Update internal documentation** in `docs/internal/` when changing architecture or implementation details.
3.  **Update the main `README.md`** if the changes are significant for new users.
4.  **Update `CHANGELOG.md`** following the Keep a Changelog format.
5.  **Add doc comments** for public APIs.

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

Understanding the modular codebase (after Phases 1-6 refactoring):

```
src/
├── main.rs              # Entry point, CLI argument parsing
├── lib.rs               # Public API (for library usage)
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
│   ├── text.rs          # Text processing
│   ├── logger.rs        # Logging system
│   ├── watcher.rs       # File watching for live reload
│   ├── loading.rs       # Loading screen animations
│   └── git.rs           # Git repository operations
│
├── ui/                  # UI components (12 modules in state/)
│   ├── mod.rs
│   ├── keybindings/     # Key event handling (6 modules)
│   │   ├── mod.rs
│   │   ├── types.rs
│   │   ├── popup_keys.rs
│   │   ├── search_keys.rs
│   │   ├── output_keys.rs
│   │   ├── command_keys.rs
│   │   └── navigation_keys.rs
│   ├── state/           # Application state (12 modules)
│   │   ├── mod.rs               # Main TuiState (1,694 lines)
│   │   ├── project_tab.rs
│   │   ├── commands.rs
│   │   ├── output.rs
│   │   ├── search.rs
│   │   ├── navigation.rs
│   │   ├── profiles.rs
│   │   ├── flags.rs
│   │   ├── tabs.rs
│   │   ├── launcher_config.rs   # JVM/Spring config helpers
│   │   └── config_reload.rs     # Config reload helpers
│   ├── panes/           # UI pane rendering (5 modules)
│   │   ├── mod.rs
│   │   ├── basic_panes.rs
│   │   ├── layout.rs
│   │   ├── tab_footer.rs
│   │   └── popups.rs
│   ├── search.rs
│   └── theme.rs
│
└── tui/                 # TUI coordination (3 modules)
    ├── mod.rs           # Main coordination
    ├── renderer.rs      # TUI rendering logic
    └── mouse.rs         # Mouse event handling
```

### Key Architectural Decisions

**22 modules created** across 6 refactoring phases:
- Phase 1: `ui/state/` split into 8 modules (-42%, 1,366 lines)
- Phase 3: `ui/panes/` split into 4 modules (-91%, 1,295 lines)
- Phase 4: `ui/keybindings/` split into 5 modules (-38%, 458 lines)
- Phase 5: `tui.rs` split into 3 modules (architectural separation)
- Phase 6: Added 2 helper modules + micro-refactored 3 functions (-10.3%, 195 lines)

**Phase 6 Innovation - Micro-refactoring**:
- `yank_debug_info()`: 281 → 21 lines (-92.5%, 14 helpers)
- `run_spring_boot_starter()`: 176 → 22 lines (-87.5%, 9 helpers)
- `reload_config()`: 111 → 12 lines (-89.2%, 7 helpers in module)
- **Total complexity reduction**: 568 → 55 lines (-90.3%)

**Benefits**:
- Clear separation of concerns (22 modules)
- Easier to test individual components
- Reduced file sizes (largest file: 1,694 lines, -48% from 3,255)
- Improved code readability via micro-refactoring
- Reusable helper functions and modules

## Refactoring Best Practices

When modifying or extending the codebase, follow these proven patterns:

### Module Extraction (Architecture)
Extract code into separate modules when:
- ✅ File exceeds ~1,000 lines
- ✅ Clear functional boundaries exist
- ✅ Code can be grouped by domain/responsibility
- ✅ Module can be tested independently

**Example**: `ui/state/mod.rs` (3,255 lines) → 12 modules

### Function Extraction (Readability)
Extract helper functions when:
- ✅ Function exceeds 100 lines
- ✅ Logic is mixed or hard to follow
- ✅ Code has extractable sections (loops, if blocks)
- ✅ Function names would improve readability

**Example**: `yank_debug_info()` (281 lines) → 21 lines + 14 helpers

### Helper Module Creation (Reusability)
Create dedicated helper modules when:
- ✅ Functions are coherent (same domain)
- ✅ Numerous helpers (5+ functions)
- ✅ Reusability potential exists
- ✅ Goal: actually reduce main file size

**Example**: `launcher_config.rs` for JVM config helpers (120 lines)

### Refactoring Workflow
1. **Analyze**: Identify large files or complex functions
2. **Extract**: Split into smaller, focused pieces
3. **Validate**: Run all tests (`cargo test`)
4. **Commit**: Atomic commits with clear messages
5. **Document**: Update AGENTS.md and relevant docs

**Key Rule**: Always maintain 100% test pass rate during refactoring
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

See [REFACTORING_SUMMARY.md](docs/internal/refactoring/REFACTORING_SUMMARY.md) for complete migration details.

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
