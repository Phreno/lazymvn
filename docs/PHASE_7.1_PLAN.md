# Phase 7.1: Split maven_tests.rs

**Date:** 2025-10-25  
**Target:** `src/maven_tests.rs` (957 lines → ~100 lines)  
**Effort:** 2-3 hours  
**Risk:** LOW

---

## Current Structure Analysis

### File Statistics
- **Total lines:** 957
- **Test functions:** 31 (`#[test]`)
- **Helper functions:** 2 (`test_lock()`, `write_script()`)
- **Imports:** 13 lines
- **Excess over target:** +557 lines

### Test Categories (By Domain)

#### 1. **Maven Command Execution** (3 tests, ~150 lines)
- `get_maven_command_returns_mvnw_if_present`
- `execute_maven_command_captures_output`
- `execute_maven_command_without_pl_for_root_module`

#### 2. **Profile Management** (10 tests, ~350 lines)
- `test_get_profiles`
- `test_get_profiles_deduplicates_and_sorts`
- `test_get_profile_xml`
- `test_get_profile_xml_from_settings`
- `test_get_profile_xml_with_maven_settings_xml`
- `test_extract_profiles_from_settings_xml`
- Additional profile-related tests

#### 3. **Spring Boot Detection** (8 tests, ~250 lines)
- `test_spring_boot_detection_with_plugin`
- `test_spring_boot_detection_with_war_packaging`
- `test_spring_boot_detection_with_pom_packaging`
- `test_spring_boot_detection_fallback_to_exec`
- `test_launch_strategy_auto_prefers_spring_boot`
- `test_launch_strategy_auto_falls_back_to_exec`
- `test_launch_strategy_force_run`
- `test_launch_strategy_force_exec`

#### 4. **Launch Command Building** (6 tests, ~200 lines)
- `test_build_launch_command_spring_boot_run`
- `test_build_launch_command_exec_java`
- `test_build_launch_command_exec_java_without_main_class`
- `test_build_launch_command_exec_java_war_packaging`
- `test_build_launch_command_exec_java_jar_packaging`
- `test_command_display_in_output`

#### 5. **Platform-Specific** (2 tests, ~50 lines)
- `test_quote_arg_for_platform_windows`
- `test_quote_arg_for_platform_unix`

#### 6. **Module Selection** (2 tests, ~100 lines)
- `test_exec_java_with_file_flag_adds_also_make`
- `test_exec_java_with_file_flag_preserves_existing_also_make`

---

## Proposed Structure

### Directory Layout
```
tests/
├── common/
│   └── mod.rs              ~50 lines (shared fixtures: test_lock, write_script)
│
├── maven/
│   ├── mod.rs              ~20 lines (module declaration)
│   ├── command_tests.rs    ~150 lines (Maven command execution)
│   ├── profile_tests.rs    ~350 lines (Profile loading and parsing)
│   ├── detection_tests.rs  ~250 lines (Spring Boot detection)
│   ├── launcher_tests.rs   ~200 lines (Launch command building)
│   ├── platform_tests.rs   ~50 lines (Platform-specific quote/args)
│   └── module_tests.rs     ~100 lines (Module selection, -pl, -f flags)
│
└── integration/
    └── end_to_end_tests.rs  ~100 lines (Future: Full integration tests)
```

### Total Impact
- **Before:** 1 file × 957 lines = 957 lines
- **After:** 8 files × ~140 lines avg = ~1,120 lines (includes new structure)
- **Main file reduction:** 957 → ~50 lines (common fixtures) = **-94.8%**
- **Modules created:** 8 new test modules

---

## Migration Strategy

### Step 1: Create Test Structure (30 min)
1. Create `tests/` directory at project root
2. Create `tests/common/mod.rs` with shared helpers:
   - `test_lock()`
   - `write_script()`
   - Common imports
3. Update `Cargo.toml` if needed (integration tests auto-discovered)

### Step 2: Extract Command Tests (20 min)
**Target:** `tests/maven/command_tests.rs`

Move tests:
- `get_maven_command_returns_mvnw_if_present`
- `execute_maven_command_captures_output`
- `execute_maven_command_without_pl_for_root_module`

### Step 3: Extract Profile Tests (45 min)
**Target:** `tests/maven/profile_tests.rs`

Move tests:
- All `test_get_profile*` tests
- `test_extract_profiles_from_settings_xml`
- Profile-related XML parsing tests

### Step 4: Extract Detection Tests (40 min)
**Target:** `tests/maven/detection_tests.rs`

Move tests:
- All `test_spring_boot_detection_*` tests
- All `test_launch_strategy_*` tests
- `test_extract_tag_content`

