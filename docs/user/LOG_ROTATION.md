# Log Rotation System

LazyMVN implements an automatic log rotation system to prevent log files from growing indefinitely.

## Overview

The logging system maintains two main log files:
- `debug.log` - Contains all debug, info, warn, and error messages
- `error.log` - Contains only error messages (for quick troubleshooting)

Both files are located in:
- **Linux/macOS**: `~/.local/share/lazymvn/logs/`
- **Windows**: `%LOCALAPPDATA%\lazymvn\logs\`

## Rotation Strategy

### Size-Based Rotation

When LazyMVN starts, it checks the size of each log file:
- **Size limit**: 5 MB per file
- **Action**: If a file exceeds 5 MB, it's rotated before new logs are written

### Rotation Process

When a log file is rotated:

1. Existing backups are shifted:
   - `debug.log.5` → deleted (oldest backup removed)
   - `debug.log.4` → `debug.log.5`
   - `debug.log.3` → `debug.log.4`
   - `debug.log.2` → `debug.log.3`
   - `debug.log.1` → `debug.log.2`

2. Current log is archived:
   - `debug.log` → `debug.log.1`

3. A fresh empty log file is created

This maintains up to **5 rotated backups** per log file, for a total of:
- 6 debug log files (current + 5 backups) = ~30 MB max
- 6 error log files (current + 5 backups) = ~30 MB max
- **Total maximum disk usage**: ~60 MB

### Time-Based Cleanup

On each startup, LazyMVN also cleans up old rotated logs:
- **Retention period**: 30 days
- **Action**: Deletes any `.log.1`, `.log.2`, etc. files older than 30 days

This prevents accumulation of very old rotated logs that are unlikely to be useful.

## Session IDs

Each LazyMVN session generates a unique session ID in the format:
```
YYYYMMDD-HHMMSS-mmm
```

Example: `20251029-143052-123`

Session IDs are included in every log line:
```
[SESSION:20251029-143052-123] [2025-10-29 14:30:52.123] INFO - LazyMVN started
```

This makes it easy to:
- Filter logs for a specific session
- Identify when issues occurred
- Track multiple concurrent LazyMVN instances

## Log Levels

LazyMVN supports standard log levels:

| Level | Usage | Example |
|-------|-------|---------|
| `trace` | Very detailed debugging | Function entry/exit |
| `debug` | Development debugging (default for nightly) | Cache hits, file parsing |
| `info` | General information | Command execution, process start |
| `warn` | Warnings (non-fatal) | Config issues, fallback behavior |
| `error` | Errors | File I/O failures, Maven errors |
| `off` | No logging | Production mode (default for releases) |

### Setting Log Level

Via command line:
```bash
lazymvn --log-level debug
lazymvn --log-level info
lazymvn --log-level off
```

Default behavior:
- **Nightly builds**: `debug` level enabled by default
- **Release builds**: Logging disabled (`off`) by default

## Accessing Logs

### From Command Line

View recent debug logs:
```bash
tail -f ~/.local/share/lazymvn/logs/debug.log
```

View only errors:
```bash
cat ~/.local/share/lazymvn/logs/error.log
```

View rotated logs:
```bash
cat ~/.local/share/lazymvn/logs/debug.log.1
```

### From LazyMVN UI

Press **`Shift+Y`** to yank comprehensive debug information, which includes:
- Last 500 lines from debug and error logs
- Current configuration
- System information
- All tabs' output

This creates a complete diagnostic report that can be pasted into bug reports.

## Troubleshooting

### Logs Not Appearing

If you don't see logs being written:

1. Check if logging is enabled:
   ```bash
   lazymvn --log-level debug
   ```

2. Verify log directory exists:
   ```bash
   ls -la ~/.local/share/lazymvn/logs/
   ```

3. Check permissions on log directory

### Disk Space Issues

If log files are consuming too much space:

1. **Manual cleanup** - Delete old rotated logs:
   ```bash
   rm ~/.local/share/lazymvn/logs/*.log.[1-5]
   ```

2. **Disable logging** - Run with `--log-level off`:
   ```bash
   lazymvn --log-level off
   ```

3. **Reduce retention** - The 30-day cleanup happens automatically on startup

### Finding Session Logs

To find logs for a specific session:

1. Note your session ID (shown at startup in logs)
2. Search for that session:
   ```bash
   grep "SESSION:20251029-143052-123" ~/.local/share/lazymvn/logs/debug.log
   ```

## Implementation Details

The rotation system is implemented in `src/utils/logger.rs`:

- **`rotate_log_file()`** - Rotates a single log file if it exceeds size limit
- **`cleanup_old_logs()`** - Removes rotated logs older than 30 days
- **`init()`** - Initializes logging with rotation on startup

Key features:
- Non-blocking: Rotation failures don't prevent LazyMVN from starting
- Silent: Rotation happens transparently without user notification
- Efficient: Only checks/rotates on startup, not during execution

## Configuration

Currently, log rotation is not configurable and uses these hardcoded values:

- Maximum file size: 5 MB
- Number of rotated backups: 5
- Retention period: 30 days

If you need different values, please open a GitHub issue with your use case.

## Best Practices

1. **Development**: Use `--log-level debug` to capture detailed information
2. **Production**: Use `--log-level off` or default (off for releases)
3. **Bug Reports**: Always include `Shift+Y` output for full diagnostic context
4. **Performance**: Logging to disk has minimal performance impact
5. **Privacy**: Logs may contain file paths and Maven coordinates - review before sharing

## Related

- [Debug Yank Guide](../scripts/test_yank_logs_guide.sh) - Testing the Shift+Y feature
- [Logger Implementation](../../src/utils/logger.rs) - Source code
- [Versioning](../../VERSIONING.md) - Nightly vs release builds
