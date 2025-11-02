# Development Guidelines

This file contains coding guidelines and conventions for contributors and AI agents working on LazyMVN.

> **Note:** For comprehensive contribution guidelines, see [CONTRIBUTING.md](CONTRIBUTING.md)

## Project Structure

```
lazymvn/
├── src/                  # Source code
│   ├── main.rs           # Entry point, CLI argument parsing
│   ├── lib.rs            # Public API (NEW - for library usage)
│   ├── tui.rs            # TUI coordination and rendering
│   ├── maven_tests.rs    # Maven integration tests
│   │
│   ├── core/             # Core functionality (NEW)
│   │   ├── mod.rs
│   │   ├── config.rs     # Configuration file management (lazymvn.toml)
│   │   └── project.rs    # POM parsing, module discovery, caching
│   │
│   ├── maven/            # Maven command execution and detection
│   │   ├── mod.rs
│   │   ├── command.rs    # Command building and execution
│   │   ├── detection.rs  # Spring Boot/exec:java detection
│   │   ├── process.rs    # Process management
│   │   ├── profiles.rs   # Profile loading
│   │   ├── log4j.rs      # Log4j configuration override
│   │   └── spring.rs     # Spring Boot properties override
│   │
│   ├── features/         # Optional features (NEW)
│   │   ├── mod.rs
│   │   ├── favorites.rs  # Favorites management
│   │   ├── history.rs    # Command history
│   │   └── starters.rs   # Spring Boot starter management
│   │
│   ├── utils/            # Shared utilities (NEW)
│   │   ├── mod.rs
│   │   ├── text.rs       # Text processing (colorization, ANSI stripping)
│   │   ├── logger.rs     # Logging system
│   │   ├── watcher.rs    # File watching for live reload
│   │   ├── loading.rs    # Loading screen animations
│   │   └── git.rs        # Git repository operations
│   │
│   └── ui/               # UI components
│       ├── mod.rs
│       ├── keybindings/  # Key event handling
│       │   ├── mod.rs
│       │   ├── types.rs
│       │   ├── popup_keys.rs
│       │   ├── search_keys.rs
│       │   ├── output_keys.rs
│       │   ├── command_keys.rs
│       │   └── navigation_keys.rs
│       ├── state/        # Application state management (REFACTORED)
│       │   ├── mod.rs              # Main state (1,694 lines, -48% from original)
│       │   ├── project_tab.rs      # Per-tab state
│       │   ├── commands.rs         # Command execution
│       │   ├── output.rs           # Output management
│       │   ├── search.rs           # Search state
│       │   ├── navigation.rs       # Navigation logic
│       │   ├── profiles.rs         # Profile management
│       │   ├── flags.rs            # Flag management
│       │   ├── tabs.rs             # Tab management
│       │   ├── launcher_config.rs  # JVM/Spring config helpers (NEW - Phase 6)
│       │   └── config_reload.rs    # Config reload helpers (NEW - Phase 6)
│       ├── panes/        # UI pane rendering (REFACTORED)
│       │   ├── mod.rs              # Pane coordination (123 lines, -91%)
│       │   ├── basic_panes.rs      # Basic pane rendering
│       │   ├── layout.rs           # Layout management
│       │   ├── tab_footer.rs       # Tab footer rendering
│       │   └── popups.rs           # Popup rendering
│       ├── search.rs     # Search functionality
│       └── theme.rs      # Colors and styles
│
├── tui/                  # TUI coordination (NEW - Phase 5)
│   ├── mod.rs            # Main coordination (540 lines)
│   ├── renderer.rs       # TUI rendering (194 lines)
│   └── mouse.rs          # Mouse event handling (122 lines)
│
├── docs/                 # Documentation
│   ├── user/             # User-facing guides and feature documentation
│   └── internal/         # Implementation notes, design docs, and refactoring plans
├── examples/             # Configuration examples
│   ├── README.md         # Examples index
│   ├── lazymvn.toml.example          # Complete example
│   ├── lazymvn.toml.spring-boot-example  # Spring Boot config
│   ├── lazymvn.toml.watch-example    # Watch mode config
│   └── ...               # Other examples
├── scripts/              # Test and utility scripts
│   ├── README.md         # Scripts documentation
│   ├── test_debug_yank.sh    # Debug yank feature test
│   ├── test-env.sh       # Environment validation
│   ├── test-refactoring.sh   # Architecture validation (NEW)
│   ├── test-live-reload.sh   # Live reload test
│   └── ...               # Other test scripts
├── demo/                 # Demo Maven projects for testing
│   ├── multi-module/     # Multi-module Maven project
│   └── single-module/    # Single-module Maven project
├── tests/                # Integration tests
├── target/               # Build outputs (untracked)
├── AGENTS.md             # This file - development guidelines
├── CONTRIBUTING.md       # Contribution guidelines
├── README.md             # User documentation
└── CHANGELOG.md          # Version history
```

