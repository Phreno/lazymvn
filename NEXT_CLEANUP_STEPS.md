# Next Cleanup Steps - Codebase Analysis

## 📊 Current Status

### Files Over 600 Lines (Target Threshold)
| File | Lines | Priority | Complexity |
|------|-------|----------|-----------|
| `src/ui/state/mod.rs` | 835 | **HIGH** | Medium |
| `src/ui/keybindings/mod.rs` | 642 | **HIGH** | Medium |
| `src/tui/mod.rs` | 619 | **MEDIUM** | High |

### Files Close to Threshold (500-600 lines)
| File | Lines | Action Needed |
|------|-------|---------------|
| `src/ui/state/output.rs` | 568 | Monitor |
| `src/ui/state/search.rs` | 534 | Monitor |
| `src/maven/command/executor.rs` | 508 | Monitor |
| `src/ui/state/profiles.rs` | 503 | Monitor |
| `src/ui/state/navigation.rs` | 503 | Monitor |

---

## 🎯 Priority 1: Split `src/ui/state/mod.rs` (835 lines)

### Current Structure
- **Types**: `ModuleOutput`, `OutputMetrics`, `MavenProfile`, `BuildFlag`
- **Main struct**: `TuiState` with 14+ methods
- **Submodules**: Already has 18 submodules handling specific functionality
- **Problem**: Too much in the main mod.rs file - types + glue code

### Proposed Refactoring

#### Option A: Extract Core Types (RECOMMENDED)
```
src/ui/state/
├── mod.rs                  (~200 lines) - TuiState struct, constructors, basic methods
├── types.rs                (~100 lines) - ModuleOutput, OutputMetrics, MavenProfile, BuildFlag
├── process_management.rs   (~100 lines) - kill_running_process, terminate_running_process
├── file_watching.rs        (~80 lines)  - check_file_watcher, command_matches_watch_list
├── preferences.rs          (~50 lines)  - save/load_module_preferences
└── (existing submodules remain)
```

**Benefits**:
- Clear separation of data types from behavior
- Process management isolated
- File watching logic separated
- Preferences handling extracted
- Each file under 200 lines
- TuiState becomes primarily a coordinator

#### Option B: Extract TuiState Impl Methods
Split the large impl block into trait-based groups:
```
src/ui/state/
├── mod.rs              (~150 lines) - Core struct + new()
├── types.rs            (~100 lines) - All type definitions
├── state_lifecycle.rs  (~150 lines) - cleanup, refresh_caches, preferences
├── state_process.rs    (~150 lines) - Process and command management
└── state_monitoring.rs (~100 lines) - File watching, elapsed time
```

### Recommended Action: **Option A**
Cleaner separation, each file has single responsibility.

---

## 🎯 Priority 2: Split `src/ui/keybindings/mod.rs` (642 lines)

### Investigation Needed
Run: `grep -n "^pub fn\|^pub struct\|^pub enum" src/ui/keybindings/mod.rs | head -30`

**Likely candidates for extraction**:
- Key mapping logic
- Event handling
- View-specific bindings
- Help text generation

**Proposed**:
```
src/ui/keybindings/
├── mod.rs          (~150 lines) - Public API, main types
├── mappings.rs     (~200 lines) - Key-to-action mappings
├── handlers.rs     (~200 lines) - Event handling logic
└── views.rs        (~100 lines) - View-specific behavior
```

---

## 🎯 Priority 3: Split `src/tui/mod.rs` (619 lines)

**Note**: This is the main TUI rendering logic - proceed carefully

### Investigation Needed
Run: `grep -n "^pub fn\|^fn\|^impl" src/tui/mod.rs | head -50`

**Likely structure**:
```
src/tui/
├── mod.rs          (~150 lines) - Main App struct, public API
├── event_loop.rs   (~200 lines) - Event handling loop
├── rendering.rs    (~200 lines) - Terminal rendering logic
└── layout.rs       (~100 lines) - Layout management
```

---

## 📋 Recommended Action Plan

### Phase 5: UI State Module Cleanup

