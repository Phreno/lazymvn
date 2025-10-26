# Phase 2 Migration: Field Access Refactoring

## Status
- **Current:** 203 compilation errors remaining  
- **Issue:** All methods accessing moved fields need refactoring

## Fields Moved to ProjectTab

These fields were moved from `TuiState` to `ProjectTab`:
- `modules: Vec<String>`
- `profiles: Vec<MavenProfile>`
- `flags: Vec<BuildFlag>`
- `modules_list_state: ListState`
- `profiles_list_state: ListState`
- `flags_list_state: ListState`
- `command_output: Vec<String>`
- `output_offset: usize`
- `output_view_height: u16`
- `output_area_width: u16`
- `output_metrics: Option<OutputMetrics>`
- `module_outputs: HashMap<String, ModuleOutput>`
- `project_root: PathBuf`
- `config: Config`
- `command_receiver: Option<Receiver<CommandUpdate>>`
- `is_command_running: bool`
- `command_start_time: Option<Instant>`
- `running_process_pid: Option<u32>`
- `file_watcher: Option<FileWatcher>`
- `watch_enabled: bool`
- `git_branch: Option<String>`
- `last_command: Option<LastCommand>`
- `module_preferences: ProjectPreferences`

## Refactoring Pattern

**OLD:**
```rust
pub fn some_method(&mut self) {
    self.modules.len();
    self.command_output.push(line);
    self.project_root.join("pom.xml");
}
```

**NEW:**
```rust
pub fn some_method(&mut self) {
    let tab = self.get_active_tab_mut();
    tab.modules.len();
    tab.command_output.push(line);
    tab.project_root.join("pom.xml");
}
```

**For read-only access:**
```rust
pub fn some_method(&self) {
    let tab = self.get_active_tab();
    let count = tab.modules.len();
}
```

## Error Breakdown (by field)

| Field | Count |
|-------|-------|
| `command_output` | 56 |
| `config` | 17 |
| `output_offset` | 16 |
| `project_root` | 15 |
| `profiles` | 11 |
| `is_command_running` | 10 |
| `output_metrics` | 9 |
| `flags` | 9 |
| `running_process_pid` | 7 |
| `watch_enabled` | 6 |
| `file_watcher` | 6 |
| `command_receiver` | 6 |
| `output_view_height` | 5 |
| `modules_list_state` | 4 |
| `modules` | 4 |
| `module_preferences` | 4 |
| `output_area_width` | 3 |
| `module_outputs` | 3 |
| `flags_list_state` | 3 |
| `profiles_list_state` | 2 |
| `last_command` | 2 |
| `command_start_time` | 2 |
| `git_branch` | 1 |

**Total:** 203 errors

## Methods Already Fixed
- ✅ `set_profiles()`
- ✅ `next_item()`
- ✅ `previous_item()`
- ✅ `toggle_profile()`
- ✅ `toggle_flag()`
- ✅ `selected_module()`
- ✅ `enabled_flag_names()`
- ✅ `get_last_executed_command()`
- ✅ `active_profile_names()`
- ✅ `current_output_context()`
- ✅ `switch_to_profiles()`
- ✅ `switch_to_flags()`
- ⚠️ `sync_selected_module_output()` (partially)
- ⚠️ `sync_selected_profile_output()` (partially)

## Methods Needing Attention (High Priority)

These methods have the most field accesses and need fixing:

1. **`run_selected_module_command_with_options()`** (~50+ accesses)
   - Uses: modules, config, project_root, profiles, flags, command_receiver, is_command_running, running_process_pid, last_command

2. **`poll_command_updates()`** (~40+ accesses)
   - Uses: command_receiver, command_output, module_outputs, is_command_running, command_start_time

3. **`kill_running_process()`**
   - Uses: running_process_pid, is_command_running

4. **`check_file_watcher()`**
   - Uses: file_watcher, watch_enabled

5. **`yank_output()`**
   - Uses: command_output

6. **Scroll methods** (scroll_output_lines, scroll_output_pages, etc.)
   - Uses: output_offset, output_view_height, command_output

7. **`save_module_preferences()` / `load_module_preferences()`**
   - Uses: module_preferences, modules, profiles, flags

8. **`reload_config()`**
   - Uses: config, project_root, file_watcher

## Next Steps

### Option A: Manual refactoring (careful but slow)
Continue method-by-method refactoring, testing frequently

### Option B: Automated refactoring (fast but risky)
Create a script to do bulk replacements:

```bash
# Pseudo-code for bulk replacement
sed -i 's/self\.command_output/tab.command_output/g' src/ui/state/mod.rs
sed -i 's/self\.config/tab.config/g' src/ui/state/mod.rs
# ... etc for all fields
```

**Issue with Option B:** Need to:
1. Insert `let tab = self.get_active_tab_mut();` at start of each method
2. Handle read-only vs mutable access correctly
3. Not replace fields that stayed in TuiState (like `self.focus`, `self.search_state`)

### Option C: Hybrid approach (recommended)
1. Fix the 5-10 most critical methods manually (those with most accesses)
2. Use targeted replacements for simpler methods
3. Test compilation frequently
4. Fix borrow checker issues as they arise

## Completed Work So Far
- ✅ ProjectTab structure complete
- ✅ TuiState refactored with tabs field
- ✅ Tab management methods added
- ✅ Helper functions added (get_active_tab, create_tab, etc.)
- ✅ ~15% of methods adapted

## Remaining Work
- ⏳ ~85% of methods need adaptation
- ⏳ All field accesses need refactoring
- ⏳ Borrow checker issues to resolve
- ⏳ Testing and validation