## Build & Test Commands

Essential commands for development:

```bash
# Build the project
cargo build

# Build release version
cargo build --release

# Run with demo project
cargo run -- --project demo/multi-module --debug

# Run all tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Validate architecture
./scripts/test-refactoring.sh
```

## Module Architecture

LazyMVN follows a modular architecture with clear separation of concerns:

### Core Modules (`src/core/`)
**Responsibility**: Configuration and project management

- `config.rs` - Configuration file loading and management
  - Centralized config in `~/.config/lazymvn/projects/<hash>/config.toml`
  - Template-based generation
  - Hot reload support
- `project.rs` - Maven POM parsing and module discovery
  - Multi-module detection
  - Caching with hash validation

**Usage**:
```rust
use crate::core::config;
use crate::core::project;

let (modules, root) = project::get_project_modules()?;
let config = config::load_config(&root);
```

### Maven Integration (`src/maven/`)
**Responsibility**: Maven command execution and Spring Boot integration

- `command.rs` - Maven command building and execution
- `detection.rs` - Spring Boot/exec:java auto-detection
- `profiles.rs` - Profile loading and activation
- `process.rs` - Process management
- `log4j.rs` - Log4j configuration override
- `spring.rs` - Spring Boot properties override

**Usage**:
```rust
use crate::maven;

maven::execute_maven_command(&root, &args)?;
```

### Features (`src/features/`)
**Responsibility**: Optional enhancement features

- `favorites.rs` - Save/load favorite command configurations
- `history.rs` - Track command execution history
- `starters.rs` - Spring Boot starter dependency management

**Usage**:
```rust
use crate::features::favorites::Favorites;
use crate::features::history::CommandHistory;

let favorites = Favorites::load();
let history = CommandHistory::load();
```

### UI Components (`src/ui/`)
**Responsibility**: Terminal user interface

- `state/` - Application state management (12 modules, refactored in Phases 1 & 6)
  - `mod.rs` - Main TuiState (1,694 lines, -48% from original)
  - `project_tab.rs` - Per-tab state
  - `commands.rs` - Command execution
  - `output.rs` - Output management
  - `search.rs` - Search state
  - `navigation.rs` - Navigation logic
  - `profiles.rs` - Profile management
  - `flags.rs` - Flag management
  - `tabs.rs` - Tab management
  - `launcher_config.rs` - JVM/Spring config helpers (Phase 6)
  - `config_reload.rs` - Config reload helpers (Phase 6)
- `keybindings/` - Keyboard event handling (6 modules, refactored in Phase 4)
- `panes/` - UI rendering (5 modules, refactored in Phase 3)
- `search.rs` - Search functionality
- `theme.rs` - Color schemes and styles

**Large files status** (after Phase 6):
- `state/mod.rs` - 1,694 lines (optimal, coordinator pattern, -48% from original 3,255)
- `keybindings/mod.rs` - 745 lines (event coordination, -38% from original 1,203)
- `panes/mod.rs` - 123 lines (well modularized, -91% from original 1,418)
- `core/config.rs` - 773 lines (configuration management, candidate for Phase 7)
- `maven/command.rs` - 571 lines (Maven commands, candidate for Phase 7)