### Step 5: Extract Launcher Tests (30 min)
**Target:** `tests/maven/launcher_tests.rs`

Move tests:
- All `test_build_launch_command_*` tests
- `test_command_display_in_output`

### Step 6: Extract Platform Tests (15 min)
**Target:** `tests/maven/platform_tests.rs`

Move tests:
- `test_quote_arg_for_platform_windows`
- `test_quote_arg_for_platform_unix`

### Step 7: Extract Module Tests (20 min)
**Target:** `tests/maven/module_tests.rs`

Move tests:
- `test_exec_java_with_file_flag_adds_also_make`
- `test_exec_java_with_file_flag_preserves_existing_also_make`

### Step 8: Delete Original & Verify (20 min)
1. Delete `src/maven_tests.rs`
2. Run `cargo test` to ensure all tests pass
3. Verify 219 tests still pass (no regressions)

**Total Time:** ~3.5 hours (includes buffer for issues)

---

## File Templates

### Template: `tests/common/mod.rs`
```rust
use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

pub fn test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

pub fn write_script(path: &Path, content: &str) {
    fs::write(path, content).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).unwrap();
    }
    #[cfg(windows)]
    {
        let bat_path = path.with_extension("bat");
        let bat_content = content
            .replace("#!/bin/sh\n", "")
            .replace("echo $@", "echo %*");
        fs::write(&bat_path, bat_content).unwrap();
    }
}
```

### Template: `tests/maven/command_tests.rs`
```rust
use lazymvn::maven::command::*;
use lazymvn::utils;
use std::fs;
use tempfile::tempdir;

mod common;

#[test]
fn get_maven_command_returns_mvnw_if_present() {
    // Test implementation...
}

#[test]
#[cfg(unix)]
fn execute_maven_command_captures_output() {
    let _guard = common::test_lock().lock().unwrap();
    // Test implementation...
}
```

### Template: `tests/maven/mod.rs`
```rust
mod command_tests;
mod detection_tests;
mod launcher_tests;
mod module_tests;
mod platform_tests;
mod profile_tests;
```

---

## Testing Strategy

### Pre-Migration Baseline
```bash
cargo test --test maven_tests
# Expected: All 31 tests pass
```

### During Migration (After Each Step)
```bash
# Test specific new module
cargo test --test command_tests  # Step 2
cargo test --test profile_tests  # Step 3
# etc.

# Ensure all existing tests still pass
cargo test
```

### Post-Migration Validation
```bash
# Full test suite
cargo test

# Verify test count (should remain 219)
cargo test -- --list | wc -l

# Run specific test domains
cargo test --test detection_tests
cargo test --test launcher_tests
```

---

## Benefits

### Immediate Benefits
✅ **Better organization:** Tests grouped by domain  
✅ **Faster test runs:** Can run specific test suites  
✅ **Parallel testing:** Cargo runs integration tests in parallel  
✅ **Easier debugging:** Smaller files, easier to navigate  
✅ **Better IDE support:** Jump to test file by domain

### Future Benefits
✅ **Scalability:** Easy to add new test domains  
✅ **Maintenance:** Changes to one domain don't affect others  
✅ **Reusability:** Common fixtures in `tests/common/`  
✅ **Documentation:** File names serve as domain documentation

---

## Risks & Mitigations

### Risk 1: Test Path Changes
**Impact:** Import paths change from `src/maven_tests.rs` to `tests/maven/*.rs`  
**Mitigation:** Use `mod common;` and relative imports, test incrementally

### Risk 2: Shared State
**Impact:** `test_lock()` must work across test files  
**Mitigation:** Keep `test_lock()` in `tests/common/mod.rs`, use `pub fn`

### Risk 3: Cargo.toml Configuration
**Impact:** Integration tests might need explicit configuration  
**Mitigation:** Cargo auto-discovers `tests/` directory, no config needed

---

## Success Criteria

✅ All 219 tests pass after migration  
✅ `src/maven_tests.rs` deleted  
✅ 8 new test modules created  
✅ Each test module < 400 lines  
✅ Common fixtures in `tests/common/`  
✅ No test duplication  
✅ Clean `cargo test` output

---

## Next Steps

**User Decision:**
1. ✅ **Approve Phase 7.1** - Start migration
2. ⏸️ **Modify plan** - Adjust structure or approach
3. ⏭️ **Skip to Phase 7.2** - Do `core/config.rs` instead

**If approved:**
1. Create test directory structure
2. Extract tests step by step
3. Validate after each step
4. Commit when all tests pass
5. Update REFACTORING_PRIORITIES.md with results

**Estimated completion:** 3-4 hours with validation
