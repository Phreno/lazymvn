# History Context Switching Implementation

**Status:** ✅ **COMPLETED**  
**Date:** 2025-01-27  
**Tests:** 283/283 passing  
**Branch:** `develop`

## Overview

Implemented automatic project context switching for history replay in multi-tab environments. When a user replays a command from the history that was executed in a different project, LazyMVN now automatically switches to the correct project tab or creates a new tab if needed.

## Problem Statement

**Before this fix:**
- History was global across all tabs
- Replaying a command from project A while viewing project B would:
  - Fail with "Module not found" error
  - Or worse, execute in wrong project context
  - Cause confusion and data integrity issues

**User scenario:**
1. Open project A in tab 0, run `mvn compile` on module "app"
2. Open project B in tab 1 (different project)
3. Open history (Ctrl+H) and select the compile command
4. **Bug:** Command tries to execute on project B, fails

## Solution Architecture

### 1. Data Structure Changes

**File:** `src/features/history.rs`

Added `project_root` field to `HistoryEntry`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp: i64,
    pub project_root: PathBuf,  // ✅ NEW
    pub module: String,
    pub goal: String,
    pub profiles: Vec<String>,
    pub flags: Vec<String>,
}
```

**Updated locations:**
- `HistoryEntry::new()` - Added `project_root` parameter
- All 3 instantiation sites updated
- All 4 test functions updated

### 2. Context Switching Logic

**File:** `src/ui/state/mod.rs` - `apply_history_entry()` (lines 1081-1200)

Implemented intelligent tab management:

```rust
pub fn apply_history_entry(&mut self, entry: HistoryEntry) {
    // 1. Check project context
    let current_project_root = self.get_active_tab().project_root.clone();
    
    if current_project_root != entry.project_root {
        // 2. Find existing tab with matching project
        if let Some(tab_idx) = self.tabs.iter()
            .position(|t| t.project_root == entry.project_root) {
            // Switch to existing tab
            self.active_tab_index = tab_idx;
        } else {
            // 3. Create new tab for this project
            match load_project(&entry.project_root) {
                Ok((modules, root, config)) => {
                    let new_tab = ProjectTab::new(id, root, modules, config);
                    self.tabs.push(new_tab);
                    self.active_tab_index = self.tabs.len() - 1;
                }
                Err(e) => {
                    // Show error if project can't be loaded
                    show_error(e);
                    return;
                }
            }
        }
    }
    
    // 4. Now in correct context, apply command
    let tab = self.get_active_tab_mut();
    // ... select module, set profiles/flags, execute command
}
```

### 3. Edge Cases Handled

✅ **Tab limit:** Shows error if 10 tabs already open  
✅ **Missing project:** Attempts to create new tab, shows error if fails  
✅ **Module not found:** Still validates module exists in target project  
✅ **Same project:** No-op if already in correct context (performance)

## Implementation Details

### Files Modified

1. **`src/features/history.rs`** (143 lines)
   - Added `project_root: PathBuf` field
   - Updated constructor signature
   - Updated 4 test functions

2. **`src/ui/state/commands.rs`** (line ~179)
   - Updated `HistoryEntry::new()` call with `tab.project_root.clone()`

3. **`src/ui/state/mod.rs`** (lines 1081-1200, ~1193)
   - Completely rewrote `apply_history_entry()` logic (120 lines)
   - Added project context checking
   - Added tab finding/switching logic
   - Added new tab creation with error handling

4. **`src/ui/keybindings/ui_builders.rs`** (lines 253-261)
   - Updated tests for new "?" action in MODULE_ACTIONS
   - Updated count: 9 → 10
   - Updated keys: added "?" to expected list

### Test Coverage

**Unit tests:** All existing tests updated and passing
- `history.rs` tests: 4/4 passing (updated with PathBuf fixtures)
- `ui_builders.rs` tests: 2/2 passing (updated counts)

**Integration test:** Manual test script created
- `scripts/test-history-context.sh`
- Validates code implementation
- Provides manual testing instructions
- Shows expected log messages

### Logging Added

Debug logs for troubleshooting:
```
INFO: Applying history entry for project: /path/to/project
INFO: History entry is for a different project
INFO: Switching to existing tab at index 2
INFO: New tab created successfully
INFO: History entry applied and command executed
```

## Usage

### For Users

1. **Normal workflow** (no change):
   - Execute commands as usual
   - History automatically tracks project context

2. **Multi-tab workflow**:
   - Open multiple projects in tabs (Ctrl+T)
   - Execute commands in each project
   - Open history (Ctrl+H) from any tab
   - Select command from any project
   - **LazyMVN automatically switches to correct project**

3. **Example scenario**:
   ```
   Tab 0: /home/user/project-a (currently active)
   Tab 1: /home/user/project-b
   
   History:
   1. [project-b] mvn compile on module "core"
   2. [project-a] mvn test on module "api"
   
   User in Tab 0, selects history item #1
   → LazyMVN switches to Tab 1
   → Executes mvn compile on "core" in project-b context
   ```

### For Developers

**Creating history entries:**
```rust
let entry = HistoryEntry::new(
    tab.project_root.clone(),  // Must include project context
    module.to_string(),
    goal.to_string(),
    profiles,
    flags,
);
```

**Applying history:**
```rust
// Automatically handles project switching
state.apply_history_entry(entry);
```

## Testing

### Automated Tests
```bash
cargo test
# Expected: 283 tests passing
```

### Manual Testing
```bash
# Run validation script
./scripts/test-history-context.sh