**Refactoring Summary (Phases 1-6):**
- **22 modules created** (8 in Phase 1, 4 in Phase 3, 5 in Phase 4, 3 in Phase 5, 2 in Phase 6)
- **~7,000 lines reorganized** across all phases
- **3 major functions micro-refactored** in Phase 6 (568 → 55 lines, -90.3% complexity)
- **100% tests maintained** (219/219 passing throughout)
- **Architecture philosophy**: Macroscopic (modules) + Microscopic (functions) refactoring

### TUI Coordination (`src/tui/`)
**Responsibility**: TUI rendering and event coordination (NEW - Phase 5)

- `mod.rs` - Main coordination and tests (540 lines)
- `renderer.rs` - TUI rendering logic (194 lines)
- `mouse.rs` - Mouse event handling (122 lines)

**Usage**:
```rust
use crate::tui;

tui::draw(&mut terminal, &mut state)?;
tui::handle_key_event(&mut state, key)?;
```

### Utilities (`src/utils/`)
**Responsibility**: Shared utility functions

- `text.rs` - Text processing (colorization, ANSI stripping, XML formatting)
- `logger.rs` - Logging system configuration
- `watcher.rs` - File watching for live reload
- `loading.rs` - Loading screen animations
- `git.rs` - Git repository operations

**Usage**:
```rust
use crate::utils::text::{clean_log_line, colorize_log_line};
use crate::utils::logger;
use crate::utils::watcher::FileWatcher;

let cleaned = clean_log_line(&raw_line);
logger::init(log_level)?;
```

### Public API (`src/lib.rs`)
LazyMVN can be used as a library:

```rust
use lazymvn::core::project;

let (modules, root) = project::get_project_modules()?;
let config = lazymvn::core::config::load_config(&root);
```

## Project Organization

### Documentation (`docs/`)
All documentation is organized into user-facing and internal sections.
- **`docs/user/`**: Contains guides for end-users on how to use features.
- **`docs/internal/`**: Contains technical deep-dives, implementation plans, and refactoring notes for developers.

### Configuration Examples (`examples/`)
Example configuration files for different use cases.
See [examples/README.md](examples/README.md) for details.

**Available examples:**
- `lazymvn.toml.example` - Complete configuration with all features
- `lazymvn.toml.spring-boot-example` - Spring Boot optimized
- `lazymvn.toml.watch-example` - File watching configuration
- And more...

### Test Scripts (`scripts/`)
All test scripts for validating features and environment setup.
See [scripts/README.md](scripts/README.md) for usage instructions.

**Run from project root:**
```bash
./scripts/test-env.sh           # Validate environment
./scripts/test_debug_yank.sh    # Test debug yank feature
./scripts/test-live-reload.sh   # Test live reload
./scripts/test-refactoring.sh   # Validate architecture
```

### Root Directory Files
Only essential files remain in the root:
- `AGENTS.md` - This file (development guidelines)
- `CONTRIBUTING.md` - Contribution process
- `README.md` - User-facing documentation
- `CHANGELOG.md` - Version history
- `Cargo.toml` / `Cargo.lock` - Rust project files

All internal documentation has been organized in `docs/internal/`:
- `docs/internal/refactoring/REFACTORING_SUMMARY.md` - Architecture refactoring details
- `docs/internal/phases/` - Phase completion reports
- `docs/internal/test-coverage/` - Test coverage documentation

## Coding Style

### Formatting
- Use `rustfmt` defaults (4-space indent, max width 100)
- Run `cargo fmt` before committing

### Naming Conventions
- `snake_case` for functions and modules
- `PascalCase` for types and traits
- `SCREAMING_SNAKE_CASE` for constants
- `pub(crate)` for module-internal APIs

### Error Handling
- Use `Result` for fallible operations
- Provide clear error contexts
- Avoid unchecked `unwrap()` - use `?` or `unwrap_or_default()`

