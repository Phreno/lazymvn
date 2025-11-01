# Maven Log Colorizer

A Rust library for colorizing Maven build logs with beautiful syntax highlighting.

## Features

- **🎨 Log Level Highlighting**: Automatic color coding for INFO, DEBUG, WARNING, and ERROR levels
- **📦 Package Name Detection**: Extracts and highlights Java package names from logs
- **🔴 Exception Highlighting**: Detects and highlights Java exception names (NullPointerException, IOException, etc.)
- **📍 Stack Trace Coloring**: Beautiful syntax highlighting for stack traces with separate colors for:
  - Class paths (cyan)
  - Method names (light yellow)
  - Source locations (gray)
- **💻 Command Line Highlighting**: Special styling for command lines starting with `$`
- **🧩 Ratatui Integration**: Returns `Line<'static>` for direct use in ratatui TUI applications

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
maven-log-colorizer = "0.1.0"
```

## Usage

### Basic Usage

```rust
use maven_log_colorizer::colorize_log_line;
use ratatui::text::Line;

let log_line = "[INFO] Building project";
let colored: Line<'static> = colorize_log_line(log_line);

// Use directly in ratatui Paragraph widget
use ratatui::widgets::Paragraph;
let paragraph = Paragraph::new(vec![colored]);
```

### With Log Format Pattern

For better package name extraction, provide your log4j pattern:

```rust
use maven_log_colorizer::colorize_log_line_with_format;

let log_line = "[INFO] com.example.MyClass - Processing data";
let log_format = "[%p] %c - %m%n";  // Log4j pattern

let colored = colorize_log_line_with_format(log_line, Some(log_format));
// Package name "com.example.MyClass" will be highlighted in cyan
```

### Cleaning ANSI Escape Codes

If your logs contain ANSI escape sequences, clean them first:

```rust
use maven_log_colorizer::{clean_log_line, colorize_log_line};

let raw_log = "\x1b[0m[INFO]\x1b[0m Building project";
let cleaned = clean_log_line(raw_log);
let colored = colorize_log_line(&cleaned);
```

## Color Scheme

| Element | Color | Style |
|---------|-------|-------|
| `[INFO]` | Green | Normal |
| `[DEBUG]` | Magenta | Normal |
| `[WARNING]` / `[WARN]` | Yellow | Normal |
| `[ERROR]` / `[ERR]` | Red | Normal |
| Package names | Cyan | Normal |
| Exceptions | Light Red | Bold |
| Stack trace `at` | Dark Gray | Normal |
| Stack trace class | Cyan | Normal |
| Stack trace method | Light Yellow | Normal |
| Stack trace location | Gray | Normal |
| Command lines (`$ ...`) | Cyan | Bold |

## Examples

### Exception Highlighting

```rust
let log_line = "[ERROR] Failed with NullPointerException: null value";
let colored = colorize_log_line(log_line);
// Output: [ERROR] in red, "NullPointerException" in bold light red
```

### Stack Trace Highlighting

```rust
let log_line = "    at com.example.MyClass.myMethod(MyClass.java:42)";
let colored = colorize_log_line(log_line);
// Output:
//   "at" in dark gray
//   "com.example.MyClass" in cyan
//   "myMethod" in light yellow
//   "(MyClass.java:42)" in gray
```

### Command Line Highlighting

```rust
let log_line = "$ mvn clean install -DskipTests";
let colored = colorize_log_line(log_line);
// Output: entire line in bold cyan
```

## Log Format Patterns

Supported log4j pattern elements for package extraction:

- `%c` - Full logger/class name
- `%C` - Full class name
- `%logger` - Logger name
- `%class` - Class name

Example patterns:
- `[%p] %c - %m%n` - Standard log4j pattern
- `%d{HH:mm:ss} [%p] %c{1}: %m%n` - With timestamp and short logger

## Dependencies

- `ratatui` - For terminal UI styling
- `regex` - For pattern matching
- `maven-log-analyzer` - For log parsing and pattern detection

## Integration with Ratatui

Perfect for TUI applications:

```rust
use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use maven_log_colorizer::colorize_log_line;

fn render_logs(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, logs: &[String]) {
    terminal.draw(|f| {
        let colored_lines: Vec<_> = logs
            .iter()
            .map(|log| colorize_log_line(log))
            .collect();
        
        let paragraph = Paragraph::new(colored_lines)
            .block(Block::default().borders(Borders::ALL).title("Build Logs"));
        
        f.render_widget(paragraph, f.size());
    }).unwrap();
}
```

## Performance

- Zero-cost abstractions with static lifetimes
- Lazy regex compilation using `lazy_static`
- Efficient string slicing without allocations where possible

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! This library is part of the [LazyMvn](https://github.com/phreno/lazymvn) project.
