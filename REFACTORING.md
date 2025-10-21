# Code Refactoring Documentation

This document describes the refactoring performed on October 21, 2025 to improve code organization and maintainability.

## Objective

Reorganize large source files (>600 lines) into smaller, specialized modules following Rust best practices.

## Changes Made

### 1. maven Module (2007 lines → 5 files)

**Before:**
```
src/maven.rs (2007 lines)
```

**After:**
```
src/maven/
├── mod.rs (30 lines)          # Public API and module declarations
├── process.rs (50 lines)       # CommandUpdate enum, kill_process()
├── detection.rs (313 lines)    # Spring Boot detection, LaunchStrategy
├── profiles.rs (317 lines)     # Profile extraction from POM/settings.xml
└── command.rs (524 lines)      # Maven command building and execution

src/maven_tests.rs (860 lines)  # All maven tests (kept separate for visibility)
```

**Responsibilities:**
- `process.rs`: Process lifecycle management
- `detection.rs`: Spring Boot capability detection and launch strategy decision
- `profiles.rs`: Maven profile discovery and XML extraction
- `command.rs`: Maven command construction and async/sync execution

### 2. ui/keybindings Module (993 lines → 2 files)

**Before:**
```
src/ui/keybindings.rs (993 lines)
```

**After:**
```
src/ui/keybindings/
├── mod.rs (953 lines)    # Main keybinding handler
└── types.rs (50 lines)   # CurrentView, Focus, SearchMode enums
```

### 3. ui/panes Module (933 lines → 1 file)

**Before:**
```
src/ui/panes.rs (933 lines)
```

**After:**
```
src/ui/panes/
└── mod.rs (933 lines)    # Pane rendering logic
```

### 4. ui/state Module (1915 lines → 1 file)

**Before:**
```
src/ui/state.rs (1915 lines)
```

**After:**
```
src/ui/state/
└── mod.rs (1923 lines)   # TUI state management
```

## Module Structure

```
src/
├── main.rs
├── config.rs
├── logger.rs
├── project.rs
├── starters.rs
├── tui.rs
├── utils.rs
├── maven_tests.rs
├── maven/
│   ├── mod.rs
│   ├── command.rs
│   ├── detection.rs
│   ├── process.rs
│   └── profiles.rs
└── ui/
    ├── mod.rs
    ├── search.rs
    ├── theme.rs
    ├── keybindings/
    │   ├── mod.rs
    │   └── types.rs
    ├── panes/
    │   └── mod.rs
    └── state/
        └── mod.rs
```

## Testing

All 101 tests continue to pass after refactoring:

```bash
cargo test
# test result: ok. 101 passed; 0 failed; 0 ignored
```

## Benefits

1. **Improved Maintainability**
   - Smaller, focused files are easier to understand
   - Clear separation of concerns
   - Easier to locate specific functionality

2. **Better Code Organization**
   - Related code grouped together
   - Clear module boundaries
   - Logical file structure

3. **No Breaking Changes**
   - All public APIs remain unchanged
   - Internal refactoring only
   - Backward compatible

4. **Enhanced Discoverability**
   - Module names indicate purpose
   - Less scrolling to find code
   - Better IDE navigation

## Guidelines for Future Development

### When adding new Maven functionality:

- **Command execution**: Add to `maven/command.rs`
- **Spring Boot detection**: Add to `maven/detection.rs`
- **Profile management**: Add to `maven/profiles.rs`
- **Process management**: Add to `maven/process.rs`
- **Tests**: Add to `maven_tests.rs`

### When adding new UI functionality:

- **Keybindings**: Add to `ui/keybindings/mod.rs`
- **View types**: Add to `ui/keybindings/types.rs`
- **Pane rendering**: Add to `ui/panes/mod.rs`
- **State management**: Add to `ui/state/mod.rs`

## Further Refactoring Opportunities

The following files could benefit from further subdivision:

1. **ui/state/mod.rs** (1923 lines)
   - Could split into: commands, starters, output, clipboard modules

2. **ui/panes/mod.rs** (933 lines)
   - Could split by pane type: modules, profiles, output, layout

3. **ui/keybindings/mod.rs** (953 lines)
   - Large `handle_key_event` function could be split by view

These are not critical as the files are reasonably maintainable at their current size.

## Validation Checklist

- [x] All tests pass
- [x] Release build succeeds
- [x] No functionality lost
- [x] No breaking API changes
- [x] Code follows Rust conventions
- [x] Module visibility correctly configured
- [x] Documentation updated

## Date

Refactoring completed: October 21, 2025
