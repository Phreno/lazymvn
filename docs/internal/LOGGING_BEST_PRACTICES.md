# Logging Best Practices for LazyMVN

This document outlines the logging strategy for LazyMVN to maintain useful debug reports while keeping them manageable in size.

## Log Levels

LazyMVN uses standard log levels with specific purposes:

### TRACE
**Purpose**: Very detailed, high-frequency events  
**When to use**: 
- Mouse movement/clicks
- Frequent UI updates
- Key presses (individual keys)
- Render cycles
- Polling operations

**Example**:
```rust
log::trace!("Mouse clicked at ({}, {})", x, y);
log::trace!("Rendering frame {}", frame_count);
log::trace!("Key pressed: {:?}", key);
```

**⚠️ Not included in debug reports** - Too verbose for troubleshooting

---

### DEBUG
**Purpose**: Development debugging, detailed flow information  
**When to use**:
- Cache hits/misses
- File parsing operations
- Configuration loading
- Process state changes
- Algorithm decisions

**Example**:
```rust
log::debug!("Loaded {} profiles from cache", count);
log::debug!("Parsing POM file: {}", path.display());
log::debug!("Selected module: {}", module_name);
```

**✅ Included in debug reports** - Essential for troubleshooting

---

### INFO
**Purpose**: General application flow and important events  
**When to use**:
- Command execution start/end
- Process creation/termination
- Configuration reloads
- File watching events
- Tab creation/closure

**Example**:
```rust
log::info!("Executing Maven command: mvn {}", args.join(" "));
log::info!("Created new tab for project: {}", path.display());
log::info!("Session started with ID: {}", session_id);
```

**✅ Included in debug reports** - Primary source of flow information

---

### WARN
**Purpose**: Non-fatal issues, fallback behavior  
**When to use**:
- Configuration issues (missing/invalid)
- Fallback to defaults
- Deprecated features
- Recoverable errors

**Example**:
```rust
log::warn!("Config file not found, using defaults");
log::warn!("No custom goals configured");
log::warn!("Maven process exited with non-zero status: {}", code);
```

**✅ Included in debug reports** - Important for diagnosing issues

---

### ERROR
**Purpose**: Errors that prevent operations from completing  
**When to use**:
- File I/O failures
- Maven execution failures
- Critical configuration errors
- Resource allocation failures

**Example**:
```rust
log::error!("Failed to read POM file: {}", e);
log::error!("Cannot create log directory: {}", e);
log::error!("Process spawn failed: {}", e);
```

**✅ Included in debug reports** - Critical for bug reports  
**Also logged to**: `error.log` (separate file)

---

## Debug Report Strategy

When users press **Shift+Y** to generate a debug report, the system includes:

### Configuration
- **Filtered**: Comments and empty lines removed
- **Reason**: Reduces size by ~50-70%, keeps only active configuration
- **Example**: A 200-line config with comments becomes ~60 lines

### Logs
- **Session-scoped**: Only logs from the current LazyMVN session
- **Level filter**: DEBUG, INFO, WARN, ERROR (no TRACE)
- **Limit**: Last 300 lines maximum
- **Cross-file**: Includes logs even if rotated to `.log.1`, `.log.2`, etc.

### Tab Output
- **Last 100 lines per tab**
- **Reason**: Most errors appear at the end of Maven output
- **Indicator**: Shows "(Showing last 100 lines of 500)" when truncated

### System Info
- Full version, commit SHA, system details (small, always included)

## Migration Guide: DEBUG → TRACE

If you find a log statement appears too frequently in debug reports:

### 1. Identify High-Frequency Logs

Look for patterns that happen multiple times per second:
```rust
// BAD - Creates hundreds of log lines
log::debug!("Mouse moved to ({}, {})", x, y);
log::debug!("Scrolling output pane");
```

### 2. Convert to TRACE

```rust
// GOOD - Still available for deep debugging, but excluded from reports
log::trace!("Mouse moved to ({}, {})", x, y);
log::trace!("Scrolling output pane");
```

### 3. Keep DEBUG for Important Events

