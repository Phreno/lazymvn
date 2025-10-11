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

## Installation

```bash
curl -sSL https://github.com/etienne/lazymvn/releases/latest/download/lazymvn -o /usr/local/bin/lazymvn
chmod +x /usr/local/bin/lazymvn
```

## License

MIT License
