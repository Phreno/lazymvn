# Fix: Profile Loading for New Tabs

## Problem

When opening a new tab with a different Maven project using `Ctrl+T`, the Maven profiles were not loaded for the new tab. The profiles list remained empty, preventing users from selecting and activating profiles for the newly opened project.

### Symptoms

1. Open LazyMVN with a project (profiles load correctly)
2. Press `Ctrl+T` to create a new tab
3. Select a different project from the recent projects list
4. The new tab opens but the profiles list is empty
5. User cannot select Maven profiles for the new project

### Root Cause

The `create_tab()` method in `src/ui/state/mod.rs` was creating a new `ProjectTab` instance but never triggering the asynchronous profile loading. The `start_loading_profiles()` method was only called:

- At application startup (in `main.rs`)
- When reloading a project (also in `main.rs`)

But **not** when creating a new tab via the tab management system.

## Solution

Added a call to `start_loading_profiles()` at the end of the `create_tab()` method, right after the new tab is created and set as active.

### Code Changes

**File:** `src/ui/state/mod.rs`

```rust
pub fn create_tab(&mut self, project_root: PathBuf) -> Result<usize, String> {
    // ... existing code ...
    
    // Add to recent projects
    let mut recent = crate::config::RecentProjects::load();
    recent.add(resolved_root);
    self.recent_projects = recent.get_projects();

    // Load profiles asynchronously for the new tab  <-- NEW
    self.start_loading_profiles();                  <-- NEW

    Ok(self.active_tab_index)
}
```

### How It Works

1. User presses `Ctrl+T` to create a new tab
2. User selects a project from the recent projects list
3. `create_tab()` is called with the project path
4. A new `ProjectTab` is created with empty profiles list
5. The tab becomes the active tab
6. `start_loading_profiles()` is called
7. Profile loading happens asynchronously in background thread
8. When complete, profiles are populated via the receiver channel
9. UI shows the loading spinner during the async operation

### Testing

To verify the fix:

1. Build the project: `cargo build --release`
2. Start LazyMVN with any Maven project
3. Wait for profiles to load (you should see profiles in the Profiles view)
4. Press `Ctrl+T` to create a new tab
5. Select a different Maven project from the list
6. **Expected:** Loading spinner appears briefly, then profiles are populated
7. Press `p` to switch to Profiles view
8. **Expected:** Profiles list shows all available profiles for the new project

### Related Code

- `src/ui/state/mod.rs`: `create_tab()` method (line ~306)
- `src/ui/state/mod.rs`: `start_loading_profiles()` method (line ~1289)
- `src/ui/state/project_tab.rs`: `ProjectTab::new()` method (line ~68)
- `src/maven/profiles.rs`: `get_profiles()` function
- `src/main.rs`: Initial profile loading (line ~225)

### Impact

- **User Experience:** Profiles now load automatically for all new tabs
- **Performance:** No impact - profile loading was already asynchronous
- **Compatibility:** No breaking changes to API or behavior
- **Tab Switching:** When switching between existing tabs, profiles remain loaded

## Status

- ✅ **Fixed** in this commit
- ✅ Compiles successfully
- ✅ Clippy clean (no warnings)
- ✅ Ready for merge

## Branch

- Branch: `fix/profile-loading`
- Commit: Profile loading now triggered for new tabs
