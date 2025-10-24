# Yank Logs Feature

## Overview

The "Yank Logs" feature allows you to quickly copy debug logs from the current session to the clipboard for debugging and troubleshooting purposes.

## Key Features

### Session-based Logging
- Each time LazyMVN starts with `--debug`, it creates a unique session ID
- Session ID format: `YYYYMMDD-HHMMSS-mmm` (e.g., `20251024-110133-106`)
- All logs are tagged with the session ID: `[SESSION:20251024-110133-106]`

### Two Log Files
1. **Debug Log** (`~/.local/share/lazymvn/logs/debug.log`)
   - Contains all debug, info, warning, and error messages
   - All log levels are recorded here

2. **Error Log** (`~/.local/share/lazymvn/logs/error.log`)
   - Contains only ERROR level messages
   - Helps quickly identify issues

### Session Log Extraction
The yank logs feature intelligently extracts only logs from the current session:
- Scans both debug and error log files
- Filters logs by the current session ID
- Concatenates them into a single clipboard-ready format

## Usage

### Basic Usage
1. Launch LazyMVN with debug mode:
   ```bash
   lazymvn --debug
   # or
   cargo run -- --project demo/multi-module --debug
   ```

2. Use the application normally

3. When you need to share logs:
   - Press `Y` (Shift+y)
   - Check the output pane for confirmation
   - Paste the logs wherever needed (GitHub issue, email, etc.)

### Keybindings
| Key | Action |
|-----|--------|
| `y` | Yank (copy) command output to clipboard |
| `Y` | **Yank (copy) debug logs to clipboard (current session only)** |

## Log Format

When you press `Y`, the clipboard will contain:

```
=== LazyMVN Session Logs ===
Session ID: 20251024-110133-106
Timestamp: 2025-10-24 11:15:32

=== Debug Logs ===
[SESSION:20251024-110133-106] [2025-10-24 11:01:33.109] INFO - === LazyMVN Session Started ===
[SESSION:20251024-110133-106] [2025-10-24 11:01:33.110] INFO - Session ID: 20251024-110133-106
[SESSION:20251024-110133-106] [2025-10-24 11:01:33.111] INFO - Starting lazymvn
...

=== Error Logs ===
[SESSION:20251024-110133-106] [2025-10-24 11:02:15.234] ERROR - Failed to load profiles: Connection timeout
...
```

## Platform Support

The yank logs feature uses platform-specific clipboard tools for maximum reliability:

### Linux
1. **wl-copy** (Wayland) - tried first
2. **xclip** (X11) - fallback
3. **xsel** (X11) - second fallback
4. **arboard** (Rust library) - final fallback

### macOS
1. **pbcopy** - native clipboard tool
2. **arboard** - fallback

### Windows
1. **PowerShell Set-Clipboard** - native clipboard
2. **arboard** - fallback

## Benefits

### For Developers
- Quickly share logs when reporting bugs
- No need to manually find and open log files
- Only current session logs (no clutter from previous sessions)
- Works seamlessly with GitHub issues, email, Discord, etc.

### For Debugging
- All relevant context in one place
- Timestamped entries for precise debugging
- Separate error logs for quick error identification
- Session ID helps correlate logs with specific runs

## Examples

### Example 1: Reporting a Bug
```
1. Reproduce the bug with --debug flag
2. Press 'Y' immediately after the bug occurs
3. Open a GitHub issue
4. Paste the logs (Ctrl+V)
5. Submit with all relevant context
```

### Example 2: Debugging Build Issues
```
1. Run a Maven build that fails
2. Press 'Y' to copy the session logs
3. Press 'y' to copy the Maven output
4. Now you have both:
   - Application logs (what LazyMVN was doing)
   - Maven output (what Maven reported)
```

### Example 3: Comparing Sessions
```
Session A: 20251024-110133-106
Session B: 20251024-113045-892

Each session has its own isolated logs.
You can run multiple sessions and extract logs from each independently.
```

## Technical Details

### Session ID Generation
```rust
let session_id = format!("{}", chrono::Local::now().format("%Y%m%d-%H%M%S-%3f"));
// Example: 20251024-110133-106
```

### Log Extraction Algorithm
1. Read the current session ID from logger state
2. Open debug.log and error.log files
3. For each file:
   - Scan line by line
   - Match lines containing `[SESSION:{current_id}]`
   - Stop when encountering a different session ID
4. Concatenate all matched lines
5. Add headers and formatting
6. Copy to clipboard

### Performance
- Efficient line-by-line reading (doesn't load entire file into memory)
- Stops reading when session changes (doesn't process the entire log file)
- Typical extraction time: < 100ms for logs up to 10,000 lines

## Log File Locations

### Linux
```
~/.local/share/lazymvn/logs/
├── debug.log
└── error.log
```

### macOS
```
~/Library/Application Support/com.lazymvn.lazymvn/logs/
├── debug.log
└── error.log
```

### Windows
```
%LOCALAPPDATA%\lazymvn\lazymvn\data\logs\
├── debug.log
└── error.log
```

## Troubleshooting

### "No logs available" message
- **Cause**: Debug mode is not enabled
- **Solution**: Launch with `--debug` flag

### "Failed to retrieve logs" message
- **Cause**: Log files don't exist or can't be read
- **Solution**: Check file permissions on the log directory

### Clipboard doesn't work
- **Linux**: Install clipboard tools:
  ```bash
  # For Wayland
  sudo apt install wl-clipboard
  
  # For X11
  sudo apt install xclip
  # or
  sudo apt install xsel
  ```
- **Other platforms**: arboard should work by default

### Logs from previous session included
- This shouldn't happen - each session has a unique ID
- If it does, please report as a bug with the session IDs involved

## Future Enhancements

Potential improvements for future versions:
- [ ] Add log filtering options (by level, by keyword)
- [ ] Add time range selection for logs
- [ ] Export logs to file instead of clipboard
- [ ] Compress logs for large sessions
- [ ] Add log viewer UI within LazyMVN
- [ ] Automatic anonymization of sensitive data

## Related

- [Debugging Guide](CONTRIBUTING.md#debugging)
- [Logging Configuration](LOGGING_CONFIG.md)
- [Clipboard Support](README.md#clipboard-features)
