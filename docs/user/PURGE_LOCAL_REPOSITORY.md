# Purge Local Repository Command

## Overview

LazyMVN includes a **[p]urge** command that clears corrupted or problematic artifacts from your local Maven repository (`~/.m2/repository`).

## Usage

### In LazyMVN UI

Press **`p`** key to execute:
```bash
mvn dependency:purge-local-repository -DreResolve=false
```

The command is positioned in the footer between:
- **[d]eps** - Display dependency tree
- **[p]urge** - Purge local repository ← NEW
- **[y]ank** - Copy output to clipboard

### Location in UI

```
Commands: [b]uild [C]lean [c]ompile pac[k]age [t]est [i]nstall [s]tart [d]eps [p]urge [y]ank output [?]
                                                                               ^^^^^^
```

## When to Use

### Problem Scenarios

1. **Plugin Resolution Errors**
   ```
   [ERROR] Plugin org.springframework.boot:spring-boot-maven-plugin:1.4.13 
   could not be resolved: ...jar:1.4.13 was not found...
   This failure was cached in the local repository
   ```

2. **Corrupted Artifacts**
   ```
   [ERROR] Failed to read artifact descriptor for com.example:artifact:jar:1.0.0
   ```

3. **Stale Dependencies**
   - After repository URL changes
   - After Artifactory/Nexus migration
   - After network/proxy issues

4. **Version Conflicts**
   - Maven cached a failed download
   - Need to force re-download from remote

### Solution

The **[p]urge** command:
- ✅ Removes artifacts from local repository for the selected module
- ✅ Clears cached failures
- ✅ Does NOT attempt re-download (`-DreResolve=false`)
- ✅ Forces Maven to fetch fresh on next build

## Command Details

### Maven Goal

```bash
dependency:purge-local-repository
```

**What it does:**
- Deletes artifacts of the current project from `~/.m2/repository`
- Removes all associated metadata files
- Clears failure cache entries

### Flag: `-DreResolve=false`

**Purpose:** Prevents Maven from immediately re-downloading artifacts

**Why important:**
- Useful when offline or behind proxy
- Allows you to fix repository URLs before re-download
- Faster execution (no network calls)
- Can test builds without repository access

**Alternative:**
```bash
# With re-download (not used by LazyMVN)
mvn dependency:purge-local-repository -DreResolve=true
```

## Workflow Examples

### Example 1: Fix Plugin Resolution Error

**Problem:** Spring Boot plugin cached as "not found"

```
[ERROR] Plugin ...spring-boot-maven-plugin:1.4.13 was not found...
This failure was cached in the local repository
```

**Solution:**
1. Press **`p`** in LazyMVN to purge
2. Wait for purge to complete
3. Press **`b`** to build again
4. Maven will re-download the plugin

### Example 2: Clear After Repository Change

**Scenario:** Company migrated from Nexus to Artifactory

**Steps:**
1. Update `maven_settings.xml` with new repository URL
2. Select module in LazyMVN
3. Press **`p`** to purge old cached artifacts
4. Press **`b`** to rebuild with new repository

### Example 3: Multi-Module Purge

**Scenario:** Clean cache for specific module in reactor

**Steps:**
1. Navigate to problematic module
2. Press **`2`** to focus Modules pane
3. Select target module with **`↑/↓`**
4. Press **`p`** to purge only that module
5. Press **`b`** to rebuild

## Visual Feedback

When you press **`p`**, LazyMVN shows:

**Running:**
```
[p]urge  ← Highlighted in CYAN
```

**Success:**
```
[INFO] Local repository purged for: com.example:artifact:1.0.0
[p]urge  ← Highlighted in GREEN
```

**Failure:**
```
[ERROR] Failed to purge local repository
[p]urge  ← Highlighted in RED
```

## Comparison with Manual Cleanup

### Manual Approach (Old Way)

```bash
# Find artifact path
cd ~/.m2/repository/com/example/artifact/1.0.0
rm -rf *

# Or delete entire group
rm -rf ~/.m2/repository/com/example
```

**Problems:**
- ❌ Need to know exact path
- ❌ Risk of deleting too much
- ❌ No logging/verification
- ❌ Tedious for multiple modules

### LazyMVN [p]urge (New Way)

```
Press 'p'
```

**Benefits:**
- ✅ Scoped to selected module
- ✅ Maven handles dependencies
- ✅ Logged output visible
- ✅ Single keypress
- ✅ Visual feedback (color-coded)

## Technical Details

### Implementation

**File:** `src/ui/keybindings/command_keys.rs`
```rust
KeyCode::Char('p') if !has_modifiers => {
    log::info!("Execute: dependency:purge-local-repository");
    state.run_selected_module_command_with_key(
        &["dependency:purge-local-repository", "-DreResolve=false"], 
        Some('p')
    );
    true
}
```

### Maven Plugin

Uses: `maven-dependency-plugin`
- Goal: `purge-local-repository`
- Documentation: https://maven.apache.org/plugins/maven-dependency-plugin/purge-local-repository-mojo.html

### Parameters

| Parameter | Value | Purpose |
|-----------|-------|---------|
| `reResolve` | `false` | Don't re-download after purge |
| `actTransitively` | `true` (default) | Include transitive dependencies |
| `verbose` | `false` (default) | Minimal output |

## Troubleshooting

### Purge Doesn't Fix Issue

**Try:**
1. **Full clean**: `rm -rf ~/.m2/repository`
2. **Update Maven**: Check for newer version
3. **Check settings.xml**: Verify repository URLs
4. **Force update**: `mvn clean install -U` (press **Shift+U** in LazyMVN if implemented)

### Purge Too Slow

**Cause:** Large dependency tree

**Solution:**
```bash
# Purge only direct dependencies (manual)
mvn dependency:purge-local-repository -DactTransitively=false
```

### Network Issues After Purge

**Symptom:** Maven tries to download but fails

**Fix:**
1. Check internet/VPN connection
2. Verify proxy settings in `settings.xml`
3. Use `-o` (offline mode) if needed

## See Also

- [Maven Settings Management](MAVEN_SETTINGS.md)
- [Dependency Tree Command](DEPENDENCY_TREE.md) - **[d]eps**
- [Build Commands](BUILD_COMMANDS.md) - **[b]uild**, **[i]nstall**

## Commit

**Feature added:** November 3, 2025  
**Commit:** `feat: add [p]urge command for local repository cleanup`  
**Location:** Between [d]eps and [y]ank in command footer
