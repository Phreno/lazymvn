# Exception Colorization Example

This example demonstrates the automatic colorization of Java exception names in log output.

## Feature Description

The text processing utility automatically detects and highlights Java exception names (any class ending with "Exception") in log lines.

## Pattern Detection

**Regex Pattern:** `\b[A-Z][a-zA-Z0-9]*Exception\b`

This pattern matches:
- Class names starting with an uppercase letter
- Followed by any combination of letters and numbers
- Ending with "Exception"
- With word boundaries to avoid false matches

## Colorization Strategy

- **Color:** Light Red (bold)
- **Scope:** Applied to the exception name only, not the entire line
- **Integration:** Works alongside package name colorization (cyan) and log level highlighting

## Examples

### Single Exception
```
[ERROR] com.example.Service - NullPointerException occurred at line 42
```
**Highlighted:**
- `[ERROR]` → Red (log level)
- `com.example.Service` → Cyan (package)
- `NullPointerException` → Light Red Bold (exception)

### Multiple Exceptions
```
[ERROR] Failed with IllegalArgumentException and RuntimeException
```
**Highlighted:**
- `[ERROR]` → Red (log level)
- `IllegalArgumentException` → Light Red Bold (exception)
- `RuntimeException` → Light Red Bold (exception)

### Exception with Stack Trace
```
[ERROR] java.io.FileReader - IOException: file not found /tmp/data.txt
```
**Highlighted:**
- `[ERROR]` → Red (log level)
- `java.io.FileReader` → Cyan (package)
- `IOException` → Light Red Bold (exception)

### Common Java Exceptions Detected

- `NullPointerException`
- `IllegalArgumentException`
- `IllegalStateException`
- `IOException`
- `FileNotFoundException`
- `SQLException`
- `RuntimeException`
- `ClassNotFoundException`
- `NoSuchMethodException`
- `ArrayIndexOutOfBoundsException`
- `NumberFormatException`
- `ParseException`
- `SecurityException`
- `UnsupportedOperationException`
- And any custom exceptions ending with "Exception"

## Implementation Details

The colorization is performed in three layers:

1. **Log Level Detection** (first pass)
   - Identifies `[DEBUG]`, `[INFO]`, `[WARN]`, `[ERROR]`
   - Applies level-specific colors

2. **Package Name Extraction** (second pass)
   - Uses multi-tier regex patterns to detect Java packages
   - Handles truncated logger names

3. **Exception Highlighting** (third pass)
   - Scans remaining text for exception names
   - Applies bold red styling to each match

## Testing

The feature includes comprehensive tests:
- `test_colorize_exceptions` - Single exception detection
- `test_colorize_multiple_exceptions` - Multiple exceptions in one line
- `test_exception_with_package_colorization` - Integration with package coloring

All tests verify that exceptions are properly styled without interfering with other colorization features.
