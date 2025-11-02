# Live Configuration Reload

## Overview

LazyMVN supports live configuration reload, allowing you to modify settings without restarting the application. This document explains how it works and what changes are supported.

## How to Use

1. Press `Ctrl+E` while running lazymvn
2. Your system editor opens with `lazymvn.toml`
3. Make your configuration changes
4. Save and close the editor
5. **Changes are automatically applied!**

The application detects what changed and logs the modifications. If watch settings were modified, the file watcher is automatically recreated with new patterns and debounce settings.

## Supported Configuration Changes

All configuration changes in `lazymvn.toml` are automatically applied:

### Maven Settings
```toml
maven_settings = "./settings.xml"
```
- **Effect**: Next Maven command will use the new settings file
- **Detection**: Logs if path changed or added/removed

### Launch Mode
```toml
launch_mode = "auto"  # or "force-run", "force-exec"
```
- **Effect**: Next Spring Boot launch uses new strategy
- **Detection**: Logs mode change (e.g., "Launch mode changed: auto → force-run")
- **Options**:
  - `auto`: Auto-detect (prefers spring-boot:run, falls back to exec:java)
  - `force-run`: Always use spring-boot:run
  - `force-exec`: Always use exec:java

### Watch Configuration
```toml
[watch]
enabled = true
commands = ["test", "start"]
patterns = ["src/**/*.java", "src/**/*.xml"]
debounce_ms = 500
```
- **Effect**: File watching starts/stops, patterns and debounce updated
- **Detection**: Logs if enabled/disabled and recreates watcher
- **Notes**: Watcher is recreated immediately with new settings

### Notifications
```toml
notifications_enabled = true
```
- **Effect**: Desktop notifications on command completion enabled/disabled
- **Detection**: Logs if changed

### Output Buffer
```toml
[output]
max_lines = 10000
max_updates_per_poll = 100
```
- **Effect**: Output pane buffer size and update rate adjusted
- **Detection**: Logs if changed
- **Use cases**: Reduce for low-memory systems, increase for verbose builds

### Logging Configuration
```toml
[logging]
packages = [
    { name = "com.example.api", level = "DEBUG" },
    { name = "org.springframework", level = "WARN" }
]
```
- **Effect**: Next Maven command injects new JVM log level arguments
- **Detection**: Logs if logging configuration changed
- **Format**: `-Dlog4j.logger.{package}={level}` and `-Dlogging.level.{package}={level}`

## Implementation Details

### Editor Selection

The editor is chosen in this order:
1. `$EDITOR` environment variable
2. `$VISUAL` environment variable
3. Platform defaults:
   - **Linux/macOS**: `vi`
   - **Windows**: `notepad.exe`

### Change Detection

Configuration reload uses `PartialEq` trait comparisons to detect changes:

```rust
pub fn reload_config(&mut self) {
    let new_config = load_config(&self.project_root);
    
    // Detect specific changes
    let launch_mode_changed = self.config.launch_mode != new_config.launch_mode;
    let watch_changed = self.config.watch.enabled != new_config.watch.enabled;
    
    // Apply new configuration
    self.config = new_config;
    
    // Recreate watcher if needed
    if watch_changed {
        self.file_watcher = create_file_watcher(&self.config);
    }
}
```

### File Watcher Recreation

When watch configuration changes:
1. Old watcher is dropped (automatically stops)
2. New watcher created with updated:
   - File patterns (glob syntax)
   - Debounce delay
   - Watched commands list
3. New watcher starts immediately

### No Restart Overhead

Unlike file watching approaches that continuously monitor configuration:
- **No CPU overhead**: Config reloaded only when user explicitly edits (Ctrl+E)
- **Immediate feedback**: Changes detected and applied synchronously
- **User control**: Reload happens when editor closes, not on file save
- **Predictable**: No race conditions or partial configuration states

## Examples

### Enable File Watching

**Before:**
```toml
# [watch] section absent or commented
```

**After:**
```toml
[watch]
enabled = true
commands = ["test", "start"]
patterns = ["src/**/*.java"]
debounce_ms = 500
```

**Result:**
```
Reloading configuration from disk
File watching enabled
✅ Configuration file saved and reloaded.
```

### Change Launch Mode

**Before:**
```toml
launch_mode = "auto"
```

**After:**
```toml
launch_mode = "force-exec"
```

**Result:**
```
Reloading configuration from disk
Launch mode changed: auto → force-exec
✅ Configuration file saved and reloaded.
```

### Add Logging Configuration

**Before:**
```toml
# No logging section
```

**After:**
```toml
[logging]
packages = [
    { name = "com.myapp.service", level = "DEBUG" }
]
```

**Result:**
```
Reloading configuration from disk
Logging configuration changed
✅ Configuration file saved and reloaded.
```

Next Spring Boot launch will include:
```bash
-Dlog4j.logger.com.myapp.service=DEBUG -Dlogging.level.com.myapp.service=DEBUG
```

## Troubleshooting

### Editor Doesn't Open

**Check environment variables:**
```bash
echo $EDITOR
echo $VISUAL
```

**Set editor explicitly:**
```bash
export EDITOR=nano  # or vim, emacs, code --wait, etc.
lazymvn
```

### Changes Not Applied

1. **Check logs** (if debug enabled):
   ```bash
   lazymvn --debug
   # In another terminal:
   tail -f ~/.local/share/lazymvn/logs/debug.log
   ```

2. **Verify file path**: Config must be named `lazymvn.toml` in project root

3. **Check syntax**: Invalid TOML syntax prevents loading
   ```bash
   # Test parsing manually:
   cat lazymvn.toml
   ```

4. **Editor must close**: Some editors (like `code`) return immediately. Use:
   ```bash
   export EDITOR="code --wait"  # VS Code waits for window close
   ```

### Watch Not Working After Reload

Check patterns are valid glob syntax:
```toml
[watch]
patterns = [
    "src/**/*.java",      # ✅ Correct
    "src/*.java",         # ⚠️ Only watches src/ directly
    "**/*.properties"     # ✅ All properties files
]
```

## Comparison with File Watching

| Approach | CPU Overhead | Feedback Timing | User Control |
|----------|--------------|-----------------|--------------|
| **Live Reload (Ctrl+E)** | None | Immediate on editor close | Explicit |
| File Watching | Continuous polling | Debounced after save | Automatic |

Live reload is preferred because:
- Zero CPU overhead when not editing
- No race conditions (changes atomic)
- Clear user intent (explicit edit action)
- Works with any editor
- No notification spam from multiple saves

## Related Documentation

- [Configuration File Format](README.md#configuration-file)
- [File Watching Configuration](README.md#file-watching)
- [Logging Configuration](LOGGING_CONFIG.md)
- [Watch Feature Documentation](WATCH_FEATURE.md)

## Future Enhancements

Potential improvements (not yet implemented):

- **Validation before apply**: Check config syntax before reloading
- **Rollback on error**: Restore previous config if new one fails to parse
- **Configuration history**: Track recent config changes
- **Visual feedback**: Show what changed in UI notification