```rust
// GOOD - Important state changes stay in DEBUG
log::debug!("Switched to tab {}", tab_index);
log::debug!("Module selection changed to: {}", module);
```

## Current Logging Locations

### Files using TRACE
- `src/tui/mouse.rs` - Mouse events (converted from DEBUG)

### Files using DEBUG
- `src/core/config/types.rs` - Config loading, cache operations
- `src/ui/state/profiles.rs` - Profile loading, caching
- `src/features/starters.rs` - Starter scanning, caching
- `src/maven/detection.rs` - Maven strategy detection
- `src/maven/command.rs` - Command building, Log4j config

### Files using INFO
- `src/ui/state/mod.rs` - Tab management, command execution, cleanup
- `src/ui/state/project_tab.rs` - Process management, file watching
- `src/maven/process.rs` - Process termination

### Files using WARN
- `src/ui/state/profiles.rs` - Missing profiles
- `src/ui/state/mod.rs` - Missing config, empty custom goals

### Files using ERROR
- (Currently minimal - good! Errors are exceptional)

## Testing Your Changes

### View Current Session Logs
```bash
# Run with debug level
lazymvn --log-level debug

# Check the logs
cat ~/.local/share/lazymvn/logs/debug.log | grep "SESSION:$(date +%Y%m%d)"
```

### Generate Debug Report
1. Run LazyMVN
2. Perform the operation you're debugging
3. Press `Shift+Y`
4. Check the clipboard content size

### Verify Log Level Distribution
```bash
# Count log levels in current session
grep "SESSION:$(date +%Y%m%d)" ~/.local/share/lazymvn/logs/debug.log | \
  grep -oE "\[(TRACE|DEBUG|INFO|WARN|ERROR)\]" | \
  sort | uniq -c
```

Expected distribution:
- TRACE: Thousands (mouse, keys) - **excluded from reports**
- DEBUG: Hundreds (operations, parsing)
- INFO: Tens (commands, major events)
- WARN: Few (issues, fallbacks)
- ERROR: Rare (only real failures)

## Size Targets

Ideal debug report sizes:

| Component | Target Size | Notes |
|-----------|-------------|-------|
| System Info | ~10 lines | Fixed size |
| Version Info | ~8 lines | Fixed size |
| Config | 20-100 lines | After removing comments |
| Logs | 100-300 lines | Session-scoped, filtered |
| Tab Output (×N tabs) | 100 lines each | Last 100 per tab |
| **Total** | **~500-1000 lines** | Manageable for bug reports |

Before optimization: Reports could exceed 5000+ lines  
After optimization: Reports typically 500-1000 lines

## When to Change Log Levels

### Move to TRACE if:
- ✅ Event happens more than 10 times per second
- ✅ Only useful for performance profiling
- ✅ Creates noise in debug reports
- ✅ Not needed for typical bug diagnosis

### Keep in DEBUG if:
- ✅ Event happens occasionally (< 10 per minute)
- ✅ Helps understand application flow
- ✅ Useful for diagnosing bugs
- ✅ Shows state changes

### Use INFO for:
- ✅ User-initiated actions
- ✅ Major application milestones
- ✅ Process lifecycle events
- ✅ Configuration changes

### Use WARN for:
- ✅ Recoverable errors
- ✅ Fallback behavior
- ✅ Configuration issues
- ✅ Deprecated usage

### Use ERROR for:
- ✅ Unrecoverable failures
- ✅ Critical errors
- ✅ Data corruption
- ✅ Resource allocation failures

## Future Improvements

Potential enhancements to consider:

1. **Structured Logging**: Add context to logs (tab ID, module, command)
2. **Log Sampling**: For very high-frequency events, log every Nth occurrence
3. **User-Configurable Limits**: Let users adjust line limits in config
4. **Incremental Exports**: Export only logs since last command
5. **Log Categories**: Tag logs by subsystem (UI, Maven, Config, etc.)

## Related Documentation

- [Log Rotation](./LOG_ROTATION.md) - How logs are rotated and cleaned up
- [Debug Yank Guide](../../scripts/test_yank_logs_guide.sh) - Testing Shift+Y feature
- [Logger Implementation](../../src/utils/logger.rs) - Source code
