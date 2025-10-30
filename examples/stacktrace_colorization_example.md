# Stack Trace Colorization Example

This example demonstrates the automatic colorization of Java stack trace lines in log output.

## Feature Description

The text processing utility automatically detects and highlights Java stack trace lines with distinct colors for each component.

## Stack Trace Format

**Standard Java stack trace line:**
```
at com.example.MyClass.myMethod(MyClass.java:42)
    at org.springframework.boot.SpringApplication.run(SpringApplication.java:1234)
        at com.example.Application.main(Application.java:15)
```

## Pattern Detection

**Regex Pattern:** `^\s*at\s+([a-zA-Z0-9_.$]+)\.([a-zA-Z0-9_<>]+)\(([^)]+)\)\s*$`

This pattern matches:
- Optional leading whitespace (indentation)
- The keyword `at` followed by a space
- **Capture 1:** Full class path including packages (supports `$` for inner classes)
- A dot separator
- **Capture 2:** Method name (supports generics like `<init>`, `<clinit>`)
- **Capture 3:** Source location in parentheses (e.g., `MyClass.java:42`)

## Colorization Strategy

Each part of the stack trace is colored distinctly:

| Component | Color | Style | Purpose |
|-----------|-------|-------|---------|
| `at` keyword | Dark Gray | Normal | Subtle, not distracting |
| Class path | Cyan | Normal | Consistent with package colorization |
| `.` separator | Dark Gray | Normal | Visual structure |
| Method name | Light Yellow | Normal | Stands out from class |
| `(` `)` parentheses | Dark Gray | Normal | Structural elements |
| Source location | Gray | Normal | Reference information |

## Examples

### Basic Stack Trace
```
at com.example.service.UserService.createUser(UserService.java:42)
```
**Highlighted:**
- `at ` → Dark Gray
- `com.example.service.UserService` → Cyan
- `.` → Dark Gray
- `createUser` → Light Yellow
- `(` → Dark Gray
- `UserService.java:42` → Gray
- `)` → Dark Gray

### With Inner Classes
```
at com.example.OuterClass$InnerClass.method(OuterClass.java:99)
```
**Highlighted:**
- `at ` → Dark Gray
- `com.example.OuterClass$InnerClass` → Cyan (includes `$`)
- `.` → Dark Gray
- `method` → Light Yellow
- `(OuterClass.java:99)` → Gray with Dark Gray parentheses

### With Generic Methods
```
at org.springframework.boot.SpringApplication.<init>(SpringApplication.java:123)
```
**Highlighted:**
- Constructor method `<init>` is properly detected and colored Light Yellow
- Common for constructors and static initializers (`<clinit>`)

### Complete Exception with Stack Trace
```
[ERROR] java.lang.NullPointerException: Cannot invoke method on null object
    at com.example.service.UserService.findUser(UserService.java:42)
    at com.example.controller.UserController.getUser(UserController.java:28)
    at org.springframework.web.method.support.InvocableHandlerMethod.invoke(InvocableHandlerMethod.java:215)
```

**Color hierarchy:**
1. `[ERROR]` → Red (log level)
2. `NullPointerException` → Light Red Bold (exception name)
3. Each stack trace line colored according to the scheme above

## Integration with Other Features

The stack trace colorization works seamlessly with:

- **Exception highlighting**: Exception names in the message are highlighted in bold red
- **Package detection**: Class paths use the same cyan color as regular package names
- **Log level coloring**: Error/warning levels remain clearly visible
- **Indentation preservation**: Leading whitespace is maintained for visual hierarchy

## Edge Cases Handled

### Not a Stack Trace
Lines containing "at" but not matching the stack trace format are handled normally:
```
[INFO] Starting application at port 8080
```
This is recognized as a regular log line, not a stack trace.

### Multiple Nested Packages
```
at org.springframework.boot.context.embedded.tomcat.TomcatEmbeddedServletContainer$1.run(TomcatEmbeddedServletContainer.java:197)
```
Deep package hierarchies and complex inner class notation are fully supported.

## Testing

The feature includes comprehensive tests:
- `test_colorize_stacktrace` - Basic stack trace detection and coloring
- `test_colorize_stacktrace_with_generics` - Constructor/static initializer methods
- `test_colorize_stacktrace_with_inner_class` - Inner class notation with `$`
- `test_normal_line_vs_stacktrace` - Distinguish stack traces from normal lines

All tests verify proper colorization without false positives.

## Visual Benefits

The stack trace colorization provides:
- **Quick scanning**: Yellow method names stand out immediately
- **Structure clarity**: Gray parentheses and separators provide visual structure
- **Consistency**: Cyan packages match other package references
- **Reduced noise**: Subtle dark gray for `at` keyword doesn't distract
- **Better readability**: Each component has a distinct, meaningful color
