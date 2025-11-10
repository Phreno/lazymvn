# 📊 LazyMVN File Size Refactoring Plan

**Generated:** 2025-11-10  
**Goal:** Keep files under 300 lines, maximum 400 lines

---

## 📈 Current Statistics

- **Total Files:** 96
- **Total Lines:** 21,761
- **Average:** 226 lines/file
- **🔴 Large (>500):** 14 files
- **🟡 Medium (300-500):** 14 files  
- **🟢 Small (<300):** 67 files (70% - good!)

---

## 🎯 Refactoring Priorities

### Phase 1: High Impact, Low Risk ⭐

#### 1. **features/history.rs** (619 lines → 3 files)
**Why First:** Already has 47 functions with clear separations, lots of pure helpers

**Split Plan:**
```
features/history/
├── mod.rs               (100 lines) - Public API, CommandHistory struct
├── storage.rs           (200 lines) - File I/O, load/save operations
├── filtering.rs         (200 lines) - Search, filter, match logic
└── helpers.rs           (100 lines) - Pure functions (format_*, normalize_*)
```

**Functions to Move:**
- `storage.rs`: `load_cache_if_exists`, `save_history_to_file`, `get_history_file_path`, `ensure_parent_dir_exists`
- `filtering.rs`: `filter_by_query`, `matches_search_query`, `find_matching_entry_index`, `remove_duplicate_entry`
- `helpers.rs`: `format_command`, `format_time`, `format_profiles`, `format_module_name`, `normalize_*`

**Tests:** 24 existing tests - split accordingly

---

#### 2. **utils/logger.rs** (622 lines → 4 files)
**Why Second:** 59 functions! Clear responsibility boundaries

**Split Plan:**
```
utils/logger/
├── mod.rs               (100 lines) - Logger struct, public API
├── core.rs              (200 lines) - Main logging impl, format_log_line
├── session.rs           (200 lines) - Session tracking, read_session_logs
├── rotation.rs          (150 lines) - File rotation, cleanup_old_logs
└── paths.rs             (70 lines)  - Path management helpers
```

**Functions to Move:**
- `core.rs`: `Logger` impl, `format_log_line`, `write_to_*_file`, `get_current_timestamp`
- `session.rs`: `read_session_logs`, `build_session_marker`, `collect_session_lines`, `add_*_logs`
- `rotation.rs`: `rotate_log_file`, `cleanup_old_logs`, `should_delete_old_log`
- `paths.rs`: `get_log_dir`, `get_*_log_path`, `ensure_dir_exists`

---

#### 3. **core/project.rs** (573 lines → 4 files)
**Why Third:** Already has good function organization, clear domains

**Split Plan:**
```
core/project/
├── mod.rs               (80 lines)  - Public API
├── discovery.rs         (150 lines) - find_pom, search_pom_upward
├── parsing.rs           (200 lines) - parse_modules_from_str, XML handling
├── cache.rs             (200 lines) - Cache management, load/save
└── helpers.rs           (100 lines) - Pure functions (normalize, hash, etc.)
```

**Functions to Move:**
- `discovery.rs`: `find_pom`, `search_pom_upward`, `has_parent_dir`
- `parsing.rs`: `parse_modules_from_str`, `create_xml_reader`, `is_module_tag`, `add_module_text`
- `cache.rs`: All cache-related functions (10+)
- `helpers.rs`: `normalize_modules`, `compute_pom_hash`, `format_*`

---

### Phase 2: Medium Impact 🔸

#### 4. **ui/state/navigation.rs** (580 lines → expand existing split)
**Current:** Already has helper functions extracted  
**Action:** Move more complex logic to helpers, create selection.rs

```
ui/state/navigation/
├── mod.rs               (200 lines) - NavigationState struct
├── helpers.rs           (200 lines) - EXISTING pure functions
├── selection.rs         (150 lines) - Selection management, jumping
└── scrolling.rs         (100 lines) - Scroll calculations
```

---

#### 5. **ui/state/output.rs** (641 lines → expand existing split)
**Current:** Already has helper functions extracted  
**Action:** Extract clipboard and formatting

```
ui/state/output/
├── mod.rs               (200 lines) - OutputState struct
├── helpers.rs           (150 lines) - EXISTING pure functions
├── formatting.rs        (200 lines) - Format output, colors, styles
└── clipboard.rs         (100 lines) - Clipboard operations
```

---

#### 6. **maven/command/executor.rs** (667 lines → 4 files)
**Why Later:** More complex dependencies, needs careful splitting

```
maven/command/executor/
├── mod.rs               (150 lines) - Main execute functions
├── async_exec.rs        (200 lines) - Async execution
├── streaming.rs         (150 lines) - read_lines_lossy, output handling
└── display.rs           (150 lines) - build_command_display + tests
```

---

#### 7. **ui/search.rs** (686 lines → 4 files)
**Why Later:** Complex with 50 functions, needs careful analysis

```
ui/search/
├── mod.rs               (150 lines) - SearchResults struct
├── matching.rs          (200 lines) - find_matches_in_line, fuzzy logic
├── rendering.rs         (200 lines) - search_line_style, highlights
└── formatting.rs        (150 lines) - Status line, prompts
```

