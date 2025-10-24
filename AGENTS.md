# Development Guidelines

This file contains coding guidelines and conventions for contributors and AI agents working on LazyMVN.

> **Note:** For comprehensive contribution guidelines, see [CONTRIBUTING.md](CONTRIBUTING.md)

## Project Structure

```
lazymvn/
├── src/                  # Source code
│   ├── main.rs           # Entry point, TUI setup, main event loop
│   ├── config.rs         # Configuration file loading (lazymvn.toml)
│   ├── maven/            # Maven command execution and detection
│   │   ├── command.rs    # Command building and execution
│   │   ├── detection.rs  # Spring Boot/exec:java detection
│   │   ├── process.rs    # Process management
│   │   └── profiles.rs   # Profile loading
│   ├── project.rs        # POM parsing, module discovery, caching
│   ├── tui.rs            # TUI coordination and rendering
│   ├── utils.rs          # Utilities (log parsing, cleaning)
│   ├── logger.rs         # Logging system
│   ├── watcher.rs        # File watching for live reload
│   ├── starters.rs       # Spring Boot starter management
│   ├── favorites.rs      # Favorites management
│   ├── history.rs        # Command history
│   └── ui/               # UI components
│       ├── keybindings/  # Key event handling
│       ├── state/        # Application state management
│       │   ├── mod.rs
│       │   └── project_tab.rs  # Per-tab state
│       ├── panes/        # UI pane rendering
│       ├── search.rs     # Search functionality
│       └── theme.rs      # Colors and styles
├── docs/                 # Documentation (features, implementations, design)
│   ├── README.md         # Documentation index
│   ├── DEBUG_YANK.md     # Debug yank feature
│   ├── TABS_*.md         # Tab system documentation
│   ├── LOGGING_CONFIG.md # Logging configuration
│   └── ...               # Other feature docs
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
```

## Project Organization

### Documentation (`docs/`)
All feature documentation, implementation notes, and design decisions are in the `docs/` directory.
See [docs/README.md](docs/README.md) for a complete index.

**Key documentation files:**
- Feature implementations (TABS_*, SPRING_BOOT_LAUNCHER, etc.)
- Bug fixes (FIX_SHARED_STARTER, etc.)
- Configuration guides (LOGGING_CONFIG, PROFILE_ACTIVATION, etc.)

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
```

### Root Directory Files
Only essential files remain in the root:
- `AGENTS.md` - This file (development guidelines)
- `CONTRIBUTING.md` - Contribution process
- `README.md` - User-facing documentation
- `CHANGELOG.md` - Version history
- `Cargo.toml` / `Cargo.lock` - Rust project files

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
