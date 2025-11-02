# Process Cleanup on Exit

## Problem Statement

When running Maven commands (especially long-running processes like `spring-boot:run` or `exec:java`), lazymvn spawns Maven processes that may continue running even after the TUI application exits. This creates:

1. **Orphaned processes**: Java/Maven processes running in the background consuming resources
2. **Port conflicts**: Spring Boot apps holding ports (e.g., 8080) preventing new instances
3. **Resource leaks**: Memory and CPU usage from zombie processes
4. **Confusing behavior**: User thinks they stopped the app but it's still running

## Solution Overview

LazyMVN now implements **graceful process cleanup** that:
- Tracks running Maven process PIDs
- Kills processes when application exits (both 'q' key and Ctrl+C)
- Handles signal interrupts (SIGINT, SIGTERM) 
- Kills entire process groups to catch child processes
- Uses platform-specific kill strategies (Unix vs Windows)

## Implementation Details

### 1. Process Tracking

The `TuiState` struct tracks the running Maven process:

```rust
pub struct TuiState {
    // ... other fields
    running_process_pid: Option<u32>,
    is_command_running: bool,
}
```

When a Maven command starts, the PID is captured from the `CommandUpdate::Started(pid)` message.

### 2. Cleanup Method

The `cleanup()` method in `TuiState` handles resource cleanup:

```rust
pub fn cleanup(&mut self) {
    log::info!("Cleaning up application resources");
    
    // Kill any running Maven process
    if let Some(pid) = self.running_process_pid {
        log::info!("Killing running Maven process with PID: {}", pid);
        match crate::maven::kill_process(pid) {
            Ok(()) => {
                log::info!("Successfully killed Maven process {}", pid);
            }
            Err(e) => {
                log::error!("Failed to kill Maven process {}: {}", pid, e);
            }
        }
        self.running_process_pid = None;
        self.is_command_running = false;
    }
    
    // Save module preferences
    if let Err(e) = self.module_preferences.save(&self.project_root) {
        log::error!("Failed to save module preferences: {}", e);
    }
    
    log::info!("Cleanup completed");
}
```

### 3. Exit Points

#### Normal Exit ('q' key)

```rust
// In main.rs event loop
if key.code == event::KeyCode::Char('q') && !state.show_projects_popup {
    log::info!("User requested quit");
    break;
}

// After loop
state.cleanup();
```

#### Signal Interrupts (Ctrl+C, SIGTERM)

```rust
// Setup signal handler using ctrlc crate
let running = Arc::new(AtomicBool::new(true));
let r = running.clone();

ctrlc::set_handler(move || {
    log::info!("Received interrupt signal (Ctrl+C), initiating shutdown");
    r.store(false, Ordering::SeqCst);
})
.expect("Error setting Ctrl-C handler");

// In main loop
if !running.load(Ordering::SeqCst) {
    log::info!("Interrupt signal detected, breaking main loop");
    break;
}

// After loop
state.cleanup();
```

### 4. Process Killing Strategy

The `kill_process()` function in `maven/process.rs` uses platform-specific approaches:

#### Unix/Linux/macOS

```rust
// Try to kill the entire process group (negative PID)
Command::new("kill")
    .arg("-TERM")           // Graceful termination
    .arg(format!("-{}", pid)) // Negative PID = process group
    .output();

// Wait briefly for graceful shutdown
std::thread::sleep(std::time::Duration::from_millis(100));

// Force kill if still running
Command::new("kill")
    .arg("-KILL")           // Force kill
    .arg(format!("-{}", pid))
    .output();
```

**Why process groups?**
- Maven spawns child processes (e.g., Java for Spring Boot)
- Killing just Maven leaves Java running
- Negative PID kills all processes in the group

#### Windows

```rust
Command::new("taskkill")
    .arg("/PID").arg(pid.to_string())
    .arg("/T")  // Kill process tree (all children)
    .arg("/F")  // Force kill
    .output();
```

The `/T` flag ensures all child processes are terminated.

## Testing

### Manual Test Procedure