---

### Phase 3: Consider Later 💭

These files are candidates but lower priority:
- `ui/keybindings/mod.rs` (642 lines) - Already in a module, just needs internal splitting
- `maven/command/builder.rs` (534 lines) - Complex but cohesive
- `maven/detection/spring_boot.rs` (524 lines) - Recently refactored, has pure functions
- `tui/mod.rs` (608 lines) - Main TUI loop, splitting might reduce cohesion

---

## 🛠️ Refactoring Process (Per File)

### Step 1: Analyze
```bash
# Count functions
grep -c "^fn \|^pub fn " filename.rs

# List all functions
grep "^fn \|^pub fn " filename.rs
```

### Step 2: Create Module Structure
```bash
mkdir -p src/path/to/module
touch src/path/to/module/mod.rs
touch src/path/to/module/submodule.rs
```

### Step 3: Extract Pure Functions First
- Move to `helpers.rs`
- No state dependencies
- Easy to test
- Low risk

### Step 4: Extract Distinct Responsibilities
- Identify clear boundaries (I/O, parsing, formatting, etc.)
- Move related functions together
- Maintain public API in mod.rs

### Step 5: Update Tests
- Move tests with their functions
- Keep integration tests separate
- Ensure all tests still pass

### Step 6: Update Imports
```bash
# Before
use crate::features::history::*;

# After
use crate::features::history::{CommandHistory, filtering};
```

### Step 7: Verify
```bash
cargo build
cargo test
cargo clippy
```

---

## 📋 Tracking Progress

### Completed ✅
- [x] Initial refactoring (pure functions extracted in some files)
- [x] All clippy warnings fixed
- [x] All tests passing

### Phase 1 - Immediate 🎯
- [ ] features/history.rs → features/history/
- [ ] utils/logger.rs → utils/logger/
- [ ] core/project.rs → core/project/

### Phase 2 - Next 🔸
- [ ] ui/state/navigation.rs (expand)
- [ ] ui/state/output.rs (expand)
- [ ] maven/command/executor.rs → maven/command/executor/
- [ ] ui/search.rs → ui/search/

### Phase 3 - Future 💭
- [ ] ui/keybindings/mod.rs (split internally)
- [ ] tui/mod.rs (consider splitting)
- [ ] maven/command/builder.rs (if needed)

---

## 🎁 Benefits

### Developer Experience
- ✅ **Easier Navigation:** Find code faster in smaller files
- ✅ **Reduced Cognitive Load:** Understand one responsibility at a time
- ✅ **Better Tests:** Test individual components in isolation
- ✅ **Clearer Dependencies:** See what depends on what

### Code Quality
- ✅ **Better Organization:** Clear separation of concerns
- ✅ **More Testable:** Pure functions are easy to test
- ✅ **Easier Maintenance:** Changes are localized
- ✅ **Better Compilation:** Smaller units compile faster

### Team Collaboration
- ✅ **Less Merge Conflicts:** Changes in different files
- ✅ **Easier Code Review:** Smaller, focused changes
- ✅ **Better Onboarding:** New developers find code easier

---

## 📝 Example: Refactoring history.rs

### Before (619 lines, single file)
```rust
// features/history.rs
pub struct CommandHistory { ... }
impl CommandHistory { ... }
pub fn format_command(...) { ... }
fn filter_by_query(...) { ... }
fn save_history_to_file(...) { ... }
// ... 47 functions
```

### After (4 files, max 200 lines each)
```rust
// features/history/mod.rs (100 lines)
pub struct CommandHistory { ... }
pub use storage::{load, save};
pub use filtering::{filter_by_query};
pub use helpers::{format_command};

// features/history/storage.rs (200 lines)
pub fn load(...) { ... }
pub fn save(...) { ... }
fn get_history_file_path() { ... }

// features/history/filtering.rs (200 lines)  
pub fn filter_by_query(...) { ... }
fn matches_search_query(...) { ... }

// features/history/helpers.rs (100 lines)
pub fn format_command(...) { ... }
pub fn format_time(...) { ... }
```

---

## 🚀 Getting Started

**Recommended First Step:**
```bash
# Start with features/history.rs - clearest separation
cd src/features
mkdir history
touch history/mod.rs history/storage.rs history/filtering.rs history/helpers.rs

# Extract pure functions to helpers.rs first (lowest risk)
# Then extract storage functions
# Then filtering logic
# Finally update mod.rs as public API
```

**Commands to track progress:**
```bash
# Check file sizes
find src -name "*.rs" -exec wc -l {} + | sort -rn | head -20

# Count files over 500 lines
find src -name "*.rs" -exec wc -l {} + | awk '$1 > 500' | wc -l

# Average file size
find src -name "*.rs" -exec wc -l {} + | awk '{sum+=$1; count++} END {print sum/count}'
```

---

## 📞 Need Help?

- Check existing small files for patterns (those <200 lines)
- Look at how standard library organizes code
- Remember: **Small files are easier to understand, test, and maintain**
- Goal: Make each file tell one clear story

**Happy Refactoring! 🎉**