### Module Organization
- Keep modules focused on their domain
- Place tests alongside implementation: `#[cfg(test)] mod tests`
- Integration tests go in `tests/` directory

## Testing Guidelines

### Test Naming
Use descriptive names following the pattern: `feature_under_test_expected_outcome`

Examples:
- `get_modules_from_pom_without_modules`
- `execute_maven_command_without_pl_for_root_module`
- `normalize_modules_returns_dot_for_empty`

### Test Structure
```rust
#[test]
fn feature_expected_outcome() {
    // Arrange: Set up test fixtures
    let input = create_test_input();
    
    // Act: Execute the code under test
    let result = function_under_test(input);
    
    // Assert: Verify expectations
    assert_eq!(result, expected_value);
}
```

### Test Fixtures
- Use `demo/multi-module/` and `demo/single-module/` for manual testing
- Use `tempfile` crate for temporary directories in tests
- Clean up resources in tests (files, directories)

### Coverage
- Test happy paths and error cases
- Test edge cases (empty inputs, boundary conditions)
- Mock external dependencies (Maven commands) in unit tests

## Refactoring Guidelines

### When to Extract Modules
Extract code into separate modules when:
- ✅ File exceeds ~1,000 lines
- ✅ Clear functional boundaries exist
- ✅ Code can be grouped by domain/responsibility
- ✅ Module can be tested independently

**Example**: Phase 1 - Split `state/mod.rs` (3,255 lines) into 8 modules

### When to Extract Functions (Micro-refactoring)
Extract helper functions when:
- ✅ Function exceeds 100 lines
- ✅ Logic is mixed or hard to follow
- ✅ Code has extractable sections (loops, if blocks)
- ✅ Function names would improve readability

**Example**: Phase 6 - Extract `yank_debug_info()` (281 → 21 lines)

### When to Create Helper Modules
Create dedicated helper modules when:
- ✅ **Functions are coherent** (same domain)
- ✅ **Numerous helpers** (5+ functions)
- ✅ **Reusability potential** exists
- ✅ **Goal: actually reduce main file size**

**Example**: Phase 6 - Create `launcher_config.rs` for JVM config helpers

### Refactoring Pattern (Proven)
1. **Analyze**: Identify large files (>1,000 lines) or complex functions (>100 lines)
2. **Extract Modules**: Split into domain-specific modules (Phases 1-5)
3. **Micro-refactor**: Extract helper functions for readability (Phase 6.1-6.2)
4. **Helper Modules**: Group coherent helpers into dedicated modules (Phase 6.3)
5. **Validate**: Ensure all tests pass (219/219)
6. **Commit**: Atomic commits with clear documentation

### Key Insights
- **Module extraction** → Architecture improvement (Phases 1-5)
- **Function extraction** → Readability improvement (Phase 6 steps 1-2)
- **Helper modules** → Readability + Size reduction (Phase 6 step 3)
- **Always maintain** 100% test pass rate
- **Combine approaches**: Extract functions first, then group coherent ones into modules

### Refactoring Results (Phases 1-6)
| Phase | Approach | Files | Impact | Modules Created |
|-------|----------|-------|--------|-----------------|
| 1 | Module extraction | state/mod.rs | -42% (-1,366 lines) | 8 modules |
| 3 | Module extraction | panes/mod.rs | -91% (-1,295 lines) | 4 modules |
| 4 | Module extraction | keybindings/mod.rs | -38% (-458 lines) | 5 modules |
| 5 | Architectural split | tui.rs | ±0 (separated) | 3 modules |
| 6 | Micro-refactoring + Helpers | state/mod.rs | -10.3% (-195 lines) | 2 modules + 23 helpers |
| **Total** | **Combined** | **Multiple** | **~7,000 lines reorganized** | **22 modules** |

## Commit Guidelines

### Commit Messages
Follow conventional commit format:

```
<type>: <short description>

<optional detailed description>

<optional footer>
```

