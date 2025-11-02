# User Documentation

This section contains documentation for users of Lazymvn.

## 🚀 Getting Started

*   **[Architecture Overview](./ARCHITECTURE.md)**: Understand how LazyMVN works
*   **[Libraries](./LIBRARIES.md)**: Reusable crates and their usage
*   **Quick Start**: See [main README](../../README.md) for installation

## 📚 Features

### Configuration
*   [Live Configuration Reload](./LIVE_CONFIG_RELOAD.md): Edit your configuration without restarting
*   [Logging Configuration](./LOGGING_CONFIG.md): Control log verbosity
*   [Custom Log Formatting](./LOG_FORMATTING.md): Customize the log output format
*   [Log4j Auto-Configuration](./LOG4J_AUTO_CONFIG.md): Automatic configuration for Log4j 1.x
*   [Log Rotation](./LOG_ROTATION.md): Automatic log file rotation
*   [Spring Boot Properties Override](./SPRING_PROPERTIES_OVERRIDE.md): Override Spring Boot properties
*   [Custom Maven Flags](./CUSTOM_FLAGS.md): Define project-specific Maven flags
*   [Custom Goals](./CUSTOM_GOALS.md): Define custom Maven goals

### Performance
*   [Caching System](./CACHING.md): Profile and starter caching for faster startup

### Maven Integration
*   [Profile Activation](./PROFILE_ACTIVATION.md): How to manage Maven profiles
*   [WAR Module Support](./WAR_MODULE_SUPPORT.md): Support for `exec:java` in WAR modules

### Spring Boot
*   [Quickstart Launch Strategy](./QUICKSTART_LAUNCH_STRATEGY.md): A quick guide to the launch strategy
*   [Spring Boot Launcher](./SPRING_BOOT_LAUNCHER.md): Detailed documentation on the Spring Boot launcher

### Application
*   [Process Cleanup](./PROCESS_CLEANUP.md): How Lazymvn handles process cleanup on exit
*   [Yanking Output and Debug Info](./YANK_AND_DEBUG.md): How to copy output and debug information

## 🛠️ Development

*   **[Libraries](./LIBRARIES.md)**: Using LazyMVN libraries in your project
*   **[Architecture](./ARCHITECTURE.md)**: System design and data flow
*   **Contributing**: See [CONTRIBUTING.md](../../CONTRIBUTING.md)

## 📖 Quick Reference

### Configuration Files
- Main config: `~/.config/lazymvn/lazymvn.toml`
- Project config: `./lazymvn.toml` (in project root)

### Data Directories
- Cache: `~/.local/share/lazymvn/cache/`
- History: `~/.local/share/lazymvn/history/`
- Logs: `~/.local/share/lazymvn/logs/`

### Log Files
- Debug log: `~/.local/share/lazymvn/logs/debug.log`
- Session logs: `~/.local/share/lazymvn/logs/sessions/`
