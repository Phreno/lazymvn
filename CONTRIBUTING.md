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
3. **Add doc comments** for public APIs:
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

## Pull Request Process

### Before Submitting

1. ✅ Code builds without errors: `cargo build`
2. ✅ All tests pass: `cargo test`
3. ✅ Code is formatted: `cargo fmt`
4. ✅ No clippy warnings: `cargo clippy -- -D warnings`
5. ✅ Documentation is updated
6. ✅ CHANGELOG.md is updated

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

Understanding the codebase:

```
src/
├── main.rs              # Entry point, TUI setup, main loop
├── config.rs            # Configuration file loading
├── project.rs           # POM parsing, module discovery, caching
├── maven.rs             # Maven command execution
├── tui.rs               # TUI coordination and drawing
├── utils.rs             # Utilities (log parsing, cleaning)
└── ui/
    ├── keybindings.rs   # Key event handling, navigation
    ├── state.rs         # Application state management
    ├── panes.rs         # UI pane rendering
    ├── search.rs        # Search functionality
    └── theme.rs         # Colors and styles
```

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
