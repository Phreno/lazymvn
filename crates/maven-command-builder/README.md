# Maven Command Builder

A fluent API for building and executing Maven commands in Rust.

[![Crates.io](https://img.shields.io/crates/v/maven-command-builder.svg)](https://crates.io/crates/maven-command-builder)
[![Documentation](https://docs.rs/maven-command-builder/badge.svg)](https://docs.rs/maven-command-builder)
[![License](https://img.shields.io/crates/l/maven-command-builder.svg)](LICENSE)

## Features

- 🔧 **Fluent Builder API** - Chainable methods for intuitive command construction
- 🎯 **Type-Safe** - Compile-time guarantees for valid Maven commands
- 📦 **Auto-Detection** - Automatically detects `mvnw` wrapper or system `mvn`
- 🚀 **Execution Support** - Built-in synchronous command execution
- ⚙️ **Rich Options** - Profiles, properties, flags, modules, and more
- 🧪 **Well-Tested** - Comprehensive test coverage
- 🪶 **Zero Dependencies** - Lightweight with no external dependencies

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
maven-command-builder = "0.1.0"
```

## Quick Start

```rust
use maven_command_builder::MavenCommandBuilder;
use std::path::Path;

// Build a simple Maven command
let cmd = MavenCommandBuilder::new(Path::new("/path/to/project"))
    .goal("clean")
    .goal("install")
    .skip_tests(true)
    .build();

println!("Command: {}", cmd);
// Output: mvn clean install -DskipTests
```

## Usage Examples

### Basic Build

```rust
use maven_command_builder::MavenCommandBuilder;
use std::path::Path;

let cmd = MavenCommandBuilder::new(Path::new("/project"))
    .goal("clean")
    .goal("package")
    .build();
```

### With Profiles and Properties

```rust
let cmd = MavenCommandBuilder::new(Path::new("/project"))
    .goal("package")
    .profile("production")
    .profile("optimized")
    .property("env", "prod")
    .property("log.level", "INFO")
    .build();

// Output: mvn -Pproduction,optimized -Denv=prod -Dlog.level=INFO package
```

### Multi-Module Projects

```rust
let cmd = MavenCommandBuilder::new(Path::new("/project"))
    .goal("install")
    .module("backend-service")
    .also_make(true)  // Build dependencies too
    .build();

// Output: mvn -pl backend-service --also-make install
```

### Fast Build with Parallel Execution

```rust
let cmd = MavenCommandBuilder::new(Path::new("/project"))
    .goal("clean")
    .goal("install")
    .skip_tests(true)
    .threads("2C")  // 2 threads per CPU core
    .offline(true)
    .build();

// Output: mvn -T 2C --offline -DskipTests clean install
```

### Spring Boot Development

```rust
let cmd = MavenCommandBuilder::new(Path::new("/project"))
    .goal("spring-boot:run")
    .profile("dev")
    .property("spring.profiles.active", "development")
    .property("server.port", "8081")
    .build();
```

### Execute Commands

```rust
use maven_command_builder::{MavenCommandBuilder, execute_maven_command};
use std::path::Path;

let builder = MavenCommandBuilder::new(Path::new("/project"))
    .goal("clean")
    .goal("compile");

match execute_maven_command(&builder) {
    Ok(output) => {
        for line in output {
            println!("{}", line);
        }
    }
    Err(e) => eprintln!("Build failed: {}", e),
}
```

### Check Maven Availability

```rust
use maven_command_builder::check_maven_availability;
use std::path::Path;

match check_maven_availability(Path::new("/project")) {
    Ok(version) => println!("Maven version: {}", version),
    Err(e) => eprintln!("Maven not found: {}", e),
}
```

## API Reference

### MavenCommandBuilder Methods

| Method | Description | Example |
|--------|-------------|---------|
| `new(path)` | Create new builder | `MavenCommandBuilder::new(Path::new("/project"))` |
| `maven_executable(cmd)` | Set custom Maven command | `.maven_executable("mvn")` |
| `goal(goal)` | Add a Maven goal | `.goal("clean")` |
| `goals(goals)` | Add multiple goals | `.goals(vec!["clean", "install"])` |
| `profile(profile)` | Add a profile | `.profile("production")` |
| `profiles(profiles)` | Add multiple profiles | `.profiles(vec!["dev", "fast"])` |
| `property(key, value)` | Add a property (-D) | `.property("skipTests", "true")` |
| `flag(flag)` | Add a custom flag | `.flag("--debug")` |
| `module(module)` | Specify a module | `.module("backend")` |
| `settings_file(path)` | Set settings file | `.settings_file("settings.xml")` |
| `threads(count)` | Set thread count | `.threads("2C")` |
| `use_file_flag(bool)` | Use -f instead of -pl | `.use_file_flag(true)` |
| `offline(bool)` | Enable offline mode | `.offline(true)` |
| `update_snapshots(bool)` | Update snapshots | `.update_snapshots(true)` |
| `skip_tests(bool)` | Skip tests | `.skip_tests(true)` |
| `also_make(bool)` | Add --also-make | `.also_make(true)` |
| `also_make_dependents(bool)` | Add --also-make-dependents | `.also_make_dependents(true)` |
| `build()` | Build command string | `.build()` |
| `build_args()` | Build args vector | `.build_args()` |

## Common Patterns

### CI/CD Build

```rust
let cmd = MavenCommandBuilder::new(Path::new("/project"))
    .goal("clean")
    .goal("verify")
    .profile("ci")
    .property("maven.test.failure.ignore", "false")
    .threads("2C")
    .update_snapshots(true)
    .build();
```

### Release Preparation

```rust
let cmd = MavenCommandBuilder::new(Path::new("/project"))
    .goal("clean")
    .goal("deploy")
    .profile("release")
    .property("gpg.skip", "false")
    .threads("1C")
    .build();
```

### Debugging Build Issues

```rust
let cmd = MavenCommandBuilder::new(Path::new("/project"))
    .goal("clean")
    .goal("install")
    .flag("--debug")
    .flag("--errors")
    .property("maven.compiler.verbose", "true")
    .build();
```

### Integration Tests Only

```rust
let cmd = MavenCommandBuilder::new(Path::new("/project"))
    .goal("verify")
    .property("skipUTs", "true")
    .property("skipITs", "false")
    .build();
```

## Maven Wrapper Auto-Detection

The builder automatically detects and uses Maven wrapper if available:

1. On Unix: Looks for `mvnw` in project root
2. On Windows: Looks for `mvnw.bat`, `mvnw.cmd`, or `mvnw`
3. Falls back to system `mvn` if wrapper not found

You can override this with `.maven_executable("custom-mvn")`.

## Thread Configuration

The `threads()` method supports Maven's thread count syntax:

- `"4"` - 4 threads total
- `"2C"` - 2 threads per CPU core
- `"1.5C"` - 1.5 threads per CPU core

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Related Projects

- [lazymvn](https://github.com/phreno/lazymvn) - A fast TUI for Maven (uses this library)
- [maven-log-analyzer](https://crates.io/crates/maven-log-analyzer) - Maven log analysis
- [maven-log-colorizer](https://crates.io/crates/maven-log-colorizer) - Maven log colorization
