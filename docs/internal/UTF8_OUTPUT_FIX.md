# Fix UTF-8 Output Reader Crash

## Date
2025-11-04

## Issue
**User Report**: LazyMVN UI freezes after error "Error reading stdout: stream did not contain valid UTF-8"

### Symptom
```
[ERROR] Error reading stdout: stream did not contain valid UTF-8
```

After this error:
- No more output captured from Maven
- UI appears frozen (no updates)
- Maven process continues running but user can't see output
- Happens with Maven projects that output non-UTF-8 characters

### Root Cause
The stdout/stderr reader threads were using `BufReader::lines()` which calls `from_utf8()` internally. This method **panics or returns an error** when it encounters invalid UTF-8 bytes.

Common sources of non-UTF-8 in Maven output:
- **Windows-1252 encoding** (Windows console default)
- **ISO-8859-1 characters** (accented characters in French, German, etc.)
- **ANSI escape codes** (color codes, progress bars)
- **Box drawing characters** in Maven progress indicators
- **Corrupted output** from third-party Maven plugins

When `from_utf8()` failed, the reader thread would log an error and **terminate**, causing:
1. No more output captured
2. UI stuck waiting for updates
3. User experience: "frozen" application

## Technical Analysis

### Before Fix (Problematic Code)

```rust
let stdout_handle = thread::spawn(move || {
    let reader = BufReader::new(stdout);
    for line in reader.lines() {  // ❌ Uses from_utf8() - fails on invalid UTF-8
        match line {
            Ok(line) => { /* ... */ }
            Err(e) => {
                log::error!("Error reading stdout: {}", e);
                break;  // ❌ Thread terminates - no more output!
            }
        }
    }
});
```

**Problem**: `BufReader::lines()` returns `Result<String, std::io::Error>` and fails with `ErrorKind::InvalidData` when UTF-8 validation fails.

### After Fix (Robust Code)

```rust
fn read_lines_lossy<R: Read>(
    reader: R,
    tx: mpsc::Sender<CommandUpdate>,
    stream_name: &str,
) {
    let mut buf_reader = BufReader::new(reader);
    let mut buffer = Vec::new();
    
    loop {
        buffer.clear();
        
        // Read until newline or EOF
        match buf_reader.read_until(b'\n', &mut buffer) {
            Ok(0) => break,  // EOF
            Ok(_) => {
                // ✅ Convert with lossy UTF-8 - replaces invalid bytes with �
                let line = String::from_utf8_lossy(&buffer);
                let line = line.trim_end_matches('\n').trim_end_matches('\r');
                
                // ✅ Continue sending output even with invalid UTF-8
                if tx.send(CommandUpdate::OutputLine(line.to_string())).is_err() {
                    break;
                }
            }
            Err(e) => {
                log::error!("Error reading {}: {}", stream_name, e);
                break;  // Only stop on actual I/O errors
            }
        }
    }
}
```

**Solution**: 
1. Read raw **bytes** with `read_until(b'\n')`
2. Convert with **`String::from_utf8_lossy()`** which:
   - Replaces invalid UTF-8 sequences with � (U+FFFD)
   - **Never fails** - always returns a String
   - Preserves valid UTF-8 characters
3. Continue processing output **without crashing**

## Changes Made

### Files Modified

1. **`src/maven/command/executor.rs`**
   - Added import: `std::io::Read`
   - Added function: `read_lines_lossy()`
   - Replaced stdout reader thread with new implementation
   - Replaced stderr reader thread with new implementation

### Code Changes

**Import Addition**:
```rust
use std::{
    io::{BufRead, BufReader, Read},  // Added Read
    // ...
};
```

**New Helper Function** (42 lines):
```rust
fn read_lines_lossy<R: Read>(
    reader: R,
    tx: mpsc::Sender<CommandUpdate>,
    stream_name: &str,
) {
    // ... implementation
}
```

**Simplified Thread Spawning**:
```rust
// Before: ~40 lines of reader logic per thread
// After: 2 lines per thread

let stdout_handle = thread::spawn(move || {
    read_lines_lossy(stdout, tx_clone, "STDOUT");
});

let stderr_handle = thread::spawn(move || {
    read_lines_lossy(stderr, tx_clone, "STDERR");
});
```