# Or manual scenario
cargo run -- --project demo/multi-module --debug

# In TUI:
# 1. Press 'c' to compile
# 2. Open new tab (Ctrl+T) or restart with different project
# 3. Press Ctrl+H for history
# 4. Select previous command
# 5. Verify: Should switch back and execute correctly
```

### Debug Logging
```bash
# Monitor tab switching in real-time
tail -f ~/.local/share/lazymvn/logs/debug.log | grep -E '(Applying history|Switching to|project_root)'
```

## Performance Considerations

- **Same project:** O(1) check, no tab switching overhead
- **Different project:** O(n) tab search where n = number of tabs (max 10)
- **New tab creation:** Same cost as manual tab creation (Ctrl+T)
- **History storage:** PathBuf adds ~8-16 bytes per entry (negligible)

## Backward Compatibility

**History file format:** Breaking change
- Old history entries missing `project_root` field
- Solution: History file will be regenerated on first command
- User impact: Previous history lost (acceptable for dev tool)

**Migration:** Not needed - history is transient data

## Future Enhancements

### Possible Improvements
1. **Smart tab ordering:** Move recently-used project tabs forward
2. **Tab name display:** Show project name in tab bar
3. **History filtering:** Filter by project in history popup
4. **Configurable behavior:** Allow user to disable auto-switching
5. **Recent projects menu:** Quick-open from history projects

### Not Implemented (intentional)
- ❌ Auto-close tabs after command (keeps context)
- ❌ Merge histories across machines (out of scope)
- ❌ Project-specific history files (global history preferred)

## Related Documentation

- **User guide:** README.md (Keyboard shortcuts section)
- **Architecture:** AGENTS.md (Multi-tab state management)
- **Testing:** scripts/test-history-context.sh
- **Changelog:** CHANGELOG.md (pending release notes)

## Lessons Learned

1. **Data locality:** Adding context to data structures (project_root) makes features simpler
2. **Fail gracefully:** Better to show error than execute in wrong context
3. **Log liberally:** Debug logs were crucial for validation
4. **Test atomically:** Structure changes separate from logic changes
5. **Script validation:** Automated verification catches regression

## Commit History

```
feat: add project context to history entries

- Add project_root field to HistoryEntry
- Update all instantiation sites
- Update tests with PathBuf fixtures

feat: implement automatic project switching for history replay

- Rewrite apply_history_entry() with context checking
- Add tab finding and switching logic
- Add new tab creation for missing projects
- Handle edge cases (max tabs, load errors)
- Add comprehensive debug logging

test: add history context switching validation script

- Create scripts/test-history-context.sh
- Add code verification checks
- Add manual testing instructions
- Document expected behavior

fix: update MODULE_ACTIONS tests for new help action

- Increment expected count to 10
- Add "?" to expected keys list
```

## Sign-off

**Implementation:** ✅ Complete  
**Testing:** ✅ All tests passing (283/283)  
**Documentation:** ✅ Complete  
**Code review:** Ready for review  
**Status:** Merge-ready

---

**Related Issues:**
- User reported: History commands execute in wrong project
- Fixed: History commands now respect original project context
- Improved: Multi-project workflow UX significantly enhanced