#### Step 1: Extract Types from `ui/state/mod.rs`
1. Create `src/ui/state/types.rs`
2. Move: `ModuleOutput`, `OutputMetrics`, `MavenProfile`, `BuildFlag`
3. Update imports in mod.rs
4. Run tests

#### Step 2: Extract Process Management
1. Create `src/ui/state/process_management.rs`
2. Move: `kill_running_process()`, `terminate_running_process()`
3. Update TuiState to use new module
4. Run tests

#### Step 3: Extract File Watching
1. Create `src/ui/state/file_watching.rs`
2. Move: `check_file_watcher()`, `command_matches_watch_list()`, `watch_term_matches()`
3. Update TuiState
4. Run tests

#### Step 4: Extract Preferences
1. Create `src/ui/state/preferences.rs`
2. Move: `save_module_preferences()`, `load_module_preferences()`
3. Update TuiState
4. Run tests

#### Expected Result
- `mod.rs`: ~200 lines (from 835)
- All new files: <150 lines each
- **Total reduction**: ~835 → ~200 in mod.rs (76% reduction)

---

### Phase 6: Keybindings Module Cleanup
(After Phase 5 completion)

1. Analyze `src/ui/keybindings/mod.rs` structure
2. Identify natural split points
3. Extract into focused submodules
4. Target: All files under 300 lines

---

### Phase 7: TUI Module Cleanup
(After Phase 6 completion)

1. Carefully analyze `src/tui/mod.rs`
2. Split rendering from event handling
3. Extract layout logic
4. Target: All files under 300 lines

---

## 🔍 Additional Recommendations

### 1. Library Extraction Opportunities

Based on previous work, these are good candidates:

#### A. Maven Log Analyzer Library (`maven-log-analyzer`)
- **Location**: `src/utils/logger.rs` + log parsing code
- **Purpose**: Parse and analyze Maven build logs
- **Features**:
  - Log level detection
  - Error/warning extraction
  - Build phase tracking
  - Statistics generation
- **Users**: LazyMVN + future tools

#### B. Log Colorizer Library (`log-colorizer`)
- **Location**: Color/ANSI handling in logger
- **Purpose**: Terminal color management
- **Features**:
  - ANSI code handling
  - Log level coloring
  - Pattern-based coloring
- **Users**: Maven log analyzer + general purpose

#### C. Maven Command Builder (Already Extracted!)
- ✅ `crates/maven-command-builder`
- Status: Complete and working

### 2. Files to Monitor (Approaching 500 lines)
- `src/core/project.rs` (475 lines)
- `src/main.rs` (471 lines)
- `src/core/config/types/preferences.rs` (470 lines)

### 3. Testing Strategy
After each refactoring:
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo build --release
```

---

## 📈 Success Metrics

### Current State
- Files >600 lines: **3**
- Largest file: **835 lines**
- Total lines in ui/state/: **5,726 lines**

### Target State (After Phase 5)
- Files >600 lines: **2**
- Largest file: **642 lines**
- ui/state/mod.rs: **~200 lines** (76% reduction)

### Ultimate Target
- **All files under 600 lines**
- **Most files under 400 lines**
- **Clear single responsibility per file**

---

## 🚀 Next Immediate Action

**START WITH**: Phase 5, Step 1 - Extract types from `ui/state/mod.rs`

This is:
- Low risk (just moving type definitions)
- High impact (immediate 100+ line reduction)
- Easy to test (no behavior change)
- Sets up foundation for further refactoring

**Command to start**:
```bash
# 1. Analyze current structure
grep -n "^pub struct\|^pub enum\|^impl" src/ui/state/mod.rs

# 2. Create new types module
touch src/ui/state/types.rs

# 3. Begin extraction
# (Manual editing required)
```

---

## 📝 Notes

- **Don't break working code**: Each step must pass tests
- **One file at a time**: Complete each extraction fully before moving to next
- **Preserve git history**: Use clear, descriptive commit messages
- **Update documentation**: Keep inline docs accurate
- **Library extraction**: Wait until internal organization is stable

---

**Generated**: 2025-11-01
**Status**: Ready for Phase 5 implementation