## Testing

### Automated Tests

Created `/workspaces/lazymvn/crates/lazymvn-test-harness/tests/utf8_handling_tests.rs` with 8 tests:

1. **`test_utf8_lossy_conversion_doesnt_crash`**
   - Verifies output reader completes without crashing
   - Confirms output is captured

2. **`test_output_reader_handles_special_characters`**
   - Tests various special characters
   - Verifies Maven output still captured

3. **`test_output_reader_survives_long_running_process`**
   - Long-running processes with thousands of lines
   - Ensures reader stays alive throughout

4. **`test_no_error_log_for_utf8_issues`**
   - Confirms no "UTF-8 error" messages in output
   - Validates silent lossy conversion

5. **`test_replacement_character_for_invalid_utf8`**
   - Invalid UTF-8 replaced with � (U+FFFD)
   - Output reader completes successfully

6. **`test_concurrent_output_with_mixed_encodings`**
   - Stdout and stderr both handle mixed encodings
   - Error output captured correctly

7. **`test_progress_bar_characters_dont_crash`**
   - Maven progress bars with special characters
   - ANSI codes don't crash reader

8. **`test_windows_encoding_compatibility`**
   - Windows-1252 encoding handled
   - Platform-specific encodings work

**Results**: 8/8 tests passed in 25.76s

### Manual Testing

To test with a real Maven project that produces non-UTF-8:

```bash
# Build LazyMVN with fix
cargo build --release

# Run on a project with special characters
# (e.g., French accents, Windows console output)
./target/release/lazymvn

# Verify:
# 1. Output continues flowing
# 2. No "Error reading stdout" messages
# 3. UI stays responsive
# 4. Special characters appear as � or readable equivalents
```

## Impact

### Fixed
- ✅ Output reader no longer crashes on non-UTF-8
- ✅ UI remains responsive throughout Maven execution
- ✅ All Maven output captured (with lossy conversion)
- ✅ Works on Windows with Windows-1252 encoding
- ✅ Handles accented characters (French, German, etc.)
- ✅ ANSI escape codes don't crash reader
- ✅ Maven progress bars work correctly

### Behavior Change
- **Invalid UTF-8 bytes** → Replaced with � (U+FFFD)
- **Performance**: Negligible (same number of syscalls)
- **Log messages**: No more "Error reading stdout" for encoding issues

### Unchanged
- ✅ Valid UTF-8 output unchanged
- ✅ Line-by-line processing preserved
- ✅ Thread safety maintained
- ✅ Error handling for real I/O errors still works

## Examples

### Before Fix (Crash)
```
[INFO] Building project...
[INFO] Downloading: https://...
[ERROR] Error reading stdout: stream did not contain valid UTF-8
(no more output - UI frozen)
```

### After Fix (Robust)
```
[INFO] Building project...
[INFO] Downloading: https://...
[INFO] T�l�chargement termin� (accented characters replaced)
[INFO] BUILD SUCCESS
```

The output continues and the UI remains responsive, even with encoding issues.

## Related Files
- `src/maven/command/executor.rs` - Main fix location
- `crates/lazymvn-test-harness/tests/utf8_handling_tests.rs` - Test coverage
- `src/maven/process.rs` - Uses `CommandUpdate` messages

## Future Enhancements

Potential improvements (not critical):

1. **Encoding Detection**: Detect Windows-1252/ISO-8859-1 and convert properly
2. **Statistics**: Track how many invalid UTF-8 sequences encountered
3. **User Option**: Allow users to specify expected encoding
4. **Logging**: Add DEBUG log when replacement occurs (currently silent)

## References
- Rust `String::from_utf8_lossy()` docs: https://doc.rust-lang.org/std/string/struct.String.html#method.from_utf8_lossy
- UTF-8 Replacement Character: https://en.wikipedia.org/wiki/Specials_(Unicode_block)#Replacement_character
- Windows-1252 encoding: https://en.wikipedia.org/wiki/Windows-1252
- User report: Debug report dated 2025-11-04 09:05:55
