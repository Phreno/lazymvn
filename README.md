# LazyMVN

**LazyMVN** is a **cross-platform TUI tool** to interact efficiently with **Maven** projects without leaving the terminal.
Inspired by *LazyGit*, it provides a minimalist interface to explore, build, and test Maven modules via a **single Rust binary**, with no external dependencies.

## Features

- List Maven modules from a multi-module project.
- Run common Maven commands (`clean`, `install`, `package`, `test`, `dependency:tree`, etc.).
- Quickly select a Maven module or profile.
- Display Maven logs in real-time in a clear interface.
- Optional project-specific configuration via a `lazymvn.toml` file.

## Technical Stack

- **Language:** Rust
- **CLI Parser:** `clap`
- **TUI:** `ratatui`, `crossterm`
- **Fuzzy Search:** `fuzzy-matcher`
- **I/O:** `tokio`
- **XML Parsing:** `quick-xml`
- **Config:** `toml` + `serde`

## Key Bindings

| Key | Action |
|---|---|
| `Up`/`Down` | Move selection |
| `b` | Build (`package`) |
| `t` | Test |
| `c` | Clean |
| `i` | Install |
| `d` | Dependency Tree |
| `q` | Quit |

## Installation

```bash
curl -sSL https://github.com/etienne/lazymvn/releases/latest/download/lazymvn -o /usr/local/bin/lazymvn
chmod +x /usr/local/bin/lazymvn
```

## Usage

Run lazymvn from within a Maven project directory:

```bash
lazymvn
```

### Command-line Options

- `-d, --debug` - Enable debug logging to `lazymvn-debug.log` in the current directory
- `-p, --project <PATH>` - Specify the path to a Maven project (defaults to current directory)
- `-h, --help` - Display help information

### Debug Logging

When debugging issues or troubleshooting, enable debug logging:

```bash
lazymvn --debug
```

This creates a `lazymvn-debug.log` file with timestamped log entries including INFO, DEBUG, and ERROR levels. The log file does not interfere with the TUI and can be monitored in a separate terminal:

```bash
tail -f lazymvn-debug.log
```

## License

MIT License
