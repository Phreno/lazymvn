# Phase 4: File Size Reduction Through Refactoring

## 🎯 Goal
Reduce all files to under 600 lines through strategic module splitting, while keeping code modular and maintainable.

## 📊 Target Files (>600 lines)

| File | Current Lines | Target | Strategy |
|------|---------------|--------|----------|
| `src/maven/command.rs` | 1114 | <600 | Split into execution + building |
| `src/maven/detection.rs` | 941 | <600 | Split into detection + parsing + strategies |
| `src/ui/state/mod.rs` | 836 | <600 | Already has submodules, reorganize |
| `src/ui/keybindings/mod.rs` | 646 | <600 | Split handlers from definitions |
| `src/tui/mod.rs` | 619 | <600 | Split rendering from state |

## 🔧 Refactoring Strategy

### 1. `src/maven/command.rs` (1114 lines) → Split into 3 files

**Current structure**: Everything in one file
- Command building logic
- Execution logic  
- Log4j configuration
- Output processing

**New structure**:
```
src/maven/
├── command/
│   ├── mod.rs           (~200 lines) - Public API & re-exports
│   ├── builder.rs       (~300 lines) - Command building logic
│   ├── executor.rs      (~400 lines) - Execution & output streaming
│   └── log4j_config.rs  (~200 lines) - Log4j configuration extraction
```

**Benefits**:
- Clear separation of concerns
- Easier to test each component
- Better code organization

---

### 2. `src/maven/detection.rs` (941 lines) → Split into 4 files

**Current structure**: All detection logic in one file
- Spring Boot detection
- Launch strategy
- POM parsing
- Plugin detection

**New structure**:
```
src/maven/
├── detection/
│   ├── mod.rs           (~150 lines) - Public API & types
│   ├── spring_boot.rs   (~300 lines) - Spring Boot specific
│   ├── pom_parser.rs    (~300 lines) - POM XML parsing
│   └── strategies.rs    (~200 lines) - Launch strategies
```

**Benefits**:
- Each file has single responsibility
- Spring Boot logic isolated
- POM parsing reusable
- Strategy pattern clear

---

### 3. `src/ui/state/mod.rs` (836 lines) → Better organization

**Current structure**: Large mod.rs with submodules
**Analysis needed**: Check if already well-split

**Potential actions**:
- Move more logic to existing submodules
- Create new submodules for large functions
- Extract state transitions

---

### 4. `src/ui/keybindings/mod.rs` (646 lines) → Split into 3 files

**Current structure**: All keybinding logic in one file

**New structure**:
```
src/ui/keybindings/
├── mod.rs           (~200 lines) - Public API & dispatcher
├── definitions.rs   (~250 lines) - Key mappings & config
└── handlers.rs      (~250 lines) - Action handlers
```

---

### 5. `src/tui/mod.rs` (619 lines) → Split into 2-3 files

**Current structure**: TUI rendering in one file

**New structure**:
```
src/tui/
├── mod.rs           (~200 lines) - Public API & setup
├── rendering.rs     (~300 lines) - Main rendering logic
└── helpers.rs       (~200 lines) - Utility functions
```

---

## 📋 Execution Order

### Phase 4A: Maven Module Refactoring
1. ✅ Split `maven/command.rs`
2. ✅ Split `maven/detection.rs`
3. ✅ Test all Maven functionality

**Estimated time**: 2-3 hours

---

### Phase 4B: UI Module Refactoring
4. ✅ Analyze `ui/state/mod.rs`
5. ✅ Split `ui/keybindings/mod.rs`
6. ✅ Split `tui/mod.rs`
7. ✅ Test all UI functionality

**Estimated time**: 2-3 hours

---

## ✅ Success Criteria

- [ ] All files under 600 lines
- [ ] All tests passing
- [ ] No functionality lost
- [ ] cargo clippy clean
- [ ] cargo build successful
- [ ] Better code organization
- [ ] Easier to navigate codebase

---

## 🎯 Benefits of This Approach

### vs. Library Extraction:
✅ **Faster**: No need to design public APIs  
✅ **Safer**: Keep internal dependencies  
✅ **Flexible**: Can still extract libraries later  
✅ **Pragmatic**: Solves the immediate problem  
✅ **Incremental**: Can refactor piece by piece  

### Code Quality:
✅ **More modular**: Each file has clear purpose  
✅ **Easier testing**: Smaller units to test  
✅ **Better navigation**: Find code faster  
✅ **Reduced cognitive load**: Smaller files easier to understand  

---

## 🚀 Let's Start!

**Next step**: Begin with Phase 4A - Maven module refactoring

Command to start:
```bash
# 1. Split maven/command.rs
mkdir -p src/maven/command

# 2. Split maven/detection.rs  
mkdir -p src/maven/detection
```

This approach gives us:
- ✅ Cleaner codebase (all files < 600 lines)
- ✅ Better organization
- ✅ Foundation for future library extraction
- ✅ Maintained functionality
- ✅ No breaking changes

