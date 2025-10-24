# Fix: Shared Starter Cache Across Tabs

## Problem Description

### Symptoms
When using the multi-tab feature:
1. Open project A 
2. Press `s` to select a Spring Boot starter 
3. Press `Ctrl+T` to open a new tab with project B
4. Press `s` in project B

**Expected:** Should ask to select a starter for project B
**Actual:** Tries to run the starter from project A which may not exist in project B

### Root Cause

The `starters_cache` field was stored globally in `TuiState`:

```rust
pub struct TuiState {
    // ...
    pub starters_cache: crate::starters::StartersCache,  // ❌ Global cache
    // ...
}
```

This meant **all tabs shared the same starter cache**, leading to:
- Project A's starter being used in Project B
- Wrong main class being launched
- Confusion about which starters belong to which project

## Solution

### Architecture Change

Move `starters_cache` from `TuiState` (global) to `ProjectTab` (per-tab):

```rust
pub struct ProjectTab {
    // ...
    pub starters_cache: crate::starters::StartersCache,  // ✅ Per-tab cache
    // ...
}
```

### Implementation Details

1. **ProjectTab Constructor** (`src/ui/state/project_tab.rs`)
   - Load starters cache when creating a new tab
   - Cache is project-specific (based on project_root hash)

2. **TuiState** (`src/ui/state/mod.rs`)
   - Remove global `starters_cache` field
   - Update all methods to use `tab.starters_cache` instead
   - Handle borrow checker issues by extracting data before mutations

3. **Rendering** (`src/tui.rs`)
   - Clone starters when rendering to avoid borrow conflicts
   - Prevents simultaneous immutable (tab) and mutable (list_state) borrows

4. **Tests** (`src/ui/keybindings/mod.rs`)
   - Update test setup to use `get_active_tab_mut().starters_cache`
   - Ensure tests verify per-tab behavior

### Key Code Changes

#### Before (Global Cache)
```rust
pub fn run_preferred_starter(&mut self) {
    if let Some(starter) = self.starters_cache.get_preferred_starter() {
        // Uses global cache - wrong!
        self.run_spring_boot_starter(&starter.fully_qualified_class_name);
    }
}
```

#### After (Per-Tab Cache)
```rust
pub fn run_preferred_starter(&mut self) {
    let tab = self.get_active_tab();
    if let Some(starter) = tab.starters_cache.get_preferred_starter() {
        // Uses tab-specific cache - correct!
        let fqcn = starter.fully_qualified_class_name.clone();
        self.run_spring_boot_starter(&fqcn);
    }
}
```

### Borrow Checker Challenges

Several methods required restructuring to satisfy Rust's borrow checker:

**Problem:** Can't have both `&mut tab` and `&mut self.list_state` at the same time

**Solution:** Extract data from tab before releasing the borrow:

```rust
pub fn remove_selected_starter(&mut self) {
    if let Some(idx) = self.starters_list_state.selected() {
        // Scope the tab borrow
        let (removed, new_len) = {
            let tab = self.get_active_tab_mut();
            // ... do work with tab ...
            (removed, tab.starters_cache.starters.len())
        }; // tab borrow ends here
        
        // Now we can modify self.starters_list_state
        if removed {
            self.starters_list_state.select(...);
        }
    }
}
```

## Testing

### Automated Tests
All existing tests pass:
- `cargo test` - 119 passed
- `cargo clippy -- -D warnings` - 0 warnings

### Manual Testing

Use the test script:
```bash
./scripts/test-starter-isolation.sh
```

Or manually:
1. `cargo run -- --project demo/multi-module`
2. Press `s`, select a starter (e.g., `App1`)
3. Press `Ctrl+T`, open another project
4. Press `s` - should show selector, not reuse `App1`

### Regression Prevention

To verify the fix works with real projects:
1. Open project A with Spring Boot
2. Select and save a starter
3. Open project B (different Spring Boot app)
4. Press `s` - should not auto-run project A's starter

## Cache File Structure

Cache files remain unchanged (backward compatible):
- Location: `~/.config/lazymvn/starters/<project_hash>.json`
- Each project still has its own cache file
- Only the in-memory representation changed (moved to ProjectTab)

Example cache:
```json
{
  "starters": [
    {
      "fully_qualified_class_name": "fr.company.some.assemblage.ApplicationStarter",
      "label": "ApplicationStarter",
      "is_default": true
    }
  ],
  "last_used": "fr.company.some.assemblage.ApplicationStarter"
}
```

## Impact

### User-Visible Changes
- ✅ Each tab now correctly remembers its own starters
- ✅ No more confusion when switching between projects
- ✅ Starter selection is project-specific (as intended)

### Code Quality
- ✅ Better encapsulation (tab owns its data)
- ✅ Clearer ownership model
- ✅ More maintainable (data follows project lifecycle)

### Performance
- No significant impact
- Cache still loaded once per tab (on tab creation)
- Minimal memory overhead (starters list is small)

## Related Files

- `src/ui/state/project_tab.rs` - Tab structure with starters_cache
- `src/ui/state/mod.rs` - State management without global cache
- `src/tui.rs` - Rendering with borrow-safe approach
- `src/ui/keybindings/mod.rs` - Keybinding handlers and tests
- `src/starters.rs` - StartersCache implementation (unchanged)

## Future Improvements

Potential enhancements:
1. **Starter Discovery**: Auto-detect starters when switching tabs
2. **Starter Validation**: Warn if cached starter no longer exists
3. **Cross-Project Sharing**: Allow copying starter config between tabs
4. **UI Indicator**: Show which tab has which starter active

## Lessons Learned

1. **Tab Isolation**: Each tab should own its project-specific data
2. **Borrow Checker**: Use scopes and data extraction to manage borrows
3. **Testing**: Update tests when moving data between structs
4. **Documentation**: Explain ownership model changes clearly

---

**Status**: ✅ Fixed and tested  
**Version**: Committed in `cba76f2`  
**Branch**: `fix/shared-starter`
