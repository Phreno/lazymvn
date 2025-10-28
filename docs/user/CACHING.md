# Performance Caching

LazyMVN implements intelligent caching to improve startup performance and reduce redundant operations.

## Overview

Three types of caching are used:

1. **Module Cache** - Project structure (modules, POM hash)
2. **Profiles Cache** - Maven profiles from `mvn help:all-profiles`
3. **Starters Cache** - Spring Boot main classes from dependency scanning

## Module Cache

**Location**: `~/.config/lazymvn/cache.json`

**Purpose**: Avoid reparsing POM files on every startup.

**Contents**:
- Project root path
- List of modules (`["."]` for single-module projects)
- POM content hash for change detection

**Behavior**:
- Created automatically on first project load
- Validated against current POM hash
- Regenerated if POM changes
- Shared across all tabs/sessions

**Manual Invalidation**:
```bash
rm ~/.config/lazymvn/cache.json
```

## Profiles Cache

**Location**: `~/.config/lazymvn/profiles/<project-hash>.json`

**Purpose**: Avoid slow `mvn help:all-profiles` execution on every startup.

**Contents**:
```json
{
  "profiles": ["dev", "prod", "test"]
}
```

**Behavior**:
- Automatically created after first Maven profile load
- Loaded instantly from cache on subsequent startups
- Per-project (using MD5 hash of project root path)
- Independent per tab

**Manual Refresh**:
1. Press `Ctrl+K` in the TUI
2. Or delete the cache file: `rm ~/.config/lazymvn/profiles/<hash>.json`

**When to Refresh**:
- After adding/removing profiles in `pom.xml`
- After switching branches with different profiles
- When profiles don't appear as expected

## Starters Cache

**Location**: `~/.config/lazymvn/starters/<project-hash>.json`

**Purpose**: Avoid rescanning dependencies for Spring Boot main classes.

**Contents**:
```json
{
  "starters": [
    {
      "id": 1,
      "label": "MyApplication",
      "fqcn": "com.example.MyApplication"
    },
    {
      "id": 2,
      "label": "AnotherApp",
      "fqcn": "com.example.AnotherApp"
    }
  ]
}
```

**Behavior**:
- Automatically scanned on first load if cache is empty
- Subsequent launches use cached starters instantly
- Per-project (using MD5 hash of project root path)
- Independent per tab

**Manual Refresh**:
1. Press `Ctrl+K` in the TUI
2. Or delete the cache file: `rm ~/.config/lazymvn/starters/<hash>.json`

**When to Refresh**:
- After adding new Spring Boot main classes
- After adding/removing dependencies
- When starters list is outdated

## Cache Refresh Keybinding

Press **`Ctrl+K`** at any time to refresh both profiles and starters caches.

**What happens**:
1. Profiles are reloaded by executing `mvn help:all-profiles`
2. Starters are rescanned by examining project dependencies
3. Both caches are saved to disk
4. Output pane shows confirmation message

**Example output**:
```
🔄 Caches refreshed successfully

✅ Maven profiles reloaded
✅ Spring Boot starters rescanned
```

## Performance Impact

### Before Caching
- **Startup time**: 10-30 seconds (depending on project size)
- **Profile loading**: 5-15 seconds per startup
- **Starter scanning**: 2-5 seconds per startup

### After Caching
- **Startup time**: 1-2 seconds
- **Profile loading**: <100ms (from cache)
- **Starter scanning**: <100ms (from cache)

### Typical Workflow
1. **First launch**: Slow (caches are created)
2. **Subsequent launches**: Fast (uses caches)
3. **After POM changes**: Press `Ctrl+K` to refresh
4. **Daily work**: No manual intervention needed

## Cache Location Structure

```
~/.config/lazymvn/
├── cache.json                          # Module cache (shared)
├── profiles/
│   ├── abc123def456.json               # Project A profiles
│   └── 789ghi012jkl.json               # Project B profiles
└── starters/
    ├── abc123def456.json               # Project A starters
    └── 789ghi012jkl.json               # Project B starters
```

**Hash generation**: MD5 of absolute project root path

**Example**:
- Project: `/home/user/my-project`
- Hash: `abc123def456`
- Profile cache: `~/.config/lazymvn/profiles/abc123def456.json`
- Starters cache: `~/.config/lazymvn/starters/abc123def456.json`

## Troubleshooting

### Profiles not showing up

**Symptoms**: Profiles view is empty or outdated

**Solution**:
1. Press `Ctrl+K` to refresh
2. Or manually delete: `rm ~/.config/lazymvn/profiles/*.json`

### Starters not showing up

**Symptoms**: Starter selector shows wrong classes

**Solution**:
1. Press `Ctrl+K` to refresh
2. Or manually delete: `rm ~/.config/lazymvn/starters/*.json`

### Cache corruption

**Symptoms**: Application crashes or behaves unexpectedly

**Solution**:
```bash
# Clear all caches
rm -rf ~/.config/lazymvn/profiles/
rm -rf ~/.config/lazymvn/starters/
rm ~/.config/lazymvn/cache.json
```

### Cache not updating after POM changes

**Symptoms**: Old modules/profiles still showing

**Solution**:
1. Module cache auto-updates (POM hash validation)
2. For profiles/starters: Press `Ctrl+K` to force refresh

## Best Practices

1. **Regular workflow**: Just use caches, they work automatically
2. **After branch switch**: Press `Ctrl+K` if profiles/starters differ
3. **After dependencies change**: Press `Ctrl+K` to refresh starters
4. **After POM edit**: Module cache auto-updates, press `Ctrl+K` for profiles
5. **Performance issues**: Check if cache files are corrupted, clear if needed

## Implementation Details

### Cache Invalidation Strategy
- **Module cache**: Automatic (POM hash comparison)
- **Profiles cache**: Manual (`Ctrl+K`) or file deletion
- **Starters cache**: Manual (`Ctrl+K`) or file deletion

### Cache Persistence
- JSON format for human readability
- Saved immediately after creation/update
- No expiration (valid until manually refreshed)

### Cache Isolation
- Per-project (using path hash)
- Per-tab (in-memory state)
- No cross-contamination between projects

## See Also

- [README.md](../../README.md#profile--starter-caching) - Quick overview
- [examples/lazymvn.toml.example](../../examples/lazymvn.toml.example) - Configuration examples
- [CHANGELOG.md](../../CHANGELOG.md) - Version history