**Types:**
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `test:` - Test additions or changes
- `refactor:` - Code refactoring
- `style:` - Code formatting
- `chore:` - Build/tooling changes

**Examples:**
```
feat: add fuzzy search for module selection

Implement fuzzy matching using fuzzy-matcher crate to allow
quick module filtering by typing partial names.

Closes #42
```

```
fix: cache invalidation when POM changes

Update POM hash comparison to properly detect changes.
```

### Commit Scope
- Group related changes into single commits
- Keep commits focused and atomic
- Update relevant documentation in same commit

## Pull Request Guidelines

### Before Submitting
- [ ] Code builds: `cargo build`
- [ ] Tests pass: `cargo test`
- [ ] Code formatted: `cargo fmt`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Documentation updated (README, CHANGELOG, etc.)
- [ ] Demo projects work if UI changed

### PR Description
Include:
1. **Summary**: Brief description of changes
2. **Testing**: Commands executed and results
3. **Screenshots**: For UI changes (or asciinema)
4. **Breaking changes**: If any
5. **Related issues**: Reference via `Closes #ID`

## Architecture Notes

### Module Discovery Flow
1. Search for `pom.xml` from current directory upward
2. Parse `<modules>` section with `quick-xml`
3. Normalize empty lists to `["."]` for single-module projects
4. Cache with POM content hash in `~/.config/lazymvn/cache.json`

### Command Execution
1. Detect Maven wrapper (`./mvnw`) or use system `mvn`
2. Build command: `[mvn] [settings] [profiles] [module] [flags] [goal]`
3. For multi-module: add `-pl <module>`
4. For single-module (`.`): omit `-pl` flag
5. Stream output to application state

### State Management
- `TuiState` holds all application state
- Three views: Modules, Profiles, Flags
- Two focus panes: left (selection) and right (output)
- Output buffer stores Maven command results
- Search state tracks regex matches and position

### UI Rendering
- `ratatui` for widget rendering
- `crossterm` for terminal control
- 50ms poll interval for key events
- Color-coded output using theme styles

## Common Tasks

### Adding a New Maven Command
1. Add keybinding in `src/ui/keybindings.rs`
2. Call `state.run_selected_module_command(&["goal"])`
3. Update key bindings table in README.md
4. Add test in tests module

### Adding a Build Flag
1. Add flag to `BuildFlag` list in `src/ui/state.rs` (`TuiState::new`)
2. Flag automatically appears in Flags view
3. Test with `f` key + module command

### Adding a Configuration Option
1. Add field to `Config` struct in `src/config.rs`
2. Update `lazymvn.toml.example`
3. Document in README.md
4. Add test for parsing

## Debugging

### Enable Debug Logging
```bash
lazymvn --debug
# In another terminal:
tail -f lazymvn-debug.log
```

### Common Issues
- **Modules not detected**: Check POM has `<modules>` section
- **Cache stale**: Delete `~/.config/lazymvn/cache.json`
- **Commands fail**: Verify `mvnw` or `mvn` is executable

### Testing with Real Projects
```bash
# Clone a real multi-module project
git clone https://github.com/spring-projects/spring-petclinic.git
cd spring-petclinic
cargo run -- --project . --debug
```

## Performance Considerations

- POM parsing is cached with hash validation
- Module list cached until POM changes
- Output streaming uses buffered readers
- UI redraws only on state changes or key events

## Security Considerations

- No secrets in code or logs
- Configuration files may contain sensitive paths
- Don't commit `lazymvn-debug.log`
- Sanitize user input in search patterns

## Resources

- [ratatui documentation](https://ratatui.rs/)
- [crossterm documentation](https://docs.rs/crossterm/)
- [Maven CLI reference](https://maven.apache.org/ref/current/maven-embedder/cli.html)
- [Rust API guidelines](https://rust-lang.github.io/api-guidelines/)

---

For detailed contribution process, code of conduct, and feature ideas, see [CONTRIBUTING.md](CONTRIBUTING.md).
