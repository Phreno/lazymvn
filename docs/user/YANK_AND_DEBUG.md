# Yanking Output and Debug Information

LazyMVN provides powerful "yanking" (copying) features to easily share command output and detailed debugging information.

## Yanking Command Output (`y`)

- **Key**: `y`
- **Action**: Copies the entire output of the currently active tab to the clipboard.
- **Use Case**: Quickly sharing the result of a Maven command (e.g., build failure, test results) with colleagues.

## Yanking a Full Debug Report (`Y`)

- **Key**: `Y` (Shift+Y)
- **Action**: Copies a comprehensive debug report for the entire application state to the clipboard. This is invaluable for reporting bugs or getting help.
- **Use Case**: Reporting a bug, asking for help, or archiving the complete state of the application for later analysis.

### Contents of the Debug Report

The debug report includes the following sections:

1.  **Header**: Timestamp of the report generation.
2.  **Version Information**:
    *   LazyMVN Version (from `Cargo.toml`).
    *   Build date, Git branch, and commit hash (if available).
3.  **System Information**:
    *   Operating System (e.g., Linux, Windows, macOS).
    *   Architecture (e.g., x86_64, aarch64).
4.  **Configuration**:
    *   The full content of the `lazymvn.toml` file.
    *   A message is shown if the file does not exist.
5.  **Output from All Tabs**: For each open tab:
    *   Tab number and name (project folder name).
    *   `[ACTIVE]` indicator for the currently active tab.
    *   Full project path.
    *   Currently selected module.
    *   Total lines of output.
    *   The last 100 lines of output (or all lines if fewer than 100).
6.  **LazyMVN Logs**:
    *   The last 500 lines of the debug log.
    *   The last 500 lines of the error log.
7.  **Footer**: End of report marker.

### Example Report Structure

```
================================================================================
LazyMVN Debug Report
================================================================================
Generated: 2025-10-24 15:30:45

=== Version Information ===
LazyMVN Version: 0.4.0-unstable

=== System Information ===
OS: linux
Architecture: x86_64

=== Configuration (lazymvn.toml) ===
[logging]
packages = [
  { name = "org.springframework", level = "WARN" },
  { name = "com.mycompany", level = "DEBUG" },
]

=== Output from All Tabs ===
--- Tab 1: multi-module [ACTIVE] ---
Project Root: /workspaces/lazymvn/demo/multi-module
Module: app
Output Lines: 45
(Showing last 45 lines of 45)
$ mvn clean install -pl app
[INFO] Scanning for projects...
...

--- Tab 2: single-module ---
Project Root: /workspaces/lazymvn/demo/single-module
Module: .
Output Lines: 0
(No output)

=== LazyMVN Logs ===
=== Debug Logs (last 500 lines) ===
[SESSION:20251024-153000-123] [2025-10-24 15:30:00.123] INFO - === LazyMVN Session Started ===
...

=== Error Logs (last 500 lines) ===
(No error logs)

================================================================================
End of Debug Report
================================================================================
```

### Use Cases for the Debug Report

*   **Reporting a Bug**: Reproduce the bug, press `Y`, and paste the report into a GitHub issue. Maintainers will have all the necessary information.
*   **Asking for Help**: Encounter a problem, press `Y`, and share the report on a forum or Discord. Others can see your exact configuration and state.
*   **Personal Debugging**: Save the application state with `Y` before making changes. If something goes wrong, you can compare it with the saved state.

### Technical Notes

*   **Size Management**: To prevent excessively large reports, logs are limited to the last 500 lines and tab outputs to the last 100 lines.
*   **Performance**: The operation is fast and does not block the UI, as file reading is buffered.
*   **Security**: Be aware that configuration files and logs can contain sensitive information like file paths. **Always review the report content before sharing it publicly.**

## Clipboard Platform Support

The yank feature uses platform-specific clipboard tools for maximum reliability:

*   **Linux**: `wl-copy` (Wayland), `xclip` (X11), or `xsel` (X11).
*   **macOS**: `pbcopy`.
*   **Windows**: PowerShell `Set-Clipboard` or `clip.exe`.
*   **Fallback**: `arboard`, a cross-platform Rust library, is used if native tools are not found.