1. **Start lazymvn with a Spring Boot module:**
   ```bash
   cargo run -- --project demo/multi-module --debug
   ```

2. **Start a long-running process:**
   - Select a module (e.g., `app`)
   - Press `s` to start Spring Boot
   - Wait for application to start

3. **Verify process is running:**
   ```bash
   # In another terminal
   ps aux | grep -E '[m]vn|[j]ava.*maven'
   ```

4. **Test Scenario 1: Normal quit**
   - Press `q` in lazymvn
   - Check processes again - should be gone

5. **Test Scenario 2: Ctrl+C**
   - Restart lazymvn and start the module
   - Press Ctrl+C
   - Check processes again - should be gone

6. **Check debug logs:**
   ```bash
   tail -f ~/.local/share/lazymvn/logs/debug.log
   ```
   
   Look for:
   ```
   Killing running Maven process with PID: 12345
   Successfully killed Maven process 12345
   ```

### Automated Test Script

Use `scripts/test-process-cleanup.sh` for guided testing.

## Debugging

### Process Still Running After Exit

**Check logs:**
```bash
grep -i "kill" ~/.local/share/lazymvn/logs/debug.log
```

**Possible causes:**
1. **Process PID not captured**: Check for `CommandUpdate::Started` message
2. **Kill command failed**: Check error logs
3. **Process group issue**: On Unix, ensure process started with new group ID

**Manual cleanup:**
```bash
# Find orphaned Maven/Java processes
ps aux | grep -E '[m]vn|[j]ava.*maven'

# Kill manually
kill -9 <PID>
```

### On Unix: Process Group Not Killed

Maven should start processes in a new process group. This is typically automatic, but can be verified:

```bash
# Check process group IDs
ps -o pid,pgid,cmd | grep mvn
```

If PGID != PID, the process isn't a group leader and negative PID won't work. This is rare but possible.

### On Windows: Process Tree Not Terminated

Windows `taskkill /T` should handle process trees, but some Java processes may resist termination if:
- Running with elevated privileges
- Using Windows services
- Spawned by a different user

## Future Improvements

Potential enhancements (not yet implemented):

1. **Process tree discovery**: Query child processes before killing
2. **Configurable kill timeout**: Allow user to set graceful shutdown delay
3. **Process monitoring**: Track multiple concurrent Maven commands
4. **Kill confirmation**: Ask user before killing long-running processes
5. **Graceful shutdown hooks**: Use Maven's shutdown hooks if available

## Related Code

- **Process tracking**: `src/ui/state/mod.rs` (`running_process_pid`, `cleanup()`)
- **Kill logic**: `src/maven/process.rs` (`kill_process()`)
- **Signal handling**: `src/main.rs` (ctrlc setup, main loop check)
- **Command execution**: `src/maven/command.rs` (async execution, PID capture)

## Dependencies

- **ctrlc**: Signal handling for Ctrl+C and SIGTERM
  ```toml
  ctrlc = "3.4"
  ```

## Platform Support

| Platform | Method | Process Groups | Notes |
|----------|--------|----------------|-------|
| Linux | `kill` command | Yes (negative PID) | SIGTERM + SIGKILL |
| macOS | `kill` command | Yes (negative PID) | SIGTERM + SIGKILL |
| Windows | `taskkill` | Yes (/T flag) | Force kill with /F |

## Security Considerations

- **Permission requirements**: Killing processes requires appropriate permissions
- **PID reuse**: PIDs can be reused; we only kill if `is_command_running` is true
- **Signal delivery**: Ctrl+C handler must complete quickly to avoid blocking
- **Error handling**: Failed kills are logged but don't prevent application exit

## Best Practices

1. **Always call cleanup()**: Even if error occurs, cleanup should run
2. **Log everything**: Process kills are critical operations - log success and failures
3. **Graceful then forceful**: Try SIGTERM before SIGKILL
4. **Check PID validity**: Ensure PID exists before attempting to kill
5. **Handle errors gracefully**: Don't panic if kill fails, just log and continue
